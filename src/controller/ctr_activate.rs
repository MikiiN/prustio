use std::env;

use crate::model::{prustio_config, board, cargo_toml};
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

    let mut config = prustio_config::get_config(&proj_path)?;
    config.set_active_env(&env.name)?;
    config.save(&proj_path)?;

    let board = board::get_board(&env.board)?;

    cargo_toml::create_cargo_toml_config(
        &proj_path, 
        &package.name, 
        &board.cargo_feature, 
        &package.hybrid_mode
    )?;

    Ok(())
}