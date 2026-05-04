//! General utility functions for file system operations and path resolution.
//!
//! This module provides common helper functions used throughout `pRustIO` 
//! to manage directories, locate executables within virtual environments, 
//! and identify project boundaries.

use dirs;
use std::{fs, path::PathBuf};

pub const APP_DIR_NAME: &str = ".prustio";
pub const PROJECT_APP_DIR_NAME: &str = ".prio";

pub const PIO_COMPILATION_PROJECT_DIR_NAME: &str = "pio_workspace";
pub const COMPILED_LIBS_DIR_NAME: &str = "built_libs";

/// Ensures that a specific directory exists, creating it if necessary.
///
/// # Arguments
/// * `path` - The path to the directory to check or create.
///
/// # Errors
/// Returns an error if the directory does not exist and cannot be created.
pub fn ensure_dir_exists(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        match std::fs::create_dir(path) {
            Ok(_) => (),
            Err(_) => {
                let name = match path.as_os_str().to_str() {
                    Some(n) => n,
                    None => "invalid path",
                };
                let error_msg = format!("Can not create directory:\n{name}");
                return Err(error_msg);
            },
        }
    }
    Ok(())
}

/// Completely removes a directory and all of its contents.
///
/// Safely does nothing if the directory already does not exist.
///
/// # Arguments
/// * `path` - The path to the directory to remove.
///
/// # Errors
/// Returns an error if the directory exists but cannot be removed.
pub fn clear_dir(path: &PathBuf) -> Result<(), String> {
    if path.exists() {
        match std::fs::remove_dir_all(path) {
            Ok(_) => (),
            Err(_) => {
                return Err("Failed to remove directory".to_string());
            }
        }
    }
    Ok(())
}

/// Resolves the absolute path to an executable inside a Python virtual environment.
///
/// This handles the structural differences between Windows (`Scripts/`) and 
/// Unix (`bin/`) virtual environments.
///
/// # Arguments
/// * `venv_dir` - The path to the root of the virtual environment.
/// * `executable` - The name of the binary to find.
pub fn get_venv_executable(venv_dir: &PathBuf, executable: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    let bin_dir = venv_dir.join("Scripts");
    #[cfg(not(target_os = "windows"))]
    let bin_dir = venv_dir.join("bin");

    #[cfg(target_os = "windows")]
    let exe_name = format!("{}.exe", executable);
    #[cfg(not(target_os = "windows"))]
    let exe_name = executable.to_string();

    bin_dir.join(exe_name)
}

/// Checks if a specific executable exists within a given virtual environment.
pub fn check_venv_executable_existence(venv_dir: &PathBuf, executable: &str) -> bool {
    let exec = get_venv_executable(venv_dir, executable);
    exec.exists()
}

/// Retrieves the path to the global `pRustIO` application directory (`~/.prustio`).
///
/// # Errors
/// Returns an error if the host operating system's home directory cannot be found, 
/// or if the `.prustio` folder cannot be created.
pub fn get_app_dir() -> Result<PathBuf, String>  {
    let home_dir = match dirs::home_dir() {
        Some(p) => p,
        None => {
            return Err(String::from("Can not find the home directory."));
        }
    };
    let app_dir = home_dir.join(APP_DIR_NAME);

    ensure_dir_exists(&app_dir)?;
    Ok(app_dir)
}

/// Retrieves the path to the current project's local `.prio` application directory.
///
/// # Errors
/// Returns an error if the directory cannot be created.
pub fn get_project_app_dir(proj_path: &PathBuf) -> Result<PathBuf, String> {
    let local_app_dir = proj_path.join(PROJECT_APP_DIR_NAME);
    ensure_dir_exists(&local_app_dir)?;
    Ok(local_app_dir)
}

/// Checks if the given path contains a valid `pRustIO` project.
///
/// Currently, this checks for the presence of a `Prustio.toml` file.
pub fn check_if_is_project_dir(path: &PathBuf) -> bool {
    // TODO - do better checks
    let conf_file = path.join("Prustio.toml");
    conf_file.exists()
}

/// Checks if the given path contains a valid PlatformIO project.
///
/// Currently, this checks for the presence of a `platformio.ini` file.
pub fn check_if_is_pio_dir(path: &PathBuf) -> bool {
    let conf_file = path.join("platformio.ini");
    conf_file.exists()
}

/// Scans the `built_libs` directory and returns the names of all compiled static libraries.
///
/// This strips the `lib` prefix and `.a` extension to return names formatted 
/// specifically for the `rustc` linker. The custom C/C++ `Wrapper` library
/// is skipped because it is linked separately as an object file.
pub fn get_compiled_libs_names(proj_path: &PathBuf) -> Vec<String> {
    let libs_dir_path = proj_path.join(PROJECT_APP_DIR_NAME)
                                 .join(COMPILED_LIBS_DIR_NAME);

    let mut lib_names = Vec::new();
    if !libs_dir_path.exists() {
        return lib_names;
    }

    if let Ok(entries) = fs::read_dir(libs_dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "a") {
                if let Some(file_stem) = path.file_stem().and_then(|n| n.to_str()) { 
                    // skip already added wrapper
                    if file_stem.contains("Wrapper") {
                        continue;
                    }

                    // remove lib prefix
                    let link_name = if file_stem.starts_with("lib") {
                        &file_stem[3..]
                    } else {
                        file_stem
                    };
                    lib_names.push(link_name.to_string());
                }
            }
        }
    }

    lib_names
}

// 
// Unit Tests
//

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ensure_dir_exists() {
        let temp = tempdir().unwrap();
        let new_dir = temp.path().join("new_folder");
        
        // Directory shouldn't exist initially
        assert!(!new_dir.exists());
        
        // Function should create it
        assert!(ensure_dir_exists(&new_dir).is_ok());
        assert!(new_dir.exists());
        
        // Calling it again on an existing directory should not error
        assert!(ensure_dir_exists(&new_dir).is_ok());
    }

    #[test]
    fn test_clear_dir() {
        let temp = tempdir().unwrap();
        let dir_to_clear = temp.path().join("to_clear");
        std::fs::create_dir(&dir_to_clear).unwrap();
        
        // Add a dummy file inside the directory
        std::fs::write(dir_to_clear.join("dummy.txt"), "data").unwrap();

        assert!(clear_dir(&dir_to_clear).is_ok());
        
        // The directory and its contents should be gone
        assert!(!dir_to_clear.exists());
    }
}