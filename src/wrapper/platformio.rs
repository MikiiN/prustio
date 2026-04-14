use std::fs::{self, ReadDir};
use std::io::Error;
use std::path::PathBuf;
use std::process::{Command, Output};
use regex::Regex;

use crate::cpp_templates::{arduino_wrapper_cpp, arduino_wrapper_h};
use crate::model::platformio_ini;
use crate::utils::{
    check_if_is_pio_dir, 
    check_venv_executable_existence, 
    clear_dir, 
    ensure_dir_exists, 
    get_app_dir, 
    get_project_app_dir, 
    get_venv_executable,
    PIO_COMPILATION_PROJECT_DIR_NAME,
    COMPILED_LIBS_DIR_NAME,
};
use crate::wrapper::avr;

const PIO_VENV_DIR_NAME: &str = "pio_venv";
const PIO_CORE_DIR_NAME: &str = "pio_core";
const PIO_SRC_DIR_NAME: &str = "src";
const PIO_PROJECT_APP_DIR_NAME: &str = ".pio";

pub fn check_pio_installation() -> bool {
    let venv_dir = match get_pio_dirs() {
        Ok((venv, _)) => venv,
        Err(_) => {return false;}
    }; 
    check_venv_executable_existence(&venv_dir, "pio")
}

pub fn get_boards(filter: &str) -> std::io::Result<Output> {
    let mut cmd = Command::new("pio");
    return cmd.args(["boards", filter, "--json-output"]).output();
}

pub fn get_pio_dirs() -> Result<(PathBuf, PathBuf), String> {
    let app_dir = get_app_dir()?;

    let pio_venv_dir = app_dir.join(PIO_VENV_DIR_NAME);
    ensure_dir_exists(&pio_venv_dir)?;

    let pio_core_dir = app_dir.join(PIO_CORE_DIR_NAME);
    ensure_dir_exists(&pio_core_dir)?;

    return Ok((pio_venv_dir, pio_core_dir));
}

// TODO check for python existence
pub fn setup_platformio() -> Result<(), String> {
    let (venv_dir, _ ) = get_pio_dirs()?; 
    let venv_status = std::process::Command::new("python3")
        .args(["-m", "venv"])
        .arg(&venv_dir)
        .status()
        .expect("Failed to execute python3. Is Python installed?");

    if !venv_status.success() {
        return Err(String::from("Failed to create python virtual environment."));
    }

    let pip_path = get_venv_executable(&venv_dir, "pip");

    let pip_status = std::process::Command::new(pip_path)
        .args(["install", "-U", "platformio"])
        .status()
        .expect("Failed to execute pip install.");

    if !pip_status.success() {
        return Err(String::from("Failed to install the platformIO."));
    }
    Ok(())
}

pub fn download_pio_toolchain(toolchain_name: &str) -> Result<(), String> {
    let (venv_dir, core_dir) = get_pio_dirs()?;
    let args = [
        "pkg", 
        "install", 
        "--global", 
        "--tool", 
        toolchain_name,
    ];
    // TODO check output.status
    match run_pio_command(&venv_dir, &core_dir, &args, None) {
        Ok(_) => {},
        Err(_) => {
            return Err(String::from("PlatformIO failed to install toolchain."));
        } 
    };
    
    Ok(())
}

pub fn download_pio_platform(platform_name: &str) -> Result<(), String> {
    let (venv_dir, core_dir) = get_pio_dirs()?;
    let args = [
        "pkg", 
        "install", 
        "--global", 
        "--platform", 
        platform_name,
    ];
    // TODO check output.status
    match run_pio_command(&venv_dir, &core_dir, &args, None) {
        Ok(_) => {},
        Err(_) => {
            return Err(String::from("PlatformIO failed to install toolchain."));
        } 
    };
    
    Ok(())
}

pub fn get_devices() -> Result<Vec<u8>, String> {
    let (venv_dir, core_dir) = get_pio_dirs()?;
    let args = [
        "device", "list", "--json-output"
    ];
    let output = match run_pio_command(&venv_dir, &core_dir, &args, None) {
        Ok(o) => o,
        Err(_) => {
            return Err(String::from("PlatformIO failed to get device list."));
        } 
    };

    if !output.status.success() {
        return Err(String::from("PlatformIO failed to execute command"));
    }

    Ok(output.stdout)
}

pub fn init_compilation_project(
    project_dir: &PathBuf,
    platform: &String,
    board_id: &String,
    framework: &String,
) ->Result<(), String> {
    let app_dir = get_project_app_dir(project_dir)?;
    let pio_proj = app_dir.join(PIO_COMPILATION_PROJECT_DIR_NAME);
    
    // Ensure data from old build will not interfere with current build
    clear_dir(&pio_proj)?;

    ensure_dir_exists(&pio_proj)?;
    let pio_path_str = match pio_proj.to_str() {
        Some(path) => path,
        None => {
            return Err("Can not parse PlatformIO compile project path to string".to_string());
        }
    };

    let (venv_dir, core_dir) = get_pio_dirs()?;
    let pio_args = [
        "project", "init", "-d", pio_path_str
    ];
    let output = match run_pio_command(&venv_dir, &core_dir, &pio_args, None) {
        Ok(o) => o,
        Err(_) => {
            return Err("Failed to init PlatformIO project".to_string());
        } 
    };

     if !output.status.success() {
        return Err("PlatformIO failed to execute init command".to_string());
    }

    platformio_ini::rewrite_pio_config(&pio_proj, platform, board_id, framework)?;

    let pio_src = pio_proj.join(PIO_SRC_DIR_NAME);
    ensure_dir_exists(&pio_src)?;

    let header_file = pio_src.join(arduino_wrapper_h::NAME);
    match std::fs::write(header_file, arduino_wrapper_h::CONTENT) {
        Ok(_) => (),
        Err(_) => {
            return Err("Failed to write wrapper.h content.".to_string()); 
        }
    };

    let cpp_file = pio_src.join(arduino_wrapper_cpp::NAME);
    match std::fs::write(cpp_file, arduino_wrapper_cpp::CONTENT) {
        Ok(_) => (),
        Err(_) => {
            return Err("Failed to write wrapper.cpp content.".to_string()); 
        }
    };

    Ok(())
}

pub fn compile_c_libraries(project_dir: &PathBuf, board_id: &String) -> Result<(), String> {
    let app_dir = get_project_app_dir(project_dir)?;
    let pio_proj = app_dir.join(PIO_COMPILATION_PROJECT_DIR_NAME);

    if !check_if_is_pio_dir(&pio_proj) {
        return Err("Missing PlatformIO project.".to_string());
    }

    let (venv_dir, core_dir) = get_pio_dirs()?;
    let pio_args = [
        "run"
    ];
    let output = match run_pio_command(&venv_dir, &core_dir, &pio_args, Some(&pio_proj)) {
        Ok(o) => o,
        Err(_) => {
            return Err("Failed to compile PlatformIO project".to_string());
        } 
    };

    if !output.status.success() {
        return Err("PlatformIO failed to execute run command".to_string());
    }

    let compiled_dir = app_dir.join(COMPILED_LIBS_DIR_NAME);
    clear_dir(&compiled_dir)?;
    ensure_dir_exists(&compiled_dir)?;

    let pio_build_dir = pio_proj.join(PIO_PROJECT_APP_DIR_NAME)
                                .join("build")
                                .join(board_id);
    if !pio_build_dir.exists() {
        return Err("Missing pio build directory.".to_string());
    }

    // archive the wrapper
    // let wrapper_obj = pio_build_dir.join("src").join("wrapper.cpp.o");
    // // let wrapper_lib = compiled_dir.join("libWrapper.a");
    // // avr::archive_object_file(&wrapper_obj, &wrapper_lib)?;

    let wrapper_dir = pio_build_dir.join("src");
    copy_lib_to_dir("wrapper.cpp.o", &wrapper_dir, &compiled_dir)?;

    copy_lib_to_dir("libFrameworkArduino.a", &pio_build_dir, &compiled_dir)?;
    copy_lib_to_dir("libFrameworkArduinoVariant.a", &pio_build_dir, &compiled_dir)?;

    let libs = search_libs_in_pio_build(&pio_build_dir)?;

    for (lib_name, lib_dir) in libs {
        copy_lib_to_dir(&lib_name, &lib_dir, &compiled_dir)?;
    }

    Ok(())    
}

// TODO - made to search for FrameworkArduino too
fn search_libs_in_pio_build(build_dir: &PathBuf) -> Result<Vec<(String, PathBuf)>, String> {
    let entries = get_dir_entries(build_dir)?;

    let mut lib_paths: Vec<(String, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        
        let potential_lib_dir = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => {
                continue;
            }
        };
        if path.is_dir() && potential_lib_dir.starts_with("lib") {
            let sub_entries = get_dir_entries(&path)?;
            for sub_entry in sub_entries.flatten() {
                let file_path = sub_entry.path();
                if file_path.extension().map_or(false, |ext| ext == "a") {
                    let lib_name = match file_path.file_name() {
                        Some(s) => s.to_string_lossy().to_string(),
                        None => {
                            return Err("Failed to get library name.".to_string());
                        } 
                    };

                    lib_paths.push((lib_name, path.clone()));
                }
            }
        }
    }
    Ok(lib_paths)
}

fn get_dir_entries(dir: &PathBuf) -> Result<ReadDir, String> {
    match fs::read_dir(dir) {
        Ok(es) => Ok(es),
        Err(_) => Err("PlatformIO build dir does not contain any libraries.".to_string())
    }
}

fn copy_lib_to_dir(lib_name: &str, src_dir: &PathBuf, dest_dir: &PathBuf) -> Result<(), String> {
    let lib = src_dir.join(lib_name);
    if !lib.exists() {
        return Err("Tried to copy non-existing library.".to_string());
    }

    if !dest_dir.exists() {
        return Err("Non-existing destination dir for library.".to_string());
    }
    let dest_lib = dest_dir.join(lib_name);

    match fs::copy(&lib, &dest_lib) {
        Ok(_) => {},
        Err(_) => {
            return Err("Failed to copy library.".to_string());
        }
    };

    Ok(())
}

fn run_pio_command(venv_dir: &PathBuf, core_dir: &PathBuf, pio_args: &[&str], run_dir: Option<&PathBuf>) -> Result<Output, Error> {
    let pio_path = get_venv_executable(venv_dir, "pio");
    match run_dir {
        Some(dir) => {
            Command::new(pio_path)
                .env("PLATFORMIO_CORE_DIR", core_dir) 
                .args(pio_args)
                .current_dir(dir)
                .output()
        },
        None => {
            Command::new(pio_path)
                .env("PLATFORMIO_CORE_DIR", core_dir) 
                .args(pio_args)
                .output()
        }
    }
}

// --------------------------------
//  Parsing pio pkg list dependencies list
// --------------------------------

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