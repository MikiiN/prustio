use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio, ExitStatus};

const CONFIGURATION_DIR_NAME: &str = ".cargo";
const CONFIGURATION_FILE_NAME: &str = "config.toml";

pub fn init_cargo(proj_path: &String) -> std::io::Result<ExitStatus> {
    let mut cmd = Command::new("cargo");
    cmd.arg("init").arg(proj_path);

    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    return status;
}

pub fn create_cargo_config(
    proj_path: &String,
    target_architecture: &String,
    target_mcu: &String,
) -> std::io::Result<()> {
    let dir_path: PathBuf = PathBuf::from(proj_path).join(CONFIGURATION_DIR_NAME);
    fs::create_dir_all(&dir_path)?;

    let file_path = dir_path.join(CONFIGURATION_FILE_NAME);
    let mut file = fs::File::create(file_path)?; 

    let content = get_config_toml_content(target_architecture, target_mcu);
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn get_config_toml_content(
    target_architecture: &String,
    target_mcu: &String,
) -> String {
    let arch = target_architecture.to_ascii_lowercase();
    let mcu = target_mcu.to_ascii_lowercase();
    format!("[build]
target = \"{arch}\"
rustflags = [\"-C\", \"target-cpu={mcu}\"]

[unstable]
build-std = [\"core\"]
")
}