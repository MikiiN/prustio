use dirs;
use std::path::PathBuf;

use crate::utils::ensure_dir_existence;

const APP_DIR_NAME: &str = ".prustio";
const PIO_VENV_DIR_NAME: &str = "pio_venv";
const PIO_CORE_DIR_NAME: &str = "pio_core";

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

fn get_venv_executable(venv_dir: &PathBuf, executable_name: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    let bin_dir = venv_dir.join("Scripts");
    #[cfg(not(target_os = "windows"))]
    let bin_dir = venv_dir.join("bin");

    #[cfg(target_os = "windows")]
    let exe_name = format!("{}.exe", executable_name);
    #[cfg(not(target_os = "windows"))]
    let exe_name = executable_name.to_string();

    bin_dir.join(exe_name)
}