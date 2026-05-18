//! Controller for activating a specific environment.
//!
//! When a user runs the `prustio activate <env>` command, this module handles 
//! the transition. It updates the active environment in `Prustio.toml`, 
//! regenerates the `Cargo.toml` file to inject the correct `arduino-hal` 
//! features, and modifies `.cargo/config.toml` to reflect the new target 
//! architecture and MCU.

use std::env;

use crate::model::{prustio_config, board, cargo_toml, cargo_config_toml};
use crate::ui::display;
use crate::utils;


/// Activates a specific hardware environment within the project.
///
/// This function coordinates the updates across the various configuration files 
/// needed to tell Cargo how to cross-compile for the newly selected board.
///
/// # Arguments
/// * `environment` - A reference to the string name of the environment to activate.
/// * `json_output` - If `true`, suppresses standard console logs for JSON compatibility.
///
/// # Errors
/// Returns an error string if:
/// * The command is run outside of a valid pRustIO project directory.
/// * The specified `environment` does not exist in the configuration.
/// * Reading or writing to `Prustio.toml`, `Cargo.toml`, or `.cargo/config.toml` fails.
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
    let user_dependencies = config.get_user_defined_dependencies();

    cargo_toml::create_cargo_toml_config(
        &proj_path, 
        &package.name, 
        &board.cargo_feature, 
        &package.hybrid_mode,
        user_dependencies,
    )?;

    let board_arch = board.platform.to_cargo_arch();
    cargo_config_toml::update_cargo_config(&proj_path, &board_arch, &board.mcu, None)?;
    if !*json_output { display::info(&format!("Environment activated.")); }

    Ok(())
}