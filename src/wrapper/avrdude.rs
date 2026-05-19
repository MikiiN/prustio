//! Wrapper for the avrdude flashing utility.
//!
//! This module handles the process of uploading the compiled `.hex` firmware 
//! to the connected microcontroller. It interacts with the `avrdude` utility 
//! provided internally by PlatformIO's package manager, ensuring the user 
//! doesn't need it installed globally.

use std::{path::PathBuf, process::Command};

use crate::wrapper::platformio;

const PIO_AVRDUDE_PACKAGE: &str = "tool-avrdude";
const BINARY_NAME: &str = "avrdude";

/// Uploads a compiled HEX binary to the target microcontroller.
///
/// This function locates the internal `avrdude` executable (downloading it via 
/// PlatformIO if it doesn't exist yet) and constructs the necessary command-line 
/// arguments to flash the board over a serial port.
///
/// # Arguments
/// * `hex_path` - The absolute path to the compiled HEX firmware file.
/// * `mcu` - The target microcontroller.
/// * `platform` - The programmer/upload protocol to use.
/// * `port` - The serial port the device is connected to.
/// * `bus_speed` - The baud rate expected by the microcontroller's bootloader.
///
/// # Errors
/// Returns an error string if:
/// * PlatformIO fails to locate or download the `tool-avrdude` package.
/// * The path to the HEX file contains invalid characters and cannot be parsed.
/// * The `avrdude` command fails to execute or returns a non-zero exit status 
///   (e.g., if the device is disconnected during upload, or the wrong port is selected).
pub fn upload_binary(
    hex_path: &PathBuf,
    mcu: &String,
    platform: &String,
    port: &String,
    bus_speed: &u32,
) -> Result<(), String> {
    let (_, core_dir) = platformio::get_pio_dirs()?;

    let bin_path = core_dir.join("packages")
                           .join(PIO_AVRDUDE_PACKAGE)
                           .join(BINARY_NAME);
    if !bin_path.exists() {
        platformio::download_pio_toolchain(PIO_AVRDUDE_PACKAGE)?;
    }
    let path_string = match hex_path.to_str() {
        Some(path) => String::from("flash:w:") + &String::from(path) + ":i",
        None => {
            return Err("Failed to upload binary".to_string());
        }
    };

    let mut cmd = Command::new(&bin_path);
    let output = cmd.arg("-p").arg(mcu)
                    .arg("-c").arg(platform)
                    .arg("-P").arg(port)
                    .arg("-b").arg(bus_speed.to_string())
                    .arg("-D")
                    .arg("-U").arg(path_string)
                    .output(); 
    
    match output {
        Ok(out) => {
            if !out.status.success() {
                let stderr_str = String::from_utf8_lossy(&out.stderr);
                return Err(format!("Tool avrdude failed with error:\n{}", stderr_str));
            }
        },
        Err(_) => {
            return Err("Failed to run avrdude tool.".to_string());
        }
    }
    Ok(())
}