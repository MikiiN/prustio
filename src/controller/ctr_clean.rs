use std::env;

use crate::utils;

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

    // Use the existing clear_dir utility to remove them safely
    if target_dir.exists() {
        utils::clear_dir(&target_dir)?;
    }
    
    if prio_dir.exists() {
        utils::clear_dir(&prio_dir)?;
    }

    Ok(())
}