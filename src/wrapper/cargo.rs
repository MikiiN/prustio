use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn init_cargo(proj_path: &PathBuf) -> Result<(), String> {
    let mut cmd = Command::new("cargo");
    cmd.arg("init").arg(proj_path);

    let output = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output();
    
    match output {
        Ok(output) => {
            if !output.status.success() {
                return Err(String::from("Tool cargo failed."));
            }
        },
        Err(_) => {
            return Err(String::from("Failed to run cargo tool."));
        }
    }
    Ok(())
}

pub fn cargo_build(proj_path: &PathBuf, target: &Option<String>) -> Result<(), String>  {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(proj_path)
        .args(["build", "--release"]);
    match target {
        Some(t) => {
            cmd.arg("--target").arg(t);
        },
        None => {}
    };

    let output = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output();
    
     match output {
        Ok(output) => {
            if !output.status.success() {
                return Err("Tool avr-objcopy failed.".to_string());
            }
        },
        Err(_) => {
            return Err("Failed to run avr-objcopy tool.".to_string());
        }
    }
    Ok(())
}

pub fn cargo_clean(proj_path: &PathBuf) -> Result<(), String> {
    let mut cmd = Command::new("cargo");
    cmd.arg("clean").current_dir(proj_path);

    let output = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output();
    
    match output {
        Ok(output) => {
            if !output.status.success() {
                return Err(String::from("Tool cargo failed."));
            }
        },
        Err(_) => {
            return Err(String::from("Failed to run cargo clean."));
        }
    }
    Ok(())
}