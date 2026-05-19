//! Controller for building and uploading the project.
//!
//! This module acts as the core build engine for pRustIO. When a user executes 
//! `prustio run`, this controller orchestrates the compilation pipeline:
//! 1. Resolves the active environment and target board.
//! 2. In hybrid mode, delegates the C/C++ framework compilation to PlatformIO.
//! 3. Generates the final Cargo configurations.
//! 4. Executes the `cargo build` process.
//! 5. Converts the resulting `.elf` binary into an `.hex` format.
//! 6. Optionally uploads the firmware to the board via `avrdude`.

use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

use crate::model::board::Board;
use crate::model::prustio_config::{Env, Package, };
use crate::model::{board, build, cargo_config_toml, cargo_toml, device, platformio_lock, prustio_config};
use crate::ui::display;
use crate::utils;
use crate::wrapper::{cargo, avr, avrdude, platformio};

const DEFAULT_ELF_BIN_NAME: &str = "bin.elf";
const DEFAULT_HEX_BIN_NAME: &str = "bin.hex"; 

/// Executes the build and/or upload processes for the current project.
///
/// This function determines the target(s) and environment, fetches the 
/// corresponding hardware specifications, and runs the build pipeline.
///
/// # Arguments
/// * `target` - Optional override for the target action (e.g., "build" or "upload"). 
/// * `environment` - Optional override for the environment to build.
/// * `json_output` - If `true`, suppresses standard console logs for JSON compatibility.
///
/// # Errors
/// Returns an error string if:
/// * Not running inside a valid pRustIO project.
/// * Configurations fail to load or parse.
/// * The build or upload tools encounter an error.
pub fn run(
    target: &Option<String>,
    environment: Option<&String>,
    json_output: &bool,
) -> Result<(), String> {
    // fetch project's dir
    let proj_path = match env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            return Err("Failed to get current working directory.".to_string());
        },
    };
    if !utils::check_if_is_project_dir(&proj_path) {
        return Err("Not in project dir.".to_string());
    }

    // fetch configuration
    let package = prustio_config::get_package_information(&proj_path)?;
    let env = prustio_config::get_env(&proj_path, environment)?;

    // get list of targets to run
    let targets = match target {
        Some(t) => {
            Vec::from([Target::from(t)?])
        },
        None => {
            match &env.targets {
                Some(ts) => {
                    let mut targets = Vec::new();
                    for target in ts {
                        targets.push(Target::from(target)?);
                    }
                    targets
                },
                None => {
                    Vec::from([Target::Upload])
                }
            }
        }
    };

    // prepare binary parameters
    let board = board::get_board(&env.board)?;
    let board_arch = board.platform.to_cargo_arch();

    let binary_path = get_binary_dir_path(&board_arch);
    let elf_bin_path = binary_path.join(DEFAULT_ELF_BIN_NAME);
    let hex_bin_path = binary_path.join(DEFAULT_HEX_BIN_NAME);

    let mut config = prustio_config::get_config(&proj_path)?;
    config.set_active_env(&env.name)?;
    config.save(&proj_path)?;

    let user_dependencies = config.get_user_defined_dependencies();

    // targets execution
    for t in targets {
        match t {
            Target::Build => {
                build_project(
                    &proj_path, &package, &board, &env, user_dependencies, 
                    &board_arch, &elf_bin_path, &hex_bin_path, json_output
                )?;
            },
            Target::Upload => {
                build_project(
                    &proj_path, &package, &board, &env, user_dependencies, 
                    &board_arch, &elf_bin_path, &hex_bin_path, json_output
                )?;
                upload_project(&board, &hex_bin_path, json_output)?;
            }
        }
    }


    Ok(())
}

/// Represents the high-level actions that can be taken on a project.
enum Target {
    Build,
    Upload,
}

impl Target {
    /// Parses a string into a `Target` variant.
    /// 
    /// # Arguments
    /// * `text` - The string of potential target. 
    /// 
    /// # Errors
    /// Returns an error, if target string doesn't match any supported target.
    fn from(text: &String) -> Result<Target, String> {
        let value = text.to_ascii_lowercase();
        match value.as_str() {
            "build" => Ok(Target::Build),
            "upload" => Ok(Target::Upload),
            _ => Err("Invalid target name.".to_string())
        }
    }
    
    /// Converts the `Target` variant into its string representation.
    fn _to_string(&self) -> String {
        match self {
            Target::Build => "build".to_string(),
            Target::Upload => "upload".to_string(),
        }
    }
}

/// Orchestrates the compilation pipeline.
///
/// If hybrid mode is active, this function triggers PlatformIO to compile the C++ framework
/// and prepares the `build.rs` script. It then updates Cargo settings and executes 
/// the final Rust build.
/// 
/// # Arguments
/// * `proj_path` - The path of the project.
/// * `package` - The project metadata.
/// * `board` - The target board parameters.
/// * `env` - Currently active environment data.
/// * `user_dependencies` - Dependencies defined by the user in `Prustio.toml`.
/// * `board_arch` - The board architecture.
/// * `elf_bin_path` - The path where the ELF binary file is located.
/// * `hex_bin_path` - The path where the final HEX binary file should be put.
/// * `json_output` - If `true`, suppresses standard console logs for JSON compatibility.
///
/// # Errors
/// Returns an error if any of the underlying build tools fail.
fn build_project(
    proj_path: &PathBuf,
    package: &Package,
    board: &Board,
    env: &Env,
    user_dependencies: Option<&BTreeMap<String, toml::Value>>,
    board_arch: &String,
    elf_bin_path: &PathBuf,
    hex_bin_path: &PathBuf,
    json_output: &bool,
) -> Result<(), String> {
    if !*json_output { display::info("Starting build process..."); }

    if !*json_output { display::info("Configuring cargo..."); }
    if package.hybrid_mode {
        // config project for the hybrid mode
        prepare_hybrid_mode_compilation(proj_path, board, env, json_output)?;
        // obtain linker path
        let linker = match avr::obtain_bin_path(avr::GCC_BINARY_NAME) {
            Ok(path) => match path.to_str() {
                Some(str_path) => str_path.to_string(),
                None => {
                    return Err("Failed to obtain avr-gcc binary path.".to_string());
                }
            },
            Err(msg) => {
                return Err(msg);
            }
        };
        cargo_config_toml::update_cargo_config(proj_path, &board_arch, &board.mcu, Some(&linker))?;
    } else {
        cargo_config_toml::update_cargo_config(proj_path, &board_arch, &board.mcu, None)?;
    }
    // recreate cargo.toml 
    cargo_toml::create_cargo_toml_config(
        proj_path, 
        &package.name, 
        &board.cargo_feature, 
        &package.hybrid_mode,
        user_dependencies,
    )?;

    if !*json_output { display::info("Building project..."); }
    let show_output = !*json_output;
    cargo::cargo_build(proj_path, &None, &show_output)?;

    if !*json_output { display::info("Converting ELF to HEX format..."); }
    avr::elf_to_hex(elf_bin_path, hex_bin_path)?;

    if !*json_output { 
        display::info("Build finished."); 
    }

    Ok(())
}

/// Automates the detection of connected hardware and flashes the compiled firmware.
///
/// # Arguments
/// * `board` - The structure representing the target board.
/// * `hex_bin_path` - The path of the HEX binary file.
/// * `json_output` - If `true`, suppresses standard console logs for JSON compatibility.
/// 
/// # Errors
/// Returns an error if no target board is connected or if `avrdude` fails.
fn upload_project(
    board: &Board,
    hex_bin_path: &PathBuf,
    json_output: &bool,
) -> Result<(), String> {
    if !*json_output { 
        display::info("Uploading binary file..."); 
    }

    if !*json_output { display::info("Detecting connected devices..."); }
    let device = match device::get_connected_device_list() {
        Ok(mut ports) => {
            match ports.pop() {
                Some(p) => p,
                None => {
                    return Err("No connected device to upload.".to_string());        
                }
            }
        },
        Err(e) => {
            return Err(e);
        }
    };

    if !*json_output { display::info(&format!("Uploading binary to {}...", device.port)); }
    avrdude::upload_binary(&hex_bin_path, &board.mcu, &board.upload_protocol, &device.port, &board.bus_speed)?;

    if !*json_output { 
        display::info("Successfully Uploaded."); 
    }
    Ok(())
}

/// Constructs the output directory path for the compiled binaries based on the architecture.
/// 
/// # Arguments
/// * `board_arch` - The architecture of the target board.
fn get_binary_dir_path(board_arch: &String) -> PathBuf {
    PathBuf::from("target").join(board_arch).join("release")
}

/// Prepares the C/C++ libraries and `build.rs` script for hybrid mode compilation.
///
/// This function initializes a hidden PlatformIO project, extracts required framework 
/// packages, compiles them into object/archive files, and generates the necessary 
/// linker instructions for Cargo.
///
/// # Arguments
/// * `proj_dir` - The path of the PrustIO project.
/// * `board` - The structure representing the target board.
/// * `env` - Currently active environment.
/// * `json_output` - If `true`, suppresses standard console logs for JSON compatibility.
/// 
/// # Errors
/// Returns an error if PlatformIO initialization, compilation, or dependency resolution fails.
fn prepare_hybrid_mode_compilation(
    proj_dir: &PathBuf,
    board: &Board,
    env: &Env,
    json_output: &bool,
) -> Result<(), String> {
    if !*json_output { display::info("Preparing PlatformIO project for hybrid mode..."); }

    let framework = match &env.framework {
        Some(f) => f.clone(),
        None => "arduino".to_string()
    };
    let lock_file = platformio_lock::get_pio_lock_path(proj_dir);
    if lock_file.exists() {
        let deps = platformio_lock::Lockfile::load(proj_dir)?;
        let platform_packages = deps.get_platform_packages();
        let libs_deps = deps.get_lib_deps();

        platformio::init_compilation_project(
            proj_dir, 
            &board.platform.to_string(), 
            &board.id, 
            &framework,
            Some(&platform_packages),
            if !libs_deps.is_empty() {
                Some(&libs_deps)
            } else {
                None
            }
        )?;

        if !*json_output { display::info("Compiling C/C++ libraries..."); }
        platformio::compile_c_libraries(proj_dir, &board.id)?;
    } else {
        platformio::init_compilation_project(
            proj_dir, 
            &board.platform.to_string(), 
            &board.id, 
            &framework,
            None,
            None
        )?;

        if !*json_output { display::info("Compiling C/C++ libraries..."); }
        platformio::compile_c_libraries(proj_dir, &board.id)?;

        let output = platformio::get_pio_project_dependencies(proj_dir)?;
        let deps = platformio_lock::parse_pio_list_output(&output)?;
        let lock = platformio_lock::Lockfile::new(deps, 1);
        lock.save(proj_dir)?;
    }

    if !*json_output { display::info("Writing Rust build scripts..."); }
    let lib_names = utils::get_compiled_libs_names(proj_dir);
    build::write_build_configuration(proj_dir, &lib_names)?;

    Ok(())
}