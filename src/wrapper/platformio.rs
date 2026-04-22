use std::fs::{self, ReadDir};
use std::io::Error;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Output, Stdio};

use crate::cpp_templates::{arduino_wrapper_cpp, arduino_wrapper_h};
use crate::model::platformio_ini;
use crate::ui::device::{EOL, Parity};
use crate::utils::{
    COMPILED_LIBS_DIR_NAME, PIO_COMPILATION_PROJECT_DIR_NAME, check_if_is_pio_dir, check_venv_executable_existence, clear_dir, ensure_dir_exists, get_app_dir, get_project_app_dir, get_venv_executable
};

const PIO_VENV_DIR_NAME: &str = "pio_venv";
const PIO_CORE_DIR_NAME: &str = "pio_core";
const PIO_SRC_DIR_NAME: &str = "src";
const PIO_PROJECT_APP_DIR_NAME: &str = ".pio";

pub fn check_pio_installation() -> bool {
    let venv_dir = match get_pio_dirs() {
        Ok((venv, _)) => venv,
        Err(_) => {return false;}
    }; 
    check_venv_executable_existence(&venv_dir, "pio")
}

pub fn get_boards(filter: &str) -> std::io::Result<Output> {
    let mut cmd = Command::new("pio");
    return cmd.args(["boards", filter, "--json-output"]).output();
}

pub fn get_pio_dirs() -> Result<(PathBuf, PathBuf), String> {
    // try to get local application pio dir
    // let proj_path = match env::current_dir() {
    //     Ok(path) => path,
    //     Err(_) => {
    //         return Err("Failed to get current working directory.".to_string());
    //     },
    // };
    // get global application pio dir (in home directory)
    let app_dir = get_app_dir()?;

    let pio_venv_dir = app_dir.join(PIO_VENV_DIR_NAME);
    ensure_dir_exists(&pio_venv_dir)?;

    let pio_core_dir = app_dir.join(PIO_CORE_DIR_NAME);
    ensure_dir_exists(&pio_core_dir)?;

    return Ok((pio_venv_dir, pio_core_dir));
}

// TODO check for python existence
pub fn setup_platformio() -> Result<(), String> {
    let (venv_dir, _ ) = get_pio_dirs()?; 
    let venv_status = Command::new("python3")
        .args(["-m", "venv"])
        .arg(&venv_dir)
        .status()
        .expect("Failed to execute python3. Is Python installed?");

    if !venv_status.success() {
        return Err("Failed to create python virtual environment.".to_string());
    }

    let pip_path = get_venv_executable(&venv_dir, "pip");

    let pip_status = Command::new(pip_path)
        .args(["install", "-U", "platformio"])
        .status()
        .expect("Failed to execute pip install.");

    if !pip_status.success() {
        return Err("Failed to install the platformIO.".to_string());
    }
    Ok(())
}

pub fn download_pio_toolchain(toolchain_name: &str) -> Result<(), String> {
    let (venv_dir, core_dir) = get_pio_dirs()?;
    let args = [
        "pkg", 
        "install", 
        "--global", 
        "--tool", 
        toolchain_name,
    ];
    // TODO check output.status
    match run_pio_command(&venv_dir, &core_dir, &args, None) {
        Ok(_) => {},
        Err(_) => {
            return Err(String::from("PlatformIO failed to install toolchain."));
        } 
    };
    
    Ok(())
}

pub fn download_pio_platform(platform_name: &str) -> Result<(), String> {
    let (venv_dir, core_dir) = get_pio_dirs()?;
    let args = [
        "pkg", 
        "install", 
        "--global", 
        "--platform", 
        platform_name,
    ];
    // TODO check output.status
    match run_pio_command(&venv_dir, &core_dir, &args, None) {
        Ok(_) => {},
        Err(_) => {
            return Err(String::from("PlatformIO failed to install toolchain."));
        } 
    };
    
    Ok(())
}

pub fn get_devices() -> Result<Vec<u8>, String> {
    let (venv_dir, core_dir) = get_pio_dirs()?;
    let args = [
        "device", "list", "--json-output"
    ];
    let output = match run_pio_command(&venv_dir, &core_dir, &args, None) {
        Ok(o) => o,
        Err(_) => {
            return Err(String::from("PlatformIO failed to get device list."));
        } 
    };

    if !output.status.success() {
        return Err(String::from("PlatformIO failed to execute command"));
    }

    Ok(output.stdout)
}

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
    
    // Ensure data from old build will not interfere with current build
    clear_dir(&pio_proj)?;

    ensure_dir_exists(&pio_proj)?;
    let pio_path_str = match pio_proj.to_str() {
        Some(path) => path,
        None => {
            return Err("Can not parse PlatformIO compile project path to string".to_string());
        }
    };

    let (venv_dir, core_dir) = get_pio_dirs()?;
    let pio_args = [
        "project", "init", "-d", pio_path_str
    ];
    let output = match run_pio_command(&venv_dir, &core_dir, &pio_args, None) {
        Ok(o) => o,
        Err(_) => {
            return Err("Failed to init PlatformIO project".to_string());
        } 
    };

    if !output.status.success() {
        return Err("PlatformIO failed to execute init command".to_string());
    }

    platformio_ini::rewrite_pio_config(&pio_proj, platform, board_id, framework, platform_packages, lib_deps)?;

    let pio_src = pio_proj.join(PIO_SRC_DIR_NAME);
    ensure_dir_exists(&pio_src)?;

    let header_file = pio_src.join(arduino_wrapper_h::NAME);
    match std::fs::write(header_file, arduino_wrapper_h::CONTENT) {
        Ok(_) => (),
        Err(_) => {
            return Err("Failed to write wrapper.h content.".to_string()); 
        }
    };

    let cpp_file = pio_src.join(arduino_wrapper_cpp::NAME);
    match std::fs::write(cpp_file, arduino_wrapper_cpp::CONTENT) {
        Ok(_) => (),
        Err(_) => {
            return Err("Failed to write wrapper.cpp content.".to_string()); 
        }
    };

    Ok(())
}

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
    let output = match run_pio_command(&venv_dir, &core_dir, &pio_args, Some(&pio_proj)) {
        Ok(o) => o,
        Err(_) => {
            return Err("Failed to execute PlatformIO pkg list command.".to_string());
        } 
    };

    if !output.status.success() {
        return Err("PlatformIO pkg list failed.".to_string());
    }

    match String::from_utf8(output.stdout) {
        Ok(stdout) => Ok(stdout),
        Err(_) => Err("Failed to parse PlatformIO pkg list output as UTF-8".to_string()),
    }
}

pub fn compile_c_libraries(project_dir: &PathBuf, board_id: &String) -> Result<(), String> {
    let app_dir = get_project_app_dir(project_dir)?;
    let pio_proj = app_dir.join(PIO_COMPILATION_PROJECT_DIR_NAME);

    if !check_if_is_pio_dir(&pio_proj) {
        return Err("Missing PlatformIO project.".to_string());
    }

    let (venv_dir, core_dir) = get_pio_dirs()?;
    let pio_args = [
        "run"
    ];
    let output = match run_pio_command(&venv_dir, &core_dir, &pio_args, Some(&pio_proj)) {
        Ok(o) => o,
        Err(_) => {
            return Err("Failed to compile PlatformIO project".to_string());
        } 
    };

    if !output.status.success() {
        return Err("PlatformIO failed to execute run command".to_string());
    }

    let compiled_dir = app_dir.join(COMPILED_LIBS_DIR_NAME);
    clear_dir(&compiled_dir)?;
    ensure_dir_exists(&compiled_dir)?;

    let pio_build_dir = pio_proj.join(PIO_PROJECT_APP_DIR_NAME)
                                .join("build")
                                .join(board_id);
    if !pio_build_dir.exists() {
        return Err("Missing pio build directory.".to_string());
    }

    // archive the wrapper
    // let wrapper_obj = pio_build_dir.join("src").join("wrapper.cpp.o");
    // // let wrapper_lib = compiled_dir.join("libWrapper.a");
    // // avr::archive_object_file(&wrapper_obj, &wrapper_lib)?;

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

// TODO - made to search for FrameworkArduino too
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

fn get_dir_entries(dir: &PathBuf) -> Result<ReadDir, String> {
    match fs::read_dir(dir) {
        Ok(es) => Ok(es),
        Err(_) => Err("PlatformIO build dir does not contain any libraries.".to_string())
    }
}

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

fn run_pio_command(venv_dir: &PathBuf, core_dir: &PathBuf, pio_args: &[&str], run_dir: Option<&PathBuf>) -> Result<Output, Error> {
    let pio_path = get_venv_executable(venv_dir, "pio");
    match run_dir {
        Some(dir) => {
            Command::new(pio_path)
                .env("PLATFORMIO_CORE_DIR", core_dir) 
                .args(pio_args)
                .current_dir(dir)
                .output()
        },
        None => {
            Command::new(pio_path)
                .env("PLATFORMIO_CORE_DIR", core_dir) 
                .args(pio_args)
                .output()
        }
    }
}

fn run_pio_command_with_output(venv_dir: &PathBuf, core_dir: &PathBuf, pio_args: &[&str], run_dir: Option<&PathBuf>) -> Result<ExitStatus, String> {
    let pio_path = get_venv_executable(venv_dir, "pio");
    let mut child = match run_dir {
        Some(dir) => {
            Command::new(pio_path)
                .env("PLATFORMIO_CORE_DIR", core_dir) 
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
