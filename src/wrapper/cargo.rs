//! Wrapper for the Cargo build system.
//!
//! This module provides functions to interface with the `cargo` CLI.
//! It handles initializing the base Rust project structure and 
//! triggering the compilation build process.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Initializes a new Cargo package in the specified directory.
///
/// This function calls `cargo init <path>` to generate the default Cargo structure 
/// (`src/main.rs`, `Cargo.toml`).
///
/// # Arguments
/// * `proj_path` - The absolute path where the project should be initialized.
///
/// # Errors
/// Returns an error string if the `cargo` command cannot be found on the host 
/// system, or if the initialization process fails.
pub fn init_cargo(proj_path: &PathBuf) -> Result<(), String> {
    let mut cmd = Command::new("cargo");
    cmd.arg("init").arg(proj_path);

    // execute command and propagate tool's STDOUT and STDERR to user outputs
    let output = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output();
    
    match output {
        Ok(output) => {
            if !output.status.success() {
                return Err("Tool cargo failed.".to_string());
            }
        },
        Err(_) => {
            return Err("Failed to run cargo tool.".to_string());
        }
    }
    Ok(())
}

/// Executes the Cargo build process for the embedded project.
///
/// This function executes `cargo build --release` in project directory. 
/// Because `.cargo/config.toml` dictates the target architecture and the custom 
/// build script (`build.rs`) links the libraries, Cargo automatically handles the 
/// compilation to AVR.

/// # Arguments
/// * `proj_path` - The root directory of the pRustIO project.
/// * `target` - An optional target architecture string.
/// * `show_output` - The flag for showing the output.
///
/// # Errors
/// Returns an error string if compilation fails due to syntax errors, missing 
/// dependencies, or linker failures.
pub fn cargo_build(proj_path: &PathBuf, target: &Option<String>, show_output: &bool) -> Result<(), String>  {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(proj_path)
        .args(["build", "--release"]);
    
    // add target if was given
    if let Some(t) = target {
        cmd.arg("--target").arg(t);
    }


    // execute command 
    let output = if *show_output {
        // propagate tool's STDOUT and STDERR to user outputs
        cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output() 
    } else {
        cmd.output()
    };
    
    // handle output status
    match output {
        Ok(output) => {
            if !output.status.success() {
                return Err("Cargo build failed.".to_string());
            }
        },
        Err(_) => {
            return Err("Failed to run cargo build.".to_string());
        }
    }
    Ok(())
}

/// Executes Cargo clean command in given directory.
/// 
/// # Arguments
/// * `proj_path` - The root directory of the pRustIO project.
/// * `show_output` - The flag for showing the output.
/// 
/// # Errors
/// Returns an error string if clean command fails.
pub fn cargo_clean(proj_path: &PathBuf, show_output: &bool) -> Result<(), String> {
    let mut cmd = Command::new("cargo");
    cmd.arg("clean").current_dir(proj_path);

    // execute command 
    let output = if *show_output {
        // propagate tool's STDOUT and STDERR to user outputs
        cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output() 
    } else {
        cmd.output()
    };
    
    match output {
        Ok(output) => {
            if !output.status.success() {
                return Err("Tool cargo failed.".to_string());
            }
        },
        Err(_) => {
            return Err("Failed to run cargo clean.".to_string());
        }
    }
    Ok(())
}