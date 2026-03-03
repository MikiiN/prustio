use std::path::PathBuf;

use crate::wrapper::cargo;
use crate::model::{boards, cargo_config_toml, cargo_toml, prustio_config, toolchain_toml};

const DEFAULT_PROJECT_NAME: &str = "project";

pub fn init_project(
    name: &Option<String>, 
    board_id: &Option<String>, 
    hybrid: &bool, 
    json_output: &bool,
) {
    let proj_name = match name {
        Some(n) => n,
        None => &String::from(DEFAULT_PROJECT_NAME),
    };
    let path: PathBuf = PathBuf::from(&proj_name);

    if path.exists() {
        eprintln!("Error: The project already exists.");
        return;
    }

    let project: String = match path.into_os_string().into_string() {
        Ok(p) => p,
        Err(os_str) => {
            eprintln!("Error: Invalid working directory path:\n{:?}", os_str);
            return;
        },
    };
    let result = cargo::init_cargo(&project);
    match result {
        Ok(status) => {
            if status.success() {
                println!("Cargo project created.");
            } else {
                eprintln!("Error: Cargo init failed with code: {}", status);
            }
        },
        Err(e) => {
            eprintln!("Error: Failed to execute the cargo command:\n {}",e);
            return;
        }
    }

    match prustio_config::create_prustio_config(&project, &proj_name, hybrid) {
        Ok(_) => {},
        Err(_) => {
            eprintln!("Error: Failed to create PrustIO configuration file.");
            return;
        },
    };

    match toolchain_toml::create_toolchain_config(&project) {
        Ok(_) => {},
        Err(_) => {
            eprintln!("Error: Failed to create toolchain configuration file.");
            return;
        },
    }

    let (b_arch, b_mcu): (Option<String>, Option<String>) = match board_id {
        Some(id) => {
            match boards::get_board(id) {
                Ok(b) => {
                    let arch = match b.get_architecture() {
                        Some(a) => a,
                        None => {
                            eprintln!("Error: Unsupported board");
                            return;
                        },
                    };
                    let feature = match b.get_cargo_feature() {
                        Some(f) => f,
                        None => {
                            "None"
                        }
                    };
                    match cargo_config_toml::create_cargo_config(&project, &arch, &b.mcu) {
                        Ok(_) => {},
                        Err(_) => {
                            eprintln!("Error: Failed to create cargo configuration.");
                            return;
                        }
                    };
                     match cargo_toml::create_cargo_toml_config(&project, &feature) {
                        Ok(_) => {},
                        Err(_) => {
                            eprintln!("Error: Failed to modify Cargo.toml file.");
                            return;
                        }
                    }

                    (Some(arch), Some(b.mcu))
                },
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            }
        },
        None => (None, None),
    };
}