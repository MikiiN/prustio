use std::{path::PathBuf, process::Command};

use crate::wrapper::platformio;

const PIO_AVR_PACKAGE: &str = "toolchain-atmelavr";
const BINARY_DIRECTORY: &str = "bin";
const BINARY_NAME: &str = "avr-objcopy"; 

pub fn elf_to_hex(elf_file_path: &PathBuf, hex_file_path: &PathBuf) -> Result<(), String>{
    let (_, core_dir) = platformio::get_pio_dirs()?;

    let bin_path = core_dir.join("packages")
                           .join(PIO_AVR_PACKAGE)
                           .join(BINARY_DIRECTORY)
                           .join(BINARY_NAME);
    if !bin_path.exists() {
        platformio::download_pio_toolchain(PIO_AVR_PACKAGE)?;
    }
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