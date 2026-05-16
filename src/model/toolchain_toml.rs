//! Generates and manages the `rust-toolchain.toml` configuration file.
//!
//! Because compiling `no_std` Rust for AVR microcontrollers requires unstable 
//! features and building the core library from source, `pRustIO` generates a 
//! `rust-toolchain.toml` file to pin the project to a specific `nightly` 
//! compiler release and automatically install the `rust-src` component.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const RUST_TOOLCHAIN_FILE_NAME: &str = "rust-toolchain.toml";

/// The root structure representing the `rust-toolchain.toml` file.
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainConfiguration {
    toolchain: Toolchain
}

/// Represents the `[toolchain]` section.
#[derive(Debug, Deserialize, Serialize)]
pub struct Toolchain {
    pub channel: String,
    pub components: Vec<String>,
    pub profile: String,
}

impl ToolchainConfiguration {
    /// Constructs a new toolchain configuration in memory.
    ///
    /// # Arguments
    /// * `channel` - The Rust release channel to pin.
    /// * `components` - A list of components to require.
    /// * `profile` - The rustup installation profile.
    pub fn new(
        channel: &String, 
        components: &Vec<String>, 
        profile: &String
    ) -> ToolchainConfiguration {
        ToolchainConfiguration { 
                toolchain: Toolchain { 
                channel: channel.clone(), 
                components: components.clone(), 
                profile: profile.clone(),
            } 
        }
    }

    /// Serializes the toolchain configuration to TOML and saves it to disk.
    ///
    /// # Arguments
    /// * `file_path` - The file path where the TOML should be written.
    ///
    /// # Errors
    /// Returns an error string if serialization or file writing fails.
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

/// Creates a new `rust-toolchain.toml` file in the project directory.
///
/// If `components` or `profile` are not explicitly provided, it defaults to 
/// requiring `rust-src` and using the `minimal` profile.
///
/// # Arguments
/// * `proj_path` - The root directory of the project.
/// * `rustc_version` - The Rust release channel or specific nightly date.
/// * `components` - Optional list of rustup components.
/// * `profile` - Optional rustup installation profile.
///
/// # Errors
/// Returns an error if the file cannot be written.
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

/// Updates an existing `rust-toolchain.toml` file.
///
/// Reads the current configuration, updates the specified fields, and writes 
/// it back. If the file does not exist or is invalid, it completely 
/// recreates it using `create_toolchain_config`.
///
/// # Arguments
/// * `proj_path` - The root directory of the project.
/// * `rustc_version` - The Rust release channel or specific nightly date.
/// * `components` - Optional updated list of rustup components.
/// * `profile` - Optional updated rustup installation profile.
///
/// # Errors
/// Returns an error if reading or writing to the file system fails.
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

    parsed_config.toolchain.channel = rustc_version.clone();
    match components {
        Some(c) => {
            parsed_config.toolchain.components = c.clone();
        },
        None => {}
    }
    match profile {
        Some(p) => {
            parsed_config.toolchain.profile = p.clone();
        },
        None => {}
    }

    parsed_config.write_configuration(&file_path)
}


//
// Unit Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolchain_config_creation() {
        let channel = "nightly-2025-04-27".to_string();
        let components = vec!["rust-src".to_string()];
        let profile = "minimal".to_string();

        let config = ToolchainConfiguration::new(&channel, &components, &profile);

        assert_eq!(config.toolchain.channel, "nightly-2025-04-27");
        assert_eq!(config.toolchain.components, vec!["rust-src"]);
        assert_eq!(config.toolchain.profile, "minimal");
    }
}