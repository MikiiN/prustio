use std::process::{Command, Output};

pub fn get_boards(filter: &str) -> std::io::Result<Output> {
    let mut cmd = Command::new("pio");
    return cmd.args(["boards", filter, "--json-output"]).output();
}