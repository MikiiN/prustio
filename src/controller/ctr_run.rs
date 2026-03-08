use std::env;

use crate::model::prustio_config;
use crate::utils;

pub fn run(
    target: &Option<String>,
    environment: &Option<String>,
    json_output: &bool,
) {
    // TODO check if current dir is project
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

    let envs = match prustio_config::get_env(&proj_path) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };
    match environment {
        Some(e) => {
            if envs.envs.contains_key(e) {

            }
        },
        None => {

        }
    };
}