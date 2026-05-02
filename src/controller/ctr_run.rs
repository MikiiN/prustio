use std::env;
use std::path::PathBuf;

use crate::model::board::Board;
use crate::model::prustio_config::{Env, Package, };
use crate::model::{board, build, cargo_config_toml, cargo_toml, device, platformio_lock, prustio_config};
use crate::ui::display;
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

    let targets = match target {
        Some(t) => {
            Vec::from([Target::from(t)?])
        },
        None => {
            match &env.targets {
                Some(ts) => {
                    let mut targets = Vec::new();
                    for target in ts {
                        targets.push(Target::from(target)?);
                    }
                    targets
                },
                None => {
                    Vec::from([Target::Upload])
                }
            }
        }
    };

    let board = board::get_board(&env.board)?;
    let board_arch = board.platform.to_cargo_arch();

    let binary_path = get_binary_dir_path(&board_arch);
    let elf_bin_path = binary_path.join(DEFAULT_ELF_BIN_NAME);
    let hex_bin_path = binary_path.join(DEFAULT_HEX_BIN_NAME);

    let mut config = prustio_config::get_config(&proj_path)?;
    config.set_active_env(&env.name)?;
    config.save(&proj_path)?;

    for t in targets {
        match t {
            Target::Build => {
                build_project(&proj_path, &package, &board, &env, &board_arch, &elf_bin_path, &hex_bin_path, json_output)?;
            },
            Target::Upload => {
                build_project(&proj_path, &package, &board, &env, &board_arch, &elf_bin_path, &hex_bin_path, json_output)?;
                upload_project(&board, &hex_bin_path, json_output)?;
            }
        }
    }


    Ok(())
}

enum Target {
    Build,
    Upload,
}

impl Target {
    fn from(text: &String) -> Result<Target, String> {
        let value = text.to_ascii_lowercase();
        match value.as_str() {
            "build" => Ok(Target::Build),
            "upload" => Ok(Target::Upload),
            _ => Err("Invalid target name.".to_string())
        }
    }

    fn to_string(&self) -> String {
        match self {
            Target::Build => "build".to_string(),
            Target::Upload => "upload".to_string(),
        }
    }
}

fn build_project(
    proj_path: &PathBuf,
    package: &Package,
    board: &Board,
    env: &Env,
    board_arch: &String,
    elf_bin_path: &PathBuf,
    hex_bin_path: &PathBuf,
    json_output: &bool,
) -> Result<(), String> {
    if !*json_output { 
        display::info("Starting build process..."); 
    }

    if !*json_output { display::info("Configuring cargo..."); }
    if package.hybrid_mode {
        prepare_hybrid_mode_compilation(proj_path, board, env, json_output)?;
        
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
        cargo_config_toml::update_cargo_config(proj_path, &board_arch, &board.mcu, Some(&linker))?;
    } else {
        cargo_config_toml::update_cargo_config(proj_path, &board_arch, &board.mcu, None)?;
    }

    cargo_toml::create_cargo_toml_config(
        proj_path, 
        &package.name, 
        &board.cargo_feature, 
        &package.hybrid_mode
    )?;

    if !*json_output { display::info("Building project..."); }
    cargo::cargo_build(proj_path, &None)?;

    if !*json_output { display::info("Converting ELF to HEX format..."); }
    avr::elf_to_hex(elf_bin_path, hex_bin_path)?;

    if !*json_output { 
        display::info("Build finished."); 
    }

    Ok(())
}

fn upload_project(
    board: &Board,
    hex_bin_path: &PathBuf,
    json_output: &bool,
) -> Result<(), String> {
    if !*json_output { 
        display::info("Uploading binary file..."); 
    }

    if !*json_output { display::info("Detecting connected devices..."); }
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

    if !*json_output { display::info(&format!("Uploading binary to {}...", device.port)); }
    avrdude::upload_binary(&hex_bin_path, &board.mcu, &board.upload_protocol, &device.port, &board.bus_speed)?;

    if !*json_output { 
        display::info("Successfully Uploaded."); 
    }
    Ok(())
}

fn get_binary_dir_path(board_arch: &String) -> PathBuf {
    PathBuf::from("target").join(board_arch).join("release")
}

fn prepare_hybrid_mode_compilation(
    proj_dir: &PathBuf,
    board: &Board,
    env: &Env,
    json_output: &bool,
) -> Result<(), String> {
    if !*json_output { display::info("Preparing PlatformIO project for hybrid mode..."); }

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

        if !*json_output { display::info("Compiling C/C++ libraries..."); }
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

        if !*json_output { display::info("Compiling C/C++ libraries..."); }
        platformio::compile_c_libraries(proj_dir, &board.id)?;

        let output = platformio::get_pio_project_dependencies(proj_dir)?;
        let deps = platformio_lock::parse_pio_list_output(&output)?;
        let lock = platformio_lock::Lockfile::new(deps, 1);
        lock.save(proj_dir)?;
    }

    if !*json_output { display::info("Writing Rust build scripts..."); }
    let lib_names = utils::get_compiled_libs_names(proj_dir);
    build::write_build_configuration(proj_dir, &lib_names)?;

    Ok(())
}