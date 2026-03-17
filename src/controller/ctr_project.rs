use std::path::PathBuf;

use crate::wrapper::cargo;
use crate::model::{
    boards, 
    cargo_config_toml,
    cargo_toml, 
    prustio_config, 
    toolchain_toml, 
    source_code
};

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
        },
        None => boards::get_unspecified_board()
    };

    let board_arch = board.platform.to_cargo_arch();

    match cargo_init(&proj_path, &board_arch, &board.mcu, &board.cargo_feature, &board.rustc_version) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    }

    // TODO - hardcoded
    let framework = if *hybrid {
            Some(&String::from("arduino"))
        } else {
            None
        };
    match prustio_init(&proj_path, &proj_name, hybrid, &board.id, framework) {
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
    rustc_version: &String,
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

    match toolchain_toml::create_toolchain_config(proj_path, rustc_version) {
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

    match cargo_toml::create_cargo_toml_config(proj_path, cargo_feature) {
        Ok(_) => {},
        Err(_) => {
            return Err(String::from("Failed to create cargo configuration."));
        }
    }

    match source_code::write_example_code(proj_path) {
        Ok(_) => {},
        Err(e) => {
            return Err(e);
        }
    }

    Ok(())
}

fn prustio_init(
    proj_path: &PathBuf, 
    proj_name: &String, 
    hybrid: &bool,
    board_id: &String,
    framework: Option<&String>,
) -> Result<(), String> {
    match prustio_config::create_prustio_config(proj_path, proj_name, hybrid, board_id, framework) {
        Ok(_) => {},
        Err(_) => {
            return Err(String::from("Failed to create PrustIO configuration file."));
        },
    };
    Ok(())
}