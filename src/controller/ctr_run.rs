use std::{env, path};
use std::path::PathBuf;

use crate::model::board::Board;
use crate::model::prustio_config::Env;
use crate::model::{board, build, cargo_config_toml, cargo_toml, device, prustio_config};
use crate::utils;
use crate::wrapper::avr::obtain_bin_path;
use crate::wrapper::{cargo, avr, avrdude, platformio};

const DEFAULT_ELF_BIN_NAME: &str = "bin.elf";
const DEFAULT_HEX_BIN_NAME: &str = "bin.hex"; 

pub fn run(
    target: &Option<String>,
    environment: Option<&String>,
    json_output: &bool,
) {
    let proj_path = match env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            eprintln!("Error: Failed to get current working directory.");
            return;
        },
    };
    if !utils::check_if_is_project_dir(&proj_path) {
        eprintln!("Error: Not in project dir.");
        return;
    }

    let package = match prustio_config::get_package_information(&proj_path) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

    let env = match prustio_config::get_env(&proj_path, environment) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

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

    let board = match board::get_board(&env.board) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };
    let board_arch = board.platform.to_cargo_arch();

    if package.hybrid_mode {
        if let Err(msg) = prepare_hybrid_mode_compilation(&proj_path, &board, &env) {
            eprintln!("Error: {}", msg);
            return;
        }
    }

    // TODO add linker only when hybrid mode is used
    let linker = match avr::obtain_bin_path(avr::GCC_BINARY_NAME) {
        Ok(path) => match path.to_str() {
            Some(str_path) => str_path.to_string(),
            None => {
                eprintln!("Error: Failed to obtain avr-gcc binary path.");
                return;
            }
        },
        Err(msg) => {
            eprintln!("Error: {}",msg);
            return;
        }
    };

    match cargo_config_toml::update_cargo_config(&proj_path, &board_arch, &board.mcu, Some(&linker)) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    }

    match cargo_toml::create_cargo_toml_config(
        &proj_path, 
        &package.name, 
        &board.cargo_feature, 
        &package.hybrid_mode
    ) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    }

    match cargo::cargo_build(&proj_path, &None) {
        Ok(_) => {},
        Err(_) => {
            eprintln!("Error: Failed to build project");
            return;
        }
    }
    let binary_path = get_binary_dir_path(&board_arch);
    let elf_bin_path = binary_path.join(DEFAULT_ELF_BIN_NAME);
    let hex_bin_path = binary_path.join(DEFAULT_HEX_BIN_NAME);

    match avr::elf_to_hex(&elf_bin_path, &hex_bin_path) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    }

    let device = match device::get_connected_device_list() {
        Ok(mut ports) => {
            match ports.pop() {
                Some(p) => p,
                None => {
                    eprintln!("Error: No connected device to upload.");
                    return;        
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    match avrdude::upload_binary(&hex_bin_path, &board.mcu, &board.upload_protocol, &device.port, &board.bus_speed) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    }
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
    platformio::init_compilation_project(proj_dir, &board.platform.to_string(), &board.id, &framework)?;
    
    platformio::compile_c_libraries(proj_dir, &board.id)?;

    let lib_names = utils::get_compiled_libs_names(proj_dir);

    build::write_build_configuration(proj_dir, &lib_names)?;

    Ok(())
}