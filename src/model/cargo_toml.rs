use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CARGO_TOML_FILE_NAME: &str = "Cargo.toml";
const DEFAULT_BIN_NAME: &str = "bin";
const DEFAULT_MAIN_PATH: &str = "src/main.rs";

#[derive(Debug, Serialize, Deserialize)]
pub struct CargoToml {
    package: PackageConfig,
    dependencies: DependenciesConfig,
    bin: Vec<BinConfig>,
    profile: ProfileConfig,
}

impl CargoToml {
    pub fn new(name: &String, feature: &String) -> CargoToml {
        CargoToml { 
            package: PackageConfig::new(name), 
            dependencies: DependenciesConfig::new(feature), 
            bin: Vec::from([BinConfig::new()]), 
            profile: ProfileConfig::new(), 
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
    pub edition: String,
}

impl PackageConfig {
    pub fn new(name: &String) -> PackageConfig{
        PackageConfig {
            name: name.clone(),
            version: "0.1.0".to_string(),
            edition: "2024".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependenciesConfig {
    #[serde(rename = "panic-halt")]
    pub panic_halt: String,
    
    pub ufmt: String,
    pub nb: String,
    
    #[serde(rename = "embedded-hal")]
    pub embedded_hal: String,
    
    #[serde(rename = "arduino-hal")]
    pub arduino_hal: ArduinoHalConfig,
}

impl DependenciesConfig {
    pub fn new(feature: &String) -> DependenciesConfig {
        DependenciesConfig { 
            panic_halt: "1.0.0".to_string(), 
            ufmt: "0.2.0".to_string(), 
            nb: "1.1.0".to_string(), 
            embedded_hal: "1.0".to_string(), 
            arduino_hal: ArduinoHalConfig::new(feature) 
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArduinoHalConfig {
    pub git: String,
    pub rev: String,
    pub features: Vec<String>,
}

impl ArduinoHalConfig {
    pub fn new(feature: &String) -> ArduinoHalConfig {
        ArduinoHalConfig { 
            git: "https://github.com/rahix/avr-hal".to_string(), 
            rev: "e5c8f37fe48419956e722490a82b9ca9b9fc61a2".to_string(), 
            features: Vec::from([feature.clone()]),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BinConfig {
    pub name: String,
    pub path: String,
    pub test: bool,
    pub bench: bool,
}

impl BinConfig {
    pub fn new() -> BinConfig {
        BinConfig {
            name: "bin".to_string(),
            path: "src/main.rs".to_string(),
            test: false,
            bench: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub dev: ProfileDevConfig,
    pub release: ProfileReleaseConfig,
}

impl ProfileConfig {
    pub fn new() -> ProfileConfig {
        ProfileConfig { 
            dev: ProfileDevConfig::new(), 
            release: ProfileReleaseConfig::new(), 
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileDevConfig {
    pub panic: String,
    pub lto: bool,
    #[serde(rename = "opt-level")]
    pub opt_level: String,
}

impl ProfileDevConfig {
    pub fn new() -> ProfileDevConfig {
        ProfileDevConfig { 
            panic: "abort".to_string(), 
            lto: true, 
            opt_level: "s".to_string(), 
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileReleaseConfig {
    pub panic: String,
    #[serde(rename = "codegen-units")]
    pub codegen_units: u32,
    pub debug: bool,
    pub lto: bool,
    #[serde(rename = "opt-level")]
    pub opt_level: String,
}

impl ProfileReleaseConfig {
    pub fn new() -> ProfileReleaseConfig {
        ProfileReleaseConfig { 
            panic: "abort".to_string(), 
            codegen_units: 1, 
            debug: true, 
            lto: true, 
            opt_level: "s".to_string(),
        }
    }
}

pub fn create_cargo_toml_config(
    proj_path: &PathBuf, 
    project_name: &String, 
    board_feature: &String
) -> Result<(), String> {
    let file_path = PathBuf::from(proj_path).join(CARGO_TOML_FILE_NAME);
    
    let config = CargoToml::new(project_name, board_feature);

    let content = match toml::to_string_pretty(&config) {
        Ok(c) => c,
        Err(_) => {
            return Err("Failed to parse Cargo.toml configuration".to_string());
        }
    };

    match fs::write(&file_path, &content) {
        Ok(_) => {},
        Err(_) => {
            return Err("Failed to write updated Cargo.toml file.".to_string());
        }
    };
    Ok(())
}
