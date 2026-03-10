use std::{path::PathBuf, process::Command};

use crate::wrapper::platformio;

const PIO_AVRDUDE_PACKAGE: &str = "tool-avrdude";
const BINARY_NAME: &str = "avrdude";

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
            return Err(String::from("Failed to upload binary"));
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
        Ok(output) => {
            if !output.status.success() {
                return Err(String::from("Tool avrdude failed."));
            }
        },
        Err(_) => {
            return Err(String::from("Failed to run avrdude tool."));
        }
    }
    Ok(())
}