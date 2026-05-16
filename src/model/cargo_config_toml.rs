//! Manages the `.cargo/config.toml` file for cross-compiling to AVR.
//!
//! Because standard Rust does not natively target AVR microcontrollers out of the box
//! without special flags, this module generates the necessary Cargo configuration. 
//! It specifies the target architecture (`avr-none`), the specific MCU (`target-cpu`), 
//! enables compiling the core library (`build-std`), and optionally injects a custom 
//! linker (like `avr-gcc`) when running in hybrid mode.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const CONFIGURATION_DIR_NAME: &str = ".cargo";
const CONFIGURATION_FILE_NAME: &str = "config.toml";


/// The root structure representing the `.cargo/config.toml` file.
#[derive(Debug, Deserialize, Serialize)]
pub struct CargoConfigToml {
    /// Configuration under the `[build]` section.
    build: CargoConfigBuild,
    /// Configuration under the `[target.<architecture>]` section.
    target: BTreeMap<String, CargoConfigTarget>,
    /// Configuration under the `[unstable]` section.
    unstable: CargoConfigUnstable,
}

impl CargoConfigToml {
    /// Constructs a new Cargo configuration.
    ///
    /// # Arguments
    /// * `target_architecture` - The cross-compilation target.
    /// * `target_mcu` - The specific microcontroller unit.
    /// * `linker` - An optional path to a custom linker, necessary for hybrid mode.
    pub fn new(
        target_architecture: &String,
        target_mcu: &String,
        linker: Option<&String>,
    ) -> CargoConfigToml {
        let mut targets = BTreeMap::new();
        if let Some(l) = linker {
            targets.insert(
                target_architecture.clone(), 
                CargoConfigTarget { linker: l.clone() }
            );
        }

        CargoConfigToml { 
            build: CargoConfigBuild::new(target_architecture, target_mcu),
            target: targets,
            unstable: CargoConfigUnstable {
                build_std: Vec::from(["core".to_string()]),
            }, 
        }
    }

    /// Updates existing Cargo configuration fields in memory.
    ///
    /// # Arguments
    /// * `target_architecture` - The new target architecture to set.
    /// * `target_mcu` - The new target microcontroller to set.
    /// * `linker` - The new custom linker path to set.
    pub fn update(
        &mut self,
        target_architecture: Option<&String>,
        target_mcu: Option<&String>,
        linker: Option<&String>,
    ) {
        match target_architecture {
            Some(arch) => {
                self.build.target = arch.to_ascii_lowercase();
                
            },
            None => {}
        };
        if let Some(arch) = target_architecture {
            if let Some(link) = linker {
                self.target.insert(
                    arch.to_ascii_lowercase(), 
                    CargoConfigTarget {
                        linker: link.clone()
                    }
                );
            }
            self.build.target = arch.to_ascii_lowercase();
        }

        match target_mcu {
            Some(mcu) => {
                self.build.rustflags = Vec::from([
                    "-C".to_string(),
                    format!("target-cpu={}", mcu.to_ascii_lowercase())
                ]) 
            },
            None => {}
        };
    } 
}

/// Represents the `[build]` section of the Cargo configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct CargoConfigBuild {
    target: String,
    rustflags: Vec<String>,
}

impl CargoConfigBuild {
    /// Constructs a new `CargoConfigBuild` representing `[build]` section.
    fn new(target_architecture: &String, target_mcu: &String) -> CargoConfigBuild {
        let arch = target_architecture.to_ascii_lowercase();
        let mcu = target_mcu.to_ascii_lowercase();
        CargoConfigBuild { 
            target: arch, 
            rustflags: Vec::from([
                "-C".to_string(),
                format!("target-cpu={}", mcu)
            ]) 
        }
    }
}

/// Represents the `[target.<architecture>]` section, typically used to override the linker.
#[derive(Debug, Deserialize, Serialize)]
pub struct CargoConfigTarget {
    linker: String
}

/// Represents the `[unstable]` section of the Cargo configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct CargoConfigUnstable {
    /// Tells Cargo to build the specified standard library crates from source (`core` for `no_std`).
    #[serde(rename = "build-std")]
    build_std: Vec<String>,
}

/// Creates a new `.cargo/config.toml` file in the project directory.
///
/// This sets up the initial environment for the specified MCU.
///
/// # Arguments
/// * `proj_path` - The root directory of the project.
/// * `target_architecture` - The compilation target.
/// * `target_mcu` - The specific microcontroller unit.
///
/// # Errors
/// Returns an error if the `.cargo` directory cannot be created, or the file cannot be written.
pub fn create_cargo_config(
    proj_path: &PathBuf,
    target_architecture: &String,
    target_mcu: &String,
) -> Result<(), String> {
    let dir_path: PathBuf = proj_path.join(CONFIGURATION_DIR_NAME);
    match fs::create_dir_all(&dir_path) {
        Ok(_) => {},
        Err(_) => {
            return Err("Failed to create .cargo folder.".to_string());
        }
    };

    let file_path = dir_path.join(CONFIGURATION_FILE_NAME);
    
    let config = CargoConfigToml::new(target_architecture, target_mcu, None);
    let content = match toml::to_string_pretty(&config) {
        Ok(c) => c,
        Err(_) => {
            return Err("Failed to parse content for .cargo/config.toml.".to_string());
        }
    };

    match fs::write(file_path, &content) {
        Ok(_) => {},
        Err(_) => {
            return Err("Failed to write config.toml configuration.".to_string());
        }
    };
    Ok(())
}


/// Updates an existing `.cargo/config.toml` file, or creates it if it doesn't exist.
///
/// This is typically called when switching environments or enabling hybrid mode, 
/// as it allows injecting the path to the custom `avr-gcc` linker.
///
/// # Arguments
/// * `proj_path` - The root directory of the project.
/// * `target_architecture` - The compilation target.
/// * `target_mcu` - The specific microcontroller unit.
/// * `linker` - An optional path to a custom linker tool.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or written back to disk.
pub fn update_cargo_config(
    proj_path: &PathBuf,
    target_architecture: &String,
    target_mcu: &String,
    linker: Option<&String>,
) -> Result<(), String> {
    let file_path = proj_path
        .join(CONFIGURATION_DIR_NAME)
        .join(CONFIGURATION_FILE_NAME);
    if !file_path.exists() {
        return create_cargo_config(proj_path, target_architecture, target_mcu);
    }
    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(_) => {
            return Err("Failed to read config.toml content.".to_string());
        }
    };
    let mut config: CargoConfigToml = match toml::de::from_str(&content) {
        Ok(c) => c,
        Err(_) => {
            return create_cargo_config(proj_path, target_architecture, target_mcu);
        }
    }; 

    config.update(Some(target_architecture), Some(target_mcu), linker);

    let content = match toml::to_string_pretty(&config) {
        Ok(c) => c,
        Err(_) => {
            return Err("Failed to parse config.toml content.".to_string());
        }
    };

    match fs::write(&file_path, content) {
        Ok(_) => {},
        Err(_) => {
            return Err("Failed to write updated configuration to config.toml.".to_string());
        }
    };
    Ok(())
}

//
// Unit Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_config_initialization_no_linker() {
        let arch = "avr-none".to_string();
        let mcu = "atmega328p".to_string();
        let config = CargoConfigToml::new(&arch, &mcu, None);

        assert_eq!(config.build.target, "avr-none");
        assert_eq!(
            config.build.rustflags, 
            vec!["-C".to_string(), "target-cpu=atmega328p".to_string()]
        );
        assert!(config.target.is_empty());
    }

    #[test]
    fn test_cargo_config_update_with_linker() {
        let arch = "avr-none".to_string();
        let mcu = "atmega328p".to_string();
        let linker = "/path/to/avr-gcc".to_string();
        
        let mut config = CargoConfigToml::new(&arch, &mcu, None);
        config.update(Some(&arch), Some(&mcu), Some(&linker));

        assert!(config.target.contains_key("avr-none"));
        assert_eq!(config.target.get("avr-none").unwrap().linker, "/path/to/avr-gcc");
    }
}