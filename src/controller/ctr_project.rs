//! Controller for initiating new pRustIO projects.
//!
//! When a user runs the `prustio project init` command, this module handles 
//! the creation of the project structure. It orchestrates the generation of 
//! `Cargo.toml`, `.cargo/config.toml`, `rust-toolchain.toml`, `Prustio.toml`, 
//! and the initial `src/main.rs` template code.

use std::path::PathBuf;

use crate::model::{
    board, 
    cargo_config_toml,
    cargo_toml, 
    prustio_config, 
    toolchain_toml, 
    source_code
};
use crate::ui;
use crate::wrapper::cargo;

const DEFAULT_PROJECT_NAME: &str = "project";

/// Initializes a new pRustIO project in a new directory.
///
/// This function verifies that the target directory does not already exist, 
/// resolves the requested board configuration, and delegates the creation of 
/// Cargo and pRustIO configuration files to internal helpers.
///
/// # Arguments
/// * `name` - An optional custom name for the project and directory. Defaults to "project".
/// * `board_id` - An optional hardware board ID. If not provided, an "unspecified" placeholder is used.
/// * `hybrid` - If `true`, the project is initialized with PlatformIO C/C++ bindings.
/// * `json_output` - If `true`, suppresses standard console logs for JSON compatibility.
///
/// # Errors
/// Returns an error string if:
/// * A directory or file with the project name already exists.
/// * An invalid or unsupported `board_id` is provided.
/// * Writing project configuration fails.
pub fn init_project(
    name: &Option<String>, 
    board_id: &Option<String>, 
    hybrid: &bool, 
    json_output: &bool,
) -> Result<(), String> {
    let proj_name = match name {
        Some(n) => n,
        None => &String::from(DEFAULT_PROJECT_NAME),
    };
    let proj_path: PathBuf = PathBuf::from(&proj_name);

    if proj_path.exists() {
        return Err("The project or directory with same name already exists.".to_string());  
    }

    let board = match board_id {
        Some(id) => {
            board::get_board(id)?
        },
        None => board::get_unspecified_board()
    };

    let board_arch = board.platform.to_cargo_arch();

    if !*json_output {
        ui::display::info("Initiating cargo project...");
    }
    cargo_init(&proj_path, proj_name, &board_arch, &board.mcu, &board.cargo_feature, &board.rustc_version, hybrid)?;

    // TODO - hardcoded
    let framework = if *hybrid {
            Some(&String::from("arduino"))
        } else {
            None
        };
    
    prustio_init(&proj_path, &proj_name, hybrid, &board.id, framework)?;
    
    Ok(())
}

/// Sets up the standard Cargo environment and dependencies.
///
/// This internally calls the native `cargo init` command and then overrides 
/// or adds the necessary files (`.cargo/config.toml`, `Cargo.toml`, 
/// `rust-toolchain.toml`, and `src/main.rs`) to support AVR compilation.
///
/// # Errors
/// Returns an error if the internal native `cargo init` call fails or if 
/// generating any of the configuration files fails.
fn cargo_init(
    proj_path: &PathBuf,
    proj_name: &String, 
    board_arch: &String, 
    board_mcu: &String,
    cargo_feature: &String,
    rustc_version: &String,
    hybrid: &bool,
) -> Result<(), String> {
    cargo::init_cargo(proj_path)?;

    toolchain_toml::create_toolchain_config(proj_path, rustc_version, &None, &None)?;

    cargo_config_toml::create_cargo_config(proj_path, board_arch, board_mcu)?;

    cargo_toml::create_cargo_toml_config(proj_path, proj_name,cargo_feature, hybrid)?;

    source_code::write_example_code(proj_path, hybrid)?;

    Ok(())
}

/// Sets up the custom `Prustio.toml` configuration file.
///
/// # Errors
/// Returns an error if writing the `Prustio.toml` file to disk fails.
fn prustio_init(
    proj_path: &PathBuf, 
    proj_name: &String, 
    hybrid: &bool,
    board_id: &String,
    framework: Option<&String>,
) -> Result<(), String> {
    prustio_config::create_prustio_config(proj_path, proj_name, hybrid, board_id, framework)?;
    Ok(())
}