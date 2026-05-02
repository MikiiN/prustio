use std::path::PathBuf;

use crate::model::{
    board, 
    cargo_config_toml,
    cargo_toml, 
    prustio_config, 
    toolchain_toml, 
    source_code
};
use crate::ui;
use crate::wrapper::cargo;

const DEFAULT_PROJECT_NAME: &str = "project";

pub fn init_project(
    name: &Option<String>, 
    board_id: &Option<String>, 
    hybrid: &bool, 
    json_output: &bool,
) -> Result<(), String> {
    let proj_name = match name {
        Some(n) => n,
        None => &String::from(DEFAULT_PROJECT_NAME),
    };
    let proj_path: PathBuf = PathBuf::from(&proj_name);

    if proj_path.exists() {
        return Err("The project or directory with same name already exists.".to_string());  
    }

    let board = match board_id {
        Some(id) => {
            board::get_board(id)?
        },
        None => board::get_unspecified_board()
    };

    let board_arch = board.platform.to_cargo_arch();

    if !*json_output {
        ui::display::info("Initiating cargo project...");
    }
    cargo_init(&proj_path, proj_name, &board_arch, &board.mcu, &board.cargo_feature, &board.rustc_version, hybrid)?;

    // TODO - hardcoded
    let framework = if *hybrid {
            Some(&String::from("arduino"))
        } else {
            None
        };
    
    prustio_init(&proj_path, &proj_name, hybrid, &board.id, framework)?;
    
    Ok(())
}

fn cargo_init(
    proj_path: &PathBuf,
    proj_name: &String, 
    board_arch: &String, 
    board_mcu: &String,
    cargo_feature: &String,
    rustc_version: &String,
    hybrid: &bool,
) -> Result<(), String> {
    cargo::init_cargo(proj_path)?;

    toolchain_toml::create_toolchain_config(proj_path, rustc_version, &None, &None)?;

    cargo_config_toml::create_cargo_config(proj_path, board_arch, board_mcu)?;

    cargo_toml::create_cargo_toml_config(proj_path, proj_name,cargo_feature, hybrid)?;

    source_code::write_example_code(proj_path, hybrid)?;

    Ok(())
}

fn prustio_init(
    proj_path: &PathBuf, 
    proj_name: &String, 
    hybrid: &bool,
    board_id: &String,
    framework: Option<&String>,
) -> Result<(), String> {
    prustio_config::create_prustio_config(proj_path, proj_name, hybrid, board_id, framework)?;
    Ok(())
}