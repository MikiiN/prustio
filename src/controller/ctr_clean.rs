use std::env;

use crate::{utils, wrapper};

pub fn clean(json_output: &bool) -> Result<(), String> {
    let proj_path = match env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            return Err("Failed to get current working directory.".to_string());
        },
    };

    if !utils::check_if_is_project_dir(&proj_path) {
        return Err("Not in project dir.".to_string());
    }

    let target_dir = proj_path.join("target");
    let prio_dir = proj_path.join(utils::PROJECT_APP_DIR_NAME);
    
    if prio_dir.exists() {
        utils::clear_dir(&prio_dir)?;
    }

    if target_dir.exists() {
        wrapper::cargo::cargo_clean(&proj_path)?;
    }

    Ok(())
}