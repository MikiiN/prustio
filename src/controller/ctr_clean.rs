//! Controller for cleaning project build artifacts.
//!
//! When a user runs the `prustio clean` command, this module is responsible
//! for safely removing the `target/` directory (created by Cargo) and the 
//! `.prio/` directory (created by pRustIO for PlatformIO dependencies and 
//! compiled C/C++ libraries).

use std::env;

use crate::{utils, wrapper};
use crate::ui::display::info;

/// Removes build files from the project.
///
/// This function locates the current working directory, verifies it is a valid 
/// pRustIO project, and deletes the `target` and `.prio` directories to free up 
/// disk space or force a completely clean build on the next run.
///
/// # Arguments
/// * `json_output` - If `true`, suppresses standard console logs for JSON compatibility.
///
/// # Errors
/// Returns an error string if:
/// * The current directory cannot be determined.
/// * The current directory is not a valid pRustIO project.
/// * The internal `clear_dir` utility or `cargo clean` fails to remove the directories  
///   due to permissions or file locks.
pub fn clean(json_output: &bool) -> Result<(), String> {
    // get project's path
    let proj_path = match env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            return Err("Failed to get current working directory.".to_string());
        },
    };
    if !utils::check_if_is_project_dir(&proj_path) {
        return Err("Not in project dir.".to_string());
    }

    // clean temporary files
    let target_dir = proj_path.join("target");
    let prio_dir = proj_path.join(utils::PROJECT_APP_DIR_NAME);
    
    if !*json_output { info("Cleaning PrustIO temporary files..."); }
    if prio_dir.exists() {
        utils::clear_dir(&prio_dir)?;
    }

    if !*json_output { info("Cleaning Cargo temporary files..."); }
    if target_dir.exists() {
        wrapper::cargo::cargo_clean(&proj_path)?;
    }

    Ok(())
}