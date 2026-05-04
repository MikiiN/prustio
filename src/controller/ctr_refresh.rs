//! Controller for refreshing the current project configuration.
//!
//! When a user runs the `prustio refresh` command, this module reads the 
//! currently active environment from `Prustio.toml` and re-applies its 
//! configuration across the project. This is particularly useful if configuration 
//! files have been manually edited or if the project state needs to be resynchronized.

use std::env;

use crate::controller::ctr_activate;
use crate::model::prustio_config;
use crate::utils;

/// Refreshes the project by re-activating the current environment.
///
/// This function locates the current pRustIO project, determines the currently 
/// active environment from the package metadata, and internally calls the 
/// `activate` controller to refresh the `Cargo.toml` and `.cargo/config.toml` files.
///
/// # Arguments
/// * `json_output` - If `true`, suppresses standard console logs for JSON compatibility 
///   (this flag is passed down to the `activate_environment` call).
///
/// # Errors
/// Returns an error string if:
/// * The current working directory cannot be determined.
/// * The current directory is not a valid pRustIO project (missing `Prustio.toml`).
/// * The `Prustio.toml` configuration cannot be read or parsed.
/// * There is no currently active environment set for the project.
pub fn refresh(json_output: &bool) -> Result<(), String> {
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
    if let Some(active_env) = package.active_env {
        ctr_activate::activate_environment(&active_env, json_output)?;
    } else {
        return Err("There is no active environment.".to_string());
    }

    Ok(())
}