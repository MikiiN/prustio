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
    pub fn new(name: &String, feature: &String, hybrid: &bool) -> CargoToml {
        CargoToml { 
            package: PackageConfig::new(name), 
            dependencies: DependenciesConfig::new(feature, hybrid), 
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
    pub arduino_hal: GitHubCrate,

    #[serde(rename = "prustio-arduino")]
    pub prustio_arduino: Option<GitHubCrate>,

    #[serde(rename = "build-dependencies")]
    pub build_dependencies: Option<BuildDependencies>,
}

impl DependenciesConfig {
    pub fn new(feature: &String, hybrid: &bool) -> DependenciesConfig {
        DependenciesConfig { 
            panic_halt: "1.0.0".to_string(), 
            ufmt: "0.2.0".to_string(), 
            nb: "1.1.0".to_string(), 
            embedded_hal: "1.0".to_string(), 
            arduino_hal: GitHubCrate::new(
                "https://github.com/rahix/avr-hal".to_string(), 
                Some("e5c8f37fe48419956e722490a82b9ca9b9fc61a2".to_string()), 
                Some(Vec::from([feature.clone()]))
            ),
            prustio_arduino: if *hybrid {
                Some(GitHubCrate::new(
                "https://github.com/MikiiN/prustio-arduino-crate".to_string(), 
                None, 
                None
                ))
            } else { None },
            build_dependencies: None
        }

    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubCrate {
    pub git: String,
    pub rev: Option<String>,
    pub features: Option<Vec<String>>
}

impl GitHubCrate {
    pub fn new(
        git: String,
        rev: Option<String>,
        features: Option<Vec<String>>
    ) -> GitHubCrate {
        GitHubCrate { 
            git: git.clone(), 
            rev: rev,
            features: features
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildDependencies {
    pub fs_extra: String
}

impl BuildDependencies {
    pub fn new() -> BuildDependencies {
        BuildDependencies { fs_extra: "1.3".to_string() }
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
            name: DEFAULT_BIN_NAME.to_string(),
            path: DEFAULT_MAIN_PATH.to_string(),
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
    board_feature: &String,
    hybrid: &bool,
) -> Result<(), String> {
    let file_path = PathBuf::from(proj_path).join(CARGO_TOML_FILE_NAME);
    
    let config = CargoToml::new(project_name, board_feature, hybrid);

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
