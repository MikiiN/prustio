use std::path::PathBuf;
use std::process::{Command, Output};

use crate::utils::{
    ensure_dir_existence,
    get_app_dir,
    get_venv_executable,
    check_venv_executable_existance,
};

const PIO_VENV_DIR_NAME: &str = "pio_venv";
const PIO_CORE_DIR_NAME: &str = "pio_core";

pub fn check_pio_installation() -> bool {
    let venv_dir = match get_pio_dirs() {
        Ok((venv, _)) => venv,
        Err(_) => {return false;}
    }; 
    check_venv_executable_existance(&venv_dir, "pio")
}

pub fn get_boards(filter: &str) -> std::io::Result<Output> {
    let mut cmd = Command::new("pio");
    return cmd.args(["boards", filter, "--json-output"]).output();
}

pub fn get_pio_dirs() -> Result<(PathBuf, PathBuf), String> {
    let app_dir = get_app_dir()?;

    let pio_venv_dir = app_dir.join(PIO_VENV_DIR_NAME);
    ensure_dir_existence(&pio_venv_dir)?;

    let pio_core_dir = app_dir.join(PIO_CORE_DIR_NAME);
    ensure_dir_existence(&pio_core_dir)?;

    return Ok((pio_venv_dir, pio_core_dir));
}

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

fn run_pio_command(venv_dir: &PathBuf, core_dir: &PathBuf, pio_args: &[&str]) {
    let pio_path = get_venv_executable(venv_dir, "pio");

    let status = Command::new(pio_path)
        .env("PLATFORMIO_CORE_DIR", core_dir) 
        .args(pio_args)
        .status()
        .expect("Failed to execute pio command.");

    if !status.success() {
        eprintln!("PlatformIO command failed.");
    }
}