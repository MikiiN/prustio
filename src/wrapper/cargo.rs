use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio, ExitStatus};

const CONFIGURATION_DIR_NAME: &str = ".cargo";
const CONFIGURATION_FILE_NAME: &str = "config.toml";

pub fn init_cargo(proj_path: &PathBuf) -> std::io::Result<ExitStatus> {
    let mut cmd = Command::new("cargo");
    cmd.arg("init").arg(proj_path);

    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    return status;
}

pub fn cargo_build(proj_path: &PathBuf) -> std::io::Result<ExitStatus> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    return status;
}