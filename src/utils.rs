use dirs;
use std::path::PathBuf;

const APP_DIR_NAME: &str = ".prustio";

pub fn ensure_dir_existence(path: &PathBuf) -> Result<(), String> {
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

pub fn check_venv_executable_existance(venv_dir: &PathBuf, executable: &str) -> bool {
    let exec = get_venv_executable(venv_dir, executable);
    exec.exists()
}

pub fn get_app_dir() -> Result<PathBuf, String>  {
    let home_dir = match dirs::home_dir() {
        Some(p) => p,
        None => {
            return Err(String::from("Can not find the home directory."));
        }
    };
    let app_dir = home_dir.join(APP_DIR_NAME);

    ensure_dir_existence(&app_dir)?;
    return Ok(app_dir);
}

// TODO - do better checks
pub fn check_if_project_dir(path: &PathBuf) -> bool {
    let conf_file = path.join("Prustio.toml");
    conf_file.exists()
}

// --------------------------------
//  Parsing pio pkg list dependencies list
// --------------------------------

use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum Category {
    Platform,
    Framework,
    Tool,
    Library,
}

#[derive(Debug, PartialEq)]
pub struct LockedDependency {
    pub name: String,
    pub version: String,
    pub category: Category,
}

pub fn parse_pio_list_output(stdout: &str) -> Result<Vec<LockedDependency>, String> {
    let package_regex = match Regex::new(r"([a-zA-Z0-9\-_/]+)\s+@\s+([a-zA-Z0-9\.\-\+]+)") {
        Ok(regex) => regex,
        Err(_) => {
            return Err("Failed to parse regex expression".to_string());
        }
    };
    
    let mut locked_deps = Vec::new();
    let mut in_libraries_section = false;

    for line in stdout.lines() {
        // check if entered the libraries section
        if line.starts_with("Libraries") {
            in_libraries_section = true;
            continue;
        }

        // skip empty lines or headers
        if line.trim().is_empty() || !line.contains('@') {
            continue;
        }

        // extract the name and version
        if let Some(captures) = package_regex.captures(line) {
            let name = match captures.get(1) {
                Some(n) => n.as_str().to_string(),
                None => {
                    return Err("Internal error while getting name.".to_string());
                }
            };
            let version = match captures.get(2) {
                Some(v) => v.as_str().to_string(),
                None => {
                    return Err("Internal error while getting version.".to_string());
                }
            }; 

            // determine the category based on context and prefixes
            let category = if line.starts_with("Platform") {
                Category::Platform
            } else if name.starts_with("framework-") {
                Category::Framework
            } else if name.starts_with("tool-") || name.starts_with("toolchain-") {
                Category::Tool
            } else if in_libraries_section {
                Category::Library
            } else {
                // fallback
                Category::Library 
            };

            locked_deps.push(LockedDependency { name, version, category });
        }
    }

    Ok(locked_deps)
}