//! Wrapper for the PlatformIO Core CLI.
//!
//! This module acts as the bridge between pRustIO and the PlatformIO ecosystem. 
//! It is responsible for setting up an isolated Python virtual environment, 
//! installing the PlatformIO CLI, querying hardware data, and most importantly, 
//! managing the hidden `pio_workspace` where the Arduino C++ framework is 
//! compiled into static libraries (`.a` files) for Cargo to link against.

use std::fs::{self, ReadDir};
use std::io::Error;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Output, Stdio};

use crate::cpp_templates::{arduino_wrapper_cpp, arduino_wrapper_h};
use crate::model::platformio_ini;
use crate::ui::device::{EOL, Parity};
use crate::utils::{
    self, COMPILED_LIBS_DIR_NAME, PIO_COMPILATION_PROJECT_DIR_NAME, check_if_is_pio_dir, check_venv_executable_existence, clear_dir, ensure_dir_exists, get_app_dir, get_project_app_dir, get_venv_executable
};

const PIO_VENV_DIR_NAME: &str = "pio_venv";
const PIO_CORE_DIR_NAME: &str = "pio_core";
const PIO_SRC_DIR_NAME: &str = "src";
const PIO_PROJECT_APP_DIR_NAME: &str = ".pio";

/// Checks if PlatformIO is already installed in the local virtual environment.
///
/// Returns `true` if the `pio` executable exists within the managed `pio_venv` 
/// directory, otherwise `false`.
pub fn check_pio_installation() -> bool {
    let venv_dir = match get_pio_dirs() {
        Ok((venv, _)) => venv,
        Err(_) => {return false;}
    }; 
    check_venv_executable_existence(&venv_dir, "pio")
}

/// Queries PlatformIO for a list of supported boards.
///
/// # Arguments
/// * `filter` - A string to filter the board results by ID or name.
/// 
/// # Errors
/// Returns an error, if command execution or parsing to string fails.
pub fn get_boards(filter: &str) -> Result<String, String> {
    let (venv_dir, core_dir) = get_pio_dirs()?;
    let args = [
        "boards", filter, "--json-output"
    ];

    let output = run_pio_command(&venv_dir, &core_dir, &args, None);
    let boards = check_command_output(
        output, 
        "PlatformIO failed to find the specified board",
        "Tool PlatformIO failed with error:", 
    )?;
    
    match String::from_utf8(boards) {
        Ok(out_str) => {
            return Ok(out_str.to_string());
        },
        Err(_) => {
            return Err("Failed to parse board output form PlatformIO.".to_string());
        }
    }
}

/// Retrieves the paths to the PlatformIO virtual environment and core directories.
///
/// # Errors
/// Returns an error if the home directory cannot be determined or created.
pub fn get_pio_dirs() -> Result<(PathBuf, PathBuf), String> {
    // get global application pio dir (in home directory)
    let app_dir = get_app_dir()?;

    let pio_venv_dir = app_dir.join(PIO_VENV_DIR_NAME);
    ensure_dir_exists(&pio_venv_dir)?;

    let pio_core_dir = app_dir.join(PIO_CORE_DIR_NAME);
    ensure_dir_exists(&pio_core_dir)?;

    return Ok((pio_venv_dir, pio_core_dir));
}

/// Sets up the PlatformIO environment.
///
/// Creates a new Python 3 virtual environment and uses `pip` to install the 
/// `platformio` package. This ensures pRustIO has a reliable, isolated instance 
/// of PlatformIO regardless of the user's global system configuration.
///
/// # Errors
/// Returns an error if `python3` or `pip` are not available, or if the installation fails.
pub fn setup_platformio() -> Result<(), String> {
    if !utils::is_python_installed() {
        return Err("Missing required dependency: Python".to_string());
    }

    // create a new virtual environment in home directory
    let (venv_dir, _ ) = get_pio_dirs()?; 
    #[cfg(target_os = "windows")]
    let python_cmd = "python";
    #[cfg(not(target_os = "windows"))]
    let python_cmd = "python3";
    let venv_output = Command::new(python_cmd)
        .args(["-m", "venv"])
        .arg(&venv_dir)
        .output();

    match venv_output {
        Ok(output) => {
            if !output.status.success() {
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to create python virtual environment with error:\n{}", stderr_str));
            }
        },
        Err(_) => {
            return Err("Failed execute the command for creating a new virtual environment.".to_string());
        }
    }

    // install PlatformIO to the virtual environment
    let pip_path = get_venv_executable(&venv_dir, "pip");
    let pip_output = Command::new(pip_path)
        .args(["install", "-U", "platformio"])
        .output();

    match pip_output {
        Ok(output) => {
            if !output.status.success() {
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to install PlatformIO in virtual environment with error:\n{}", stderr_str));
            }
        },
        Err(_) => {
            return Err("Failed execute the command for installing platformIO.".to_string());
        }
    }

    Ok(())
}

/// Downloads a specific toolchain package via PlatformIO.
///
/// # Arguments
/// * `toolchain_name` - The name of the toolchain package.
/// 
/// # Errors
/// Returns an error when it fails to download the toolchain.
pub fn download_pio_toolchain(toolchain_name: &str) -> Result<(), String> {
    let (venv_dir, core_dir) = get_pio_dirs()?;
    let args = [
        "pkg", 
        "install", 
        "--global", 
        "--tool", 
        toolchain_name,
    ];

    let output = run_pio_command(&venv_dir, &core_dir, &args, None);
    check_command_output(
        output, 
        "PlatformIO failed to install the toolchain with error:", 
        "PlatformIO failed to install the toolchain.",
    )?;
    
    Ok(())
}

/// Downloads a specific hardware platform package via PlatformIO.
///
/// # Arguments
/// * `platform_name` - The name of the platform package.
/// 
/// # Errors
/// Returns an error when it fails to download the platform.
pub fn download_pio_platform(platform_name: &str) -> Result<(), String> {
    let (venv_dir, core_dir) = get_pio_dirs()?;
    let args = [
        "pkg", 
        "install", 
        "--global", 
        "--platform", 
        platform_name,
    ];

    let output = run_pio_command(&venv_dir, &core_dir, &args, None);
    check_command_output(
        output, 
        "PlatformIO failed to install the platform with error:", 
        "PlatformIO failed to install the platform.",
    )?;
    
    Ok(())
}

/// Retrieves the raw JSON list of connected hardware devices.
/// 
/// # Errors
/// Returns an error when it fails to run PlatformIO command.
pub fn get_devices() -> Result<Vec<u8>, String> {
    let (venv_dir, core_dir) = get_pio_dirs()?;
    let args = [
        "device", "list", "--json-output"
    ];

    let output = run_pio_command(&venv_dir, &core_dir, &args, None);
    check_command_output(
        output, 
        "PlatformIO failed to get device list with error:", 
        "PlatformIO failed to get device.",
    )
}

/// Initializes the PlatformIO workspace for compiling C/C++ dependencies.
///
/// This function wipes any existing workspace, initializes a new `platformio.ini`, 
/// and automatically injects the `wrapper.cpp` and `wrapper.h` templates into 
/// the `src/` directory. These wrappers expose the Arduino framework functions 
/// to the Rust application via FFI.
/// 
/// # Arguments
/// * `project_dir` - The project location. 
/// * `platform` - The platform identifier.
/// * `board_id` - The board identifier.
/// * `framework` - Used PlatformIO framework.
/// * `platform_packages` - The platform's packages.
/// * `lib_deps` - The PlatformIO's project dependencies. 
///
/// # Errors
/// Returns an error if the workspace directory cannot be managed, PlatformIO fails 
/// to initialize, or the wrapper files cannot be written.
pub fn init_compilation_project(
    project_dir: &PathBuf,
    platform: &String,
    board_id: &String,
    framework: &String,
    platform_packages: Option<&Vec<String>>,
    lib_deps: Option<&Vec<String>>,
) ->Result<(), String> {
    let app_dir = get_project_app_dir(project_dir)?;
    let pio_proj = app_dir.join(PIO_COMPILATION_PROJECT_DIR_NAME);
    
    // ensure data from old build will not interfere with current build
    clear_dir(&pio_proj)?;

    ensure_dir_exists(&pio_proj)?;
    let pio_path_str = match pio_proj.to_str() {
        Some(path) => path,
        None => {
            return Err("Can not parse PlatformIO compile project path to string".to_string());
        }
    };

    // init pio project
    let (venv_dir, core_dir) = get_pio_dirs()?;
    let pio_args = [
        "project", "init", "-d", pio_path_str
    ];
    let cmd_output = run_pio_command(&venv_dir, &core_dir, &pio_args, None);
    check_command_output(
        cmd_output, 
        "Failed to init the PlatformIO project with error:", 
        "PlatformIO failed to execute init command.",
    )?;

    platformio_ini::rewrite_pio_config(&pio_proj, platform, board_id, framework, platform_packages, lib_deps)?;

    // add wrapper header file to the src directory
    let pio_src = pio_proj.join(PIO_SRC_DIR_NAME);
    ensure_dir_exists(&pio_src)?;
    let header_file = pio_src.join(arduino_wrapper_h::NAME);
    if let Err(_) = std::fs::write(header_file, arduino_wrapper_h::CONTENT) {
        return Err("Failed to write wrapper.h content.".to_string()); 
    }
    
    // add wrapper source files to the src directory
    let cpp_file = pio_src.join(arduino_wrapper_cpp::NAME);
    if let Err(_) = std::fs::write(cpp_file, arduino_wrapper_cpp::CONTENT) {
        return Err("Failed to write wrapper.cpp content.".to_string()); 
    }

    Ok(())
}

/// Executes the PlatformIO interactive serial monitor.
///
/// Passes standard input/output directly to the terminal, allowing the user to 
/// interact with their running hardware.
/// 
/// # Arguments
/// * `project_dir` - The project path.
/// * other arguments - Parameters for `pio device monitor` command.
/// 
/// # Errors
/// Returns error, if `pio device monitor` command fails.  
pub fn device_monitor(
    project_dir: &PathBuf,
    port: &Option<String>, 
    baud: &Option<u32>, 
    parity: &Option<Parity>, 
    rtscts: &bool, 
    xonxoff: &bool, 
    rts: &Option<u8>, 
    dtr: &Option<u8>, 
    echo: &bool, 
    encoding: &Option<String>, 
    filter: &Option<String>, 
    eol: &Option<EOL>, 
    raw: &bool, 
    exit_char: &Option<u8>, 
    menu_char: &Option<u8>, 
    quiet: &bool, 
    no_reconnect: &bool,
) -> Result<(), String> {
    let (venv_dir, core_dir) = get_pio_dirs()?;
    let mut pio_args = Vec::from(["device", "monitor"]);
    
    if let Some(p) = port {
        pio_args.push("--port");
        pio_args.push(p.as_str());
    }

    let baud_string: String;
    if let Some(b) = baud {
        pio_args.push("--baud");
        baud_string = b.to_string();
        pio_args.push(baud_string.as_str());
    }

    let parity_string: String;
    if let Some(p) = parity {
        parity_string = p.to_string(); 
        pio_args.push("--parity");
        pio_args.push(parity_string.as_str());
    }

    if *rtscts {
        pio_args.push("--rtscts");
    }

    if *xonxoff {
        pio_args.push("--xonxoff");
    }

    let rts_string: String;
    if let Some(value) = rts {
        rts_string = value.to_string();
        pio_args.push("--rts");
        pio_args.push(&rts_string);
    }

    let dtr_string: String;
    if let Some(value) = dtr {
        dtr_string = value.to_string();
        pio_args.push("--dtr");
        pio_args.push(&dtr_string);
    }

    if *echo {
        pio_args.push("--echo");
    }

    if let Some(value) = encoding {
        pio_args.push("--encoding");
        pio_args.push(value.as_str());
    }

    if let Some(value) = filter {
        pio_args.push("--filter");
        pio_args.push(value.as_str());
    }


    let eol_string: String;
    if let Some(value) = eol {
        eol_string = value.to_string();
        pio_args.push("--eol");
        pio_args.push(eol_string.as_str());
    }

    if *raw {
        pio_args.push("--raw");
    }

    let exit_string: String;
    if let Some(value) = exit_char {
        exit_string = value.to_string();
        pio_args.push("--exit-char");
        pio_args.push(&exit_string);
    }

    let menu_string: String;
    if let Some(value) = menu_char {
        menu_string = value.to_string();
        pio_args.push("--menu-char");
        pio_args.push(&menu_string);
    }

    if *quiet {
        pio_args.push("--quiet");
    }

    if *no_reconnect {
        pio_args.push("--no_reconnect");
    }

    let status = run_pio_command_with_output(&venv_dir, &core_dir, &pio_args, Some(project_dir))?;
    if !status.success() {
        return Err("PlatformIO monitor exited".to_string());
    }
    Ok(())
}

/// Executes `pio pkg list` to retrieve all resolved dependencies for the lockfile.
/// 
/// # Arguments
/// * `project_dir` - The path of the project.
/// 
/// # Errors
/// Returns an error when the PlatformIO project is invalid, or it fails to run the command. 
pub fn get_pio_project_dependencies(project_dir: &PathBuf) -> Result<String, String> {
    let app_dir = get_project_app_dir(project_dir)?;
    let pio_proj = app_dir.join(PIO_COMPILATION_PROJECT_DIR_NAME);

    if !check_if_is_pio_dir(&pio_proj) {
        return Err("Missing PlatformIO project to extract dependencies from.".to_string());
    }

    let (venv_dir, core_dir) = get_pio_dirs()?;
    let pio_args = [
        "pkg", "list"
    ];
    let output = run_pio_command(&venv_dir, &core_dir, &pio_args, Some(&pio_proj));
    let stdout = check_command_output(
        output, 
        "The PlatformIO command failed with error:", 
        "Failed to execute PlatformIO pkg list command."
    )?;

    match String::from_utf8(stdout) {
        Ok(out) => Ok(out),
        Err(_) => Err("Failed to parse PlatformIO pkg list output as UTF-8".to_string()),
    }
}

/// Orchestrates the C/C++ framework compilation and library extraction.
///
/// Executes `pio run` inside the workspace. Once PlatformIO finishes 
/// compiling the Arduino framework, this function searches the the `.pio/build` directory,
/// locates the resulting `.a` static libraries, and copies them to the central  
/// `built_libs` directory so Cargo can easily link them later.
/// 
/// # Arguments
/// * `project_dir` - The project directory.
/// * `board_id` - The board's identifier.
/// 
/// # Errors
/// Returns an error, when fails to compile the libraries. 
pub fn compile_c_libraries(project_dir: &PathBuf, board_id: &String) -> Result<(), String> {
    let app_dir = get_project_app_dir(project_dir)?;
    let pio_proj = app_dir.join(PIO_COMPILATION_PROJECT_DIR_NAME);

    if !check_if_is_pio_dir(&pio_proj) {
        return Err("Missing PlatformIO project.".to_string());
    }

    // project compilation
    let (venv_dir, core_dir) = get_pio_dirs()?;
    let pio_args = [
        "run"
    ];
    let output = run_pio_command(&venv_dir, &core_dir, &pio_args, Some(&pio_proj));
    check_command_output(
        output, 
        "Failed to compile PlatformIO project with error:", 
        "PlatformIO failed to execute run command"
    )?;

    // ensure directory exists
    let compiled_dir = app_dir.join(COMPILED_LIBS_DIR_NAME);
    clear_dir(&compiled_dir)?;
    ensure_dir_exists(&compiled_dir)?;
    
    let pio_build_dir = pio_proj.join(PIO_PROJECT_APP_DIR_NAME)
                                .join("build")
                                .join(board_id);
        
    if !pio_build_dir.exists() {
        return Err("Missing pio build directory.".to_string());
    }

    // copying compiled libraries to same directory
    let wrapper_dir = pio_build_dir.join("src");
    copy_lib_to_dir("wrapper.cpp.o", &wrapper_dir, &compiled_dir)?;

    copy_lib_to_dir("libFrameworkArduino.a", &pio_build_dir, &compiled_dir)?;
    copy_lib_to_dir("libFrameworkArduinoVariant.a", &pio_build_dir, &compiled_dir)?;

    let libs = search_libs_in_pio_build(&pio_build_dir)?;

    for (lib_name, lib_dir) in libs {
        copy_lib_to_dir(&lib_name, &lib_dir, &compiled_dir)?;
    }

    Ok(())    
}

/// Recursively searches the PlatformIO build directory for `.a` static libraries.
/// 
/// # Arguments
/// * `build_dir` - The path of directory containing PlatformIO build.
/// 
/// # Errors
/// Returns an error if fails to parse library name, or read the directory entries.
fn search_libs_in_pio_build(build_dir: &PathBuf) -> Result<Vec<(String, PathBuf)>, String> {
    let entries = get_dir_entries(build_dir)?;

    let mut lib_paths: Vec<(String, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        
        let potential_lib_dir = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => {
                continue;
            }
        };
        if path.is_dir() && potential_lib_dir.starts_with("lib") {
            let sub_entries = get_dir_entries(&path)?;
            for sub_entry in sub_entries.flatten() {
                let file_path = sub_entry.path();
                if file_path.extension().map_or(false, |ext| ext == "a") {
                    let lib_name = match file_path.file_name() {
                        Some(s) => s.to_string_lossy().to_string(),
                        None => {
                            return Err("Failed to get library name.".to_string());
                        } 
                    };

                    lib_paths.push((lib_name, path.clone()));
                }
            }
        }
    }
    Ok(lib_paths)
}

/// Helper function to safely read the entries of a directory.
/// 
/// # Errors
/// Returns an error when it fails to read the given directory content.
fn get_dir_entries(dir: &PathBuf) -> Result<ReadDir, String> {
    match fs::read_dir(dir) {
        Ok(es) => Ok(es),
        Err(_) => Err("PlatformIO build dir does not contain any libraries.".to_string())
    }
}

/// Copies a specific compiled library or object file into the central linking directory.
/// 
/// # Arguments
/// * `lib_name` - The library name.
/// * `src_dir` - The path of the directory containing the library.
/// * `dest_dir` - The path of the directory where the library should be copied.
/// 
/// # Errors
/// Returns an error if the library, source directory, or destination does not exist.
fn copy_lib_to_dir(lib_name: &str, src_dir: &PathBuf, dest_dir: &PathBuf) -> Result<(), String> {
    let lib = src_dir.join(lib_name);
    if !lib.exists() {
        return Err("Tried to copy non-existing library.".to_string());
    }

    if !dest_dir.exists() {
        return Err("Non-existing destination dir for library.".to_string());
    }
    let dest_lib = dest_dir.join(lib_name);

    match fs::copy(&lib, &dest_lib) {
        Ok(_) => {},
        Err(_) => {
            return Err("Failed to copy library.".to_string());
        }
    };

    Ok(())
}

/// Executes a PlatformIO command and returns its output (stdout/stderr captured).
/// 
/// Automatically sets the `PLATFORMIO_CORE_DIR` environment variable to ensure 
/// the command uses the isolated internal installation.
/// 
/// # Arguments
/// * `venv_dir` - The path of a virtual environment directory.
/// * `core_dir` - The path of a PlatformIO core directory.
/// * `pio_args` - The list of a PlatformIO command arguments.
/// * `run_dir` - The path of the directory, where command should be run.
/// 
/// # Errors
/// Returns an error if command fails to execute.
fn run_pio_command(venv_dir: &PathBuf, core_dir: &PathBuf, pio_args: &[&str], run_dir: Option<&PathBuf>) -> Result<Output, Error> {
    let pio_path = get_venv_executable(venv_dir, "pio");
    match run_dir {
        Some(dir) => {
            Command::new(pio_path)
                .env("PLATFORMIO_CORE_DIR", core_dir) 
                .env("PYTHONIOENCODING", "utf-8")
                .args(pio_args)
                .current_dir(dir)
                .output()
        },
        None => {
            Command::new(pio_path)
                .env("PLATFORMIO_CORE_DIR", core_dir)
                .env("PYTHONIOENCODING", "utf-8") 
                .args(pio_args)
                .output()
        }
    }
}

/// Executes a PlatformIO command directly hooked into the host's terminal (`Stdio::inherit()`).
/// 
/// Used primarily for the serial monitor and interactive prompts.
///
/// # Arguments
/// * `venv_dir` - The path of a virtual environment directory.
/// * `core_dir` - The path of a PlatformIO core directory.
/// * `pio_args` - The list of a PlatformIO command arguments.
/// * `run_dir` - The path of the directory, where command should be run.
/// 
/// # Errors
/// Returns an error if command fails to execute.
fn run_pio_command_with_output(venv_dir: &PathBuf, core_dir: &PathBuf, pio_args: &[&str], run_dir: Option<&PathBuf>) -> Result<ExitStatus, String> {
    let pio_path = get_venv_executable(venv_dir, "pio");
    let mut child = match run_dir {
        Some(dir) => {
            Command::new(pio_path)
                .env("PLATFORMIO_CORE_DIR", core_dir) 
                .env("PYTHONIOENCODING", "utf-8")
                .args(pio_args)
                .current_dir(dir)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .map_err(|e| format!("Failed to spawn PlatformIO process: {}", e))?
        },
        None => {
            Command::new(pio_path)
                .env("PLATFORMIO_CORE_DIR", core_dir) 
                .env("PYTHONIOENCODING", "utf-8")
                .args(pio_args)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .map_err(|e| format!("Failed to spawn PlatformIO process: {}", e))?
        }
    };
    let status = child.wait()
        .map_err(|e| format!("Failed to wait on PlatformIO process: {}", e))?;
    Ok(status)
}

/// Checks the output of executed command and returns the STDOUT if succeeded.
/// 
/// # Arguments
/// * `output` - The output of the command.
/// * `fail_msg` - The error message if the command was executed, but failed.
/// * `error_msg` - The error message if the command wasn't executed.
/// 
/// # Errors
/// Returns error if command for some reason didn't succeeded.
fn check_command_output(
    output: Result<Output, Error>,
    fail_msg: &str,
    error_msg: &str,
) -> Result<Vec<u8>, String> {
    match output {
        Ok(output) => {
            if !output.status.success() {
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                return Err(format!("{}\n{}", fail_msg, stderr_str))
            }
            return Ok(output.stdout);
        },
        Err(_) => {
            return Err(error_msg.to_string());
        } 
    };
}

//
// Unit Tests
//

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn test_copy_lib_to_dir_success() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src");
        let dest_dir = temp.path().join("dest");
        
        fs::create_dir(&src_dir).unwrap();
        fs::create_dir(&dest_dir).unwrap();
        
        let lib_name = "libCore.a";
        let lib_path = src_dir.join(lib_name);
        // create dummy library
        File::create(&lib_path).unwrap(); 
        
        let result = copy_lib_to_dir(lib_name, &src_dir, &dest_dir);
        assert!(result.is_ok());
        
        let copied_lib = dest_dir.join(lib_name);
        assert!(copied_lib.exists());
    }

    #[test]
    fn test_copy_lib_to_dir_missing_src() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src");
        let dest_dir = temp.path().join("dest");
        
        fs::create_dir(&src_dir).unwrap();
        fs::create_dir(&dest_dir).unwrap();
        
        // don't create the library file in src_dir
        let result = copy_lib_to_dir("libMissing.a", &src_dir, &dest_dir);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Tried to copy non-existing library.");
    }

    #[test]
    fn test_copy_lib_to_dir_missing_dest() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src");
        let dest_dir = temp.path().join("dest");
        
        fs::create_dir(&src_dir).unwrap();
        // don't create dest_dir
        
        let lib_name = "libCore.a";
        let lib_path = src_dir.join(lib_name);
        File::create(&lib_path).unwrap();
        
        let result = copy_lib_to_dir(lib_name, &src_dir, &dest_dir);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Non-existing destination dir for library.");
    }

    #[test]
    fn test_search_libs_in_pio_build() {
        let temp = tempdir().unwrap();
        let build_dir = temp.path().join("build");
        fs::create_dir(&build_dir).unwrap();
        
        // create a valid library structure: build/libTest/libTest.a
        let lib_test_dir = build_dir.join("libTest");
        fs::create_dir(&lib_test_dir).unwrap();
        File::create(lib_test_dir.join("libTest.a")).unwrap();
        
        // create a invalid file to ensure it gets ignored (e.g. object files)
        File::create(lib_test_dir.join("libTest.o")).unwrap();
        
        // create a directory that doesn't start with "lib" but contains a .a file
        let other_dir = build_dir.join("otherThing");
        fs::create_dir(&other_dir).unwrap();
        File::create(other_dir.join("libHidden.a")).unwrap();
        
        let result = search_libs_in_pio_build(&build_dir).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "libTest.a");
        assert_eq!(result[0].1, lib_test_dir);
    }

    #[test]
    fn test_get_dir_entries_failure() {
        let temp = tempdir().unwrap();
        let missing_dir = temp.path().join("non_existent_folder");
        
        let result = get_dir_entries(&missing_dir);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "PlatformIO build dir does not contain any libraries.");
    }
}