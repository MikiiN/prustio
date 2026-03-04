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
    let proj_path: PathBuf = PathBuf::from(&proj_name);

    if proj_path.exists() {
        eprintln!("Error: The project or folder with same name already exists.");
        return;
    }

    let board = match board_id {
        Some(id) => {
            match boards::get_board(id) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return;
                }
            }
        }
        None => boards::get_unspecified_board()
    };

    let board_arch = match board.get_architecture() {
        Some(arch) => arch,
        None => {
            eprintln!("Error: Unsupported board.");
            return;
        }
    };
    let board_feature_cargo = match board.get_cargo_feature() {
        Some(feature) => feature,
        None => {
            eprintln!("Error: Unsupported board.");
            return;
        }
    };


    match cargo_init(&proj_path, &board_arch, &board.mcu, &board_feature_cargo) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    }

    match prustio_init(&proj_path, &proj_name, hybrid) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    }
    
   
}

fn cargo_init(
    proj_path: &PathBuf, 
    board_arch: &String, 
    board_mcu: &String,
    cargo_feature: &String,
) -> Result<(), String> {
    match cargo::init_cargo(proj_path) {
        Ok(status) => {
            if !status.success() {
                return Err(format!("Cargo init failed with code: {status}"));
            }
        },
        Err(e) => {
            return Err(format!("Failed to execute the cargo command:\n {e}"));
        }
    }

    match toolchain_toml::create_toolchain_config(proj_path) {
        Ok(_) => {},
        Err(_) => {
            return Err(String::from("Failed to create toolchain configuration file."));
        },
    }

    match cargo_config_toml::create_cargo_config(proj_path, board_arch, board_mcu) {
        Ok(_) => {},
        Err(_) => {
            return Err(String::from("Failed to create cargo configuration."));
        }
    };

    match cargo_config_toml::create_cargo_config(proj_path, board_arch, board_mcu) {
        Ok(_) => {},
        Err(_) => {
            return Err(String::from("Failed to create cargo configuration."));
        }
    };

    match cargo_toml::create_cargo_toml_config(proj_path, cargo_feature) {
        Ok(_) => {},
        Err(_) => {
            return Err(String::from("Failed to create cargo configuration."));
        }
    }

    Ok(())
}

fn prustio_init(proj_path: &PathBuf, proj_name: &String, hybrid: &bool) -> Result<(), String> {
    match prustio_config::create_prustio_config(proj_path, proj_name, hybrid) {
        Ok(_) => {},
        Err(_) => {
            return Err(String::from("Failed to create PrustIO configuration file."));
        },
    };
    Ok(())
}