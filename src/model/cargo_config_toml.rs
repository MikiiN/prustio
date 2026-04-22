use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const CONFIGURATION_DIR_NAME: &str = ".cargo";
const CONFIGURATION_FILE_NAME: &str = "config.toml";


#[derive(Debug, Deserialize, Serialize)]
pub struct CargoConfigToml {
    build: CargoConfigBuild,
    target: BTreeMap<String, CargoConfigTarget>,
    unstable: CargoConfigUnstable,
}

impl CargoConfigToml {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct CargoConfigBuild {
    target: String,
    rustflags: Vec<String>,
}

impl CargoConfigBuild {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct CargoConfigTarget {
    linker: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CargoConfigUnstable {
    #[serde(rename = "build-std")]
    build_std: Vec<String>,
}


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

// TODO
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