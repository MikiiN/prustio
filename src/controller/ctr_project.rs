use std::path::PathBuf;

use crate::wrapper::cargo;
use crate::model::boards::{self, Board};

const DEFAULT_PROJECT_NAME: &str = "project";

pub fn init_project(
    name: &Option<String>, 
    board_id: &Option<String>, 
    hybrid: &bool, 
    json_output: &bool,
) {
    let path: PathBuf = match name {
        Some(n) => PathBuf::from(n),
        None => PathBuf::from(DEFAULT_PROJECT_NAME),
    };

    if path.exists() {
        eprintln!("Error: The project already exists.");
        return;
    }

    let project: String = match path.into_os_string().into_string() {
        Ok(p) => p,
        Err(os_str) => {
            eprintln!("Error: Invalid working directory path:\n{:?}", os_str);
            return;
        },
    };
    let result = cargo::init_cargo(&project);
    match result {
        Ok(status) => {
            if status.success() {
                println!("Cargo project created.");
            } else {
                eprintln!("Error: Cargo init failed with code: {}", status);
            }
        },
        Err(e) => {
            eprintln!("Error: Failed to execute the cargo command:\n {}",e);
            return;
        }
    }

    let (b_arch, b_mcu): (Option<String>, Option<String>) = match board_id {
        Some(id) => {
            match boards::get_board(id) {
                Ok(b) => {
                    let arch = match b.get_architecture() {
                        Some(a) => a,
                        None => {
                            eprintln!("Error: Unsupported board");
                            return;
                        },
                    };
                    // TODO - check errors
                    let _ = cargo::create_cargo_config(&project, &arch, &b.mcu);
                    (Some(arch), Some(b.mcu))
                },
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            }
        },
        None => (None, None),
    };
}