use std::{path::PathBuf, process::Command};

use crate::wrapper::platformio;

const PIO_AVR_PACKAGE: &str = "toolchain-atmelavr";
const BINARY_DIRECTORY: &str = "bin";
const OBJ_COPY_BINARY_NAME: &str = "avr-objcopy";
const AR_BINARY_NAME: &str = "avr-ar";

pub const GCC_BINARY_NAME: &str = "avr-gcc";

pub fn elf_to_hex(elf_file_path: &PathBuf, hex_file_path: &PathBuf) -> Result<(), String>{
    let bin_path = obtain_bin_path(OBJ_COPY_BINARY_NAME)?;
    
    let mut cmd = Command::new(&bin_path);
    let output = cmd.arg("-O")
                    .arg("ihex")
                    .arg("-R")
                    .arg(".eeprom")
                    .arg(elf_file_path)
                    .arg(hex_file_path)
                    .output();
    match output {
        Ok(output) => {
            if !output.status.success() {
                return Err(String::from("Tool avr-objcopy failed."));
            }
        },
        Err(_) => {
            return Err(String::from("Failed to run avr-objcopy tool."));
        }
    }
    Ok(())
}

pub fn archive_object_file(object_file_path: &PathBuf, archive_path: &PathBuf) -> Result<(), String> {
    let bin_path = obtain_bin_path(AR_BINARY_NAME)?;

    let mut cmd = Command::new(&bin_path);
    let output = cmd.arg("rcs")
                    .arg(archive_path)
                    .arg(object_file_path)
                    .output();
    match output {
        Ok(output) => {
            if !output.status.success() {
                return Err("Tool avr-ar failed.".to_string());
            }
        },
        Err(_) => {
            return Err("Failed to run avr-ar tool.".to_string());
        }
    }

    Ok(())
}

pub fn obtain_bin_path(bin_name: &str) -> Result<PathBuf, String> {
    let (_, core_dir) = platformio::get_pio_dirs()?;

    let bin_path = core_dir.join("packages")
                           .join(PIO_AVR_PACKAGE)
                           .join(BINARY_DIRECTORY)
                           .join(bin_name);
    if !bin_path.exists() {
        platformio::download_pio_toolchain(PIO_AVR_PACKAGE)?;
    }

    Ok(bin_path)
}