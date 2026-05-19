//! Generates and structures the `Cargo.toml` manifest for the embedded project.
//!
//! This module constructs the necessary dependencies and build profiles required
//! to compile `no_std` Rust for AVR microcontrollers. It supports generating 
//! configurations for both pure Rust environments and hybrid C/C++ builds.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const CARGO_TOML_FILE_NAME: &str = "Cargo.toml";
const DEFAULT_BIN_NAME: &str = "bin";
const DEFAULT_MAIN_PATH: &str = "src/main.rs";

/// The root structure representing the generated `Cargo.toml` manifest.
#[derive(Debug, Serialize, Deserialize)]
pub struct CargoToml {
    /// Basic package metadata (name, version, edition).
    package: PackageConfig,
    /// Project dependencies.
    dependencies: BTreeMap<String, toml::Value>,
    /// Binary target configurations.
    bin: Vec<BinConfig>,
    /// Build profile configurations.
    profile: ProfileConfig,
}

impl CargoToml {
    /// Constructs a new `Cargo.toml` configuration.
    ///
    /// # Arguments
    /// * `name` - The name of the Cargo package.
    /// * `feature` - The specific `arduino-hal` hardware feature flag.
    /// * `hybrid` - Whether the project includes hybrid C/C++ bindings.
    /// * `user_dependencies` - Extra dependencies from `Prustio.toml`.
    pub fn new(
        name: &String, 
        feature: &String, 
        hybrid: &bool,
        user_dependencies: Option<&BTreeMap<String, toml::Value>>
    ) -> CargoToml {
        let mut deps = BTreeMap::new();
        
        // standard built-in dependencies
        deps.insert("panic-halt".to_string(), toml::Value::String("1.0.0".to_string()));
        deps.insert("ufmt".to_string(), toml::Value::String("0.2.0".to_string()));
        deps.insert("nb".to_string(), toml::Value::String("1.1.0".to_string()));
        deps.insert("embedded-hal".to_string(), toml::Value::String("1.0".to_string()));
        
        let mut hal_map = toml::map::Map::new();
        hal_map.insert("git".to_string(), toml::Value::String("https://github.com/rahix/avr-hal".to_string()));
        hal_map.insert("rev".to_string(), toml::Value::String("e5c8f37fe48419956e722490a82b9ca9b9fc61a2".to_string()));
        hal_map.insert("features".to_string(), toml::Value::Array(vec![toml::Value::String(feature.clone())]));
        deps.insert("arduino-hal".to_string(), toml::Value::Table(hal_map));

        // add interface crate if in hybrid mode
        if *hybrid {
            let mut prustio_map = toml::map::Map::new();
            prustio_map.insert("git".to_string(), toml::Value::String("https://github.com/MikiiN/prustio-arduino-crate".to_string()));
            deps.insert("prustio-arduino".to_string(), toml::Value::Table(prustio_map));
        }

        // merge user dependencies
        if let Some(user_deps) = user_dependencies {
            for (key, val) in user_deps {
                deps.insert(key.clone(), val.clone());
            }
        }

        CargoToml { 
            package: PackageConfig::new(name), 
            dependencies: deps, 
            bin: vec![BinConfig::new()], 
            profile: ProfileConfig::new(), 
        }
    }
}

/// Represents the `[package]` section in `Cargo.toml`.
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
    pub edition: String,
}

impl PackageConfig {
    /// Constructs a new struct representing a `package` section in `Cargo.toml` file.
    /// 
    /// # Arguments
    /// * `name` - The project's name. 
    pub fn new(name: &String) -> PackageConfig{
        PackageConfig {
            name: name.clone(),
            version: "0.1.0".to_string(),
            edition: "2024".to_string(),
        }
    }
}

/// Represents a `[[bin]]` section for configuring executable targets.
#[derive(Debug, Serialize, Deserialize)]
pub struct BinConfig {
    pub name: String,
    pub path: String,
    pub test: bool,
    pub bench: bool,
}

impl BinConfig {
    /// Constructs a new `bin` section in `Cargo.toml` file.
    pub fn new() -> BinConfig {
        BinConfig {
            name: DEFAULT_BIN_NAME.to_string(),
            path: DEFAULT_MAIN_PATH.to_string(),
            test: false,
            bench: false,
        }
    }
}

/// Represents the `[profile]` section for build optimizations.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub dev: ProfileDevConfig,
    pub release: ProfileReleaseConfig,
}

impl ProfileConfig {
    /// Constructs a new `profile` section in `Cargo.toml` file.
    pub fn new() -> ProfileConfig {
        ProfileConfig { 
            dev: ProfileDevConfig::new(), 
            release: ProfileReleaseConfig::new(), 
        }
    }
}

/// Represents the `[profile.dev]` section.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileDevConfig {
    pub panic: String,
    pub lto: bool,
    #[serde(rename = "opt-level")]
    pub opt_level: String,
}

impl ProfileDevConfig {
    /// Constructs a new `profile.dev` section in `Cargo.toml` file.
    pub fn new() -> ProfileDevConfig {
        ProfileDevConfig { 
            panic: "abort".to_string(), 
            lto: true, 
            opt_level: "s".to_string(), 
        }
    }
}

/// Represents the `[profile.release]` section.
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
    /// Constructs a new `profile.release` section in `Cargo.toml` file.
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

/// Creates or overwrites the `Cargo.toml` configuration for the project.
///
/// Automatically includes required `no_std` crates, configuring the specific 
/// hardware features based on the selected board.
///
/// # Arguments
/// * `proj_path` - The root directory of the project.
/// * `project_name` - The name of the crate.
/// * `board_feature` - The specific `arduino-hal` feature flag.
/// * `hybrid` - If true, adds dependencies necessary for linking with PlatformIO.
/// * `user_dependencies` - Dependencies loaded from `Prustio.toml`.
///
/// # Errors
/// Returns an error if the struct cannot be serialized to TOML or if writing to disk fails.
pub fn create_cargo_toml_config(
    proj_path: &PathBuf, 
    project_name: &String, 
    board_feature: &String,
    hybrid: &bool,
    user_dependencies: Option<&BTreeMap<String, toml::Value>>,
) -> Result<(), String> {
    let file_path = PathBuf::from(proj_path).join(CARGO_TOML_FILE_NAME);
    let config = CargoToml::new(project_name, board_feature, hybrid, user_dependencies);

    let content = match toml::to_string_pretty(&config) {
        Ok(c) => c,
        Err(_) => {
            return Err("Failed to parse Cargo.toml configuration".to_string());
        }
    };

    if let Err(_) = fs::write(&file_path, &content) {
        return Err("Failed to write updated Cargo.toml file.".to_string());
    }
    Ok(())
}


//
// Unit Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_toml_generation_pure_mode() {
        let cargo = CargoToml::new(&"pure_app".to_string(), &"arduino-uno".to_string(), &false, None);
        
        assert_eq!(cargo.package.name, "pure_app");
        // ensure prustio_arduino is not included in pure mode
        assert!(cargo.dependencies.get("prustio-arduino").is_none());
        
        // Check hardware abstraction layer features via the Map
        let hal = cargo.dependencies.get("arduino-hal").unwrap().as_table().unwrap();
        let features = hal.get("features").unwrap().as_array().unwrap();
        assert_eq!(features[0].as_str().unwrap(), "arduino-uno");
    }

    #[test]
    fn test_cargo_toml_generation_hybrid_mode() {
        let cargo = CargoToml::new(&"hybrid_app".to_string(), &"arduino-mega2560".to_string(), &true, None);
        
        // ensure prustio_arduino is included in hybrid mode
        assert!(cargo.dependencies.get("prustio-arduino").is_some());
        
        let hal = cargo.dependencies.get("arduino-hal").unwrap().as_table().unwrap();
        let features = hal.get("features").unwrap().as_array().unwrap();
        assert_eq!(features[0].as_str().unwrap(), "arduino-mega2560");
    }
}