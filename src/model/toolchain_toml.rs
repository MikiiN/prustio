use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const RUST_TOOLCHAIN_FILE_NAME: &str = "rust-toolchain.toml";

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainConfiguration {
    pub channel: String,
    pub components: Vec<String>,
    pub profile: String,
}

impl ToolchainConfiguration {
    pub fn new(
        channel: &String, 
        components: &Vec<String>, 
        profile: &String
    ) -> ToolchainConfiguration {
        ToolchainConfiguration { 
            channel: channel.clone(), 
            components: components.clone(), 
            profile: profile.clone(), 
        }
    }

    pub fn write_configuration(&self, file_path: &PathBuf) -> Result<(), String> {
        let toml = match toml::to_string_pretty(&self) {
            Ok(t) => t,
            Err(_) => {
                return Err("Failed to parse the configuration.".to_string());
            }
        };

        match fs::write(file_path, toml) {
            Ok(_) => {},
            Err(_) => {
                return Err("Failed to write rust-toolchain.toml configuration.".to_string());
            } 
        };
        Ok(())    
    }
}

pub fn create_toolchain_config(
    proj_path: &PathBuf, 
    rustc_version: &String,
    components: &Option<Vec<String>>,
    profile: &Option<String>,
) -> Result<(), String> {
    let file_path = PathBuf::from(proj_path).join(RUST_TOOLCHAIN_FILE_NAME);
    let parsed_components = match components {
        Some(c) => c,
        None => &Vec::from(["rust-src".to_string()]), 
    };
    let parsed_profile = match profile {
        Some(p) => p,
        None => &"minimal".to_string(),
    };

    let configuration = ToolchainConfiguration::new(
        rustc_version, 
        parsed_components, 
        parsed_profile,
    );

    configuration.write_configuration(&file_path)
}

pub fn update_toolchain_config(
    proj_path: &PathBuf, 
    rustc_version: &String,
    components: &Option<Vec<String>>,
    profile: &Option<String>,
) -> Result<(), String> { 
    let file_path = PathBuf::from(proj_path).join(RUST_TOOLCHAIN_FILE_NAME);
    if !file_path.exists() {
        return create_toolchain_config(proj_path, rustc_version, components, profile);
    }
    
    let config = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(_) => {
            return Err("Failed to read the rust toolchain configuration.".to_string());
        }
    };

    let mut parsed_config: ToolchainConfiguration = match toml::de::from_str(&config) {
        Ok(conf) => conf,
        Err(_) => {
            return create_toolchain_config(proj_path, rustc_version, components, profile);
        }
    };

    parsed_config.channel = rustc_version.clone();
    match components {
        Some(c) => {
            parsed_config.components = c.clone();
        },
        None => {}
    }
    match profile {
        Some(p) => {
            parsed_config.profile = p.clone();
        },
        None => {}
    }

    parsed_config.write_configuration(&file_path)
}