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