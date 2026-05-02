use std::env;

use crate::model::{prustio_config, board, cargo_toml, cargo_config_toml};
use crate::ui::display;
use crate::utils;


pub fn activate_environment(environment: &String, json_output: &bool) -> Result<(), String> {
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
    let env = prustio_config::get_env(&proj_path, Some(environment))?;
    
    if !*json_output { display::info(&format!("Activating environment '{}'...", environment)); }
    let mut config = prustio_config::get_config(&proj_path)?;
    config.set_active_env(environment)?;
    config.save(&proj_path)?;
    
    let board = board::get_board(&env.board)?;
    
    cargo_toml::create_cargo_toml_config(
        &proj_path, 
        &package.name, 
        &board.cargo_feature, 
        &package.hybrid_mode
    )?;

    let board_arch = board.platform.to_cargo_arch();
    cargo_config_toml::update_cargo_config(&proj_path, &board_arch, &board.mcu, None)?;
    if !*json_output { display::info(&format!("Environment activated.")); }

    Ok(())
}