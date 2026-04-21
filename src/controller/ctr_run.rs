use std::{env, path};
use std::path::PathBuf;

use crate::model::board::Board;
use crate::model::prustio_config::Env;
use crate::model::{board, build, cargo_config_toml, cargo_toml, device, platformio_lock, prustio_config};
use crate::utils;
use crate::wrapper::{cargo, avr, avrdude, platformio};

const DEFAULT_ELF_BIN_NAME: &str = "bin.elf";
const DEFAULT_HEX_BIN_NAME: &str = "bin.hex"; 

pub fn run(
    target: &Option<String>,
    environment: Option<&String>,
    json_output: &bool,
) -> Result<(), String> {
    let proj_path = match env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            return Err("Failed to get current working directory.".to_string());
        },
    };
    if !utils::check_if_is_project_dir(&proj_path) {
        return Err("Not in project dir.".to_string());
    }

    let package = prustio_config::get_package_information(&proj_path)?;

    let env = prustio_config::get_env(&proj_path, environment)?;

    // TODO target usage
    // let target = match target {
    //     Some(t) => {
    //         // TODO target validation
    //         Vec::from([t.clone()])
    //     },
    //     None => {
    //         match &env.targets {
    //             Some(ts) => ts,
    //             None => {
    //                 let mut ts: Vec<String> = Vec::new();
    //                 ts.push(String::from("upload"));
    //                 ts
    //             }
    //         }
    //     }
    // };

    let board = board::get_board(&env.board)?;
    let board_arch = board.platform.to_cargo_arch();

    if package.hybrid_mode {
        prepare_hybrid_mode_compilation(&proj_path, &board, &env)?;
    }

    // TODO add linker only when hybrid mode is used
    let linker = match avr::obtain_bin_path(avr::GCC_BINARY_NAME) {
        Ok(path) => match path.to_str() {
            Some(str_path) => str_path.to_string(),
            None => {
                return Err("Failed to obtain avr-gcc binary path.".to_string());
            }
        },
        Err(msg) => {
            return Err(msg);
        }
    };

    cargo_config_toml::update_cargo_config(&proj_path, &board_arch, &board.mcu, Some(&linker))?;

    cargo_toml::create_cargo_toml_config(
        &proj_path, 
        &package.name, 
        &board.cargo_feature, 
        &package.hybrid_mode
    )?;

    cargo::cargo_build(&proj_path, &None)?;

    let binary_path = get_binary_dir_path(&board_arch);
    let elf_bin_path = binary_path.join(DEFAULT_ELF_BIN_NAME);
    let hex_bin_path = binary_path.join(DEFAULT_HEX_BIN_NAME);

    avr::elf_to_hex(&elf_bin_path, &hex_bin_path)?;
    
    let device = match device::get_connected_device_list() {
        Ok(mut ports) => {
            match ports.pop() {
                Some(p) => p,
                None => {
                    return Err("No connected device to upload.".to_string());        
                }
            }
        },
        Err(e) => {
            return Err(e);
        }
    };

    avrdude::upload_binary(&hex_bin_path, &board.mcu, &board.upload_protocol, &device.port, &board.bus_speed)?;

    Ok(())
}

fn get_binary_dir_path(board_arch: &String) -> PathBuf {
    PathBuf::from("target").join(board_arch).join("release")
}

fn prepare_hybrid_mode_compilation(
    proj_dir: &PathBuf,
    board: &Board,
    env: &Env,

) -> Result<(), String> {
    let framework = match &env.framework {
        Some(f) => f.clone(),
        None => "arduino".to_string()
    };
    let lock_file = platformio_lock::get_pio_lock_path(proj_dir);
    if lock_file.exists() {
        let deps = platformio_lock::Lockfile::load(proj_dir)?;
        let platform_packages = deps.get_platform_packages();
        let libs_deps = deps.get_lib_deps();

        platformio::init_compilation_project(
            proj_dir, 
            &board.platform.to_string(), 
            &board.id, 
            &framework,
            Some(&platform_packages),
            if !libs_deps.is_empty() {
                Some(&libs_deps)
            } else {
                None
            }
        )?;
        platformio::compile_c_libraries(proj_dir, &board.id)?;
    } else {
        platformio::init_compilation_project(
            proj_dir, 
            &board.platform.to_string(), 
            &board.id, 
            &framework,
            None,
            None
        )?;
        platformio::compile_c_libraries(proj_dir, &board.id)?;

        let output = platformio::get_pio_project_dependencies(proj_dir)?;
        let deps = platformio_lock::parse_pio_list_output(&output)?;
        let lock = platformio_lock::Lockfile::new(deps, 1);
        lock.save(proj_dir)?;
    }

    let lib_names = utils::get_compiled_libs_names(proj_dir);

    build::write_build_configuration(proj_dir, &lib_names)?;

    Ok(())
}