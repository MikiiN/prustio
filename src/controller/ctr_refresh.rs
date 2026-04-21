use std::env;

use crate::controller::ctr_activate;
use crate::model::prustio_config;
use crate::utils;

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