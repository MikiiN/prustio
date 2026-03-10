use std::path::PathBuf;
use std::process::{Command, Stdio, ExitStatus};

pub fn init_cargo(proj_path: &PathBuf) -> std::io::Result<ExitStatus> {
    let mut cmd = Command::new("cargo");
    cmd.arg("init").arg(proj_path);

    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    return status;
}

pub fn cargo_build(proj_path: &PathBuf, target: &Option<String>) -> std::io::Result<ExitStatus> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(proj_path)
        .args(["build", "--release"]);
    match target {
        Some(t) => {
            cmd.arg("--target").arg(t);
        },
        None => {}
    };

    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    return status;
}