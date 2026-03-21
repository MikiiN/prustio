use std::env;
use std::path::PathBuf;

use crate::model::{board, cargo_config_toml, cargo_toml, prustio_config, device};
use crate::utils;
use crate::wrapper::{cargo, avr_objcopy, avrdude};

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
    if !utils::check_if_project_dir(&proj_path) {
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
    let target = match target {
        Some(t) => {
            // TODO target validation
            Vec::from([t.clone()])
        },
        None => {
            match env.targets {
                Some(ts) => ts,
                None => {
                    let mut ts: Vec<String> = Vec::new();
                    ts.push(String::from("upload"));
                    ts
                }
            }
        }
    };

    let board = match board::get_board(&env.board) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };
    let board_arch = board.platform.to_cargo_arch();

    match cargo_config_toml::update_cargo_config(&proj_path, &board_arch, &board.mcu) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    }

    match cargo_toml::create_cargo_toml_config(&proj_path, &package.name, &board.cargo_feature) {
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

    match avr_objcopy::elf_to_hex(&elf_bin_path, &hex_bin_path) {
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