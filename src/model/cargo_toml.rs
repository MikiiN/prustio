//! Generates and structures the `Cargo.toml` manifest for the embedded project.
//!
//! This module constructs the necessary dependencies and build profiles required
//! to compile `no_std` Rust for AVR microcontrollers. It supports generating 
//! configurations for both pure Rust environments and hybrid C/C++ builds.

use serde::{Deserialize, Serialize};
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
    dependencies: DependenciesConfig,
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
    pub fn new(name: &String, feature: &String, hybrid: &bool) -> CargoToml {
        CargoToml { 
            package: PackageConfig::new(name), 
            dependencies: DependenciesConfig::new(feature, hybrid), 
            bin: Vec::from([BinConfig::new()]), 
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
    pub fn new(name: &String) -> PackageConfig{
        PackageConfig {
            name: name.clone(),
            version: "0.1.0".to_string(),
            edition: "2024".to_string(),
        }
    }
}

/// Represents the `[dependencies]` section in `Cargo.toml`.
#[derive(Debug, Serialize, Deserialize)]
pub struct DependenciesConfig {
    #[serde(rename = "panic-halt")]
    /// Required for `no_std` panic handling.
    pub panic_halt: String,

    /// Micro-formatted string library for embedded systems.
    pub ufmt: String,
    /// Non-blocking I/O traits.
    pub nb: String,
    
    /// Hardware Abstraction Layer traits.
    #[serde(rename = "embedded-hal")]
    pub embedded_hal: String,
    
    /// The specific AVR hardware abstraction layer.
    #[serde(rename = "arduino-hal")]
    pub arduino_hal: GitHubCrate,

    /// The custom pRustIO Arduino bindings crate (only included in hybrid mode).
    #[serde(rename = "prustio-arduino")]
    pub prustio_arduino: Option<GitHubCrate>,

    /// Build-time dependencies (e.g., for `build.rs`).
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

/// Represents a Git dependency in `Cargo.toml`.
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

/// Represents the `[build-dependencies]` section.
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildDependencies {
    pub fs_extra: String
}

impl BuildDependencies {
    pub fn new() -> BuildDependencies {
        BuildDependencies { fs_extra: "1.3".to_string() }
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
///
/// # Errors
/// Returns an error if the struct cannot be serialized to TOML or if writing to disk fails.
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


//
// Unit Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_toml_generation_pure_mode() {
        let cargo = CargoToml::new(&"pure_app".to_string(), &"arduino-uno".to_string(), &false);
        
        assert_eq!(cargo.package.name, "pure_app");
        // Ensure prustio_arduino is NOT included in pure mode
        assert!(cargo.dependencies.prustio_arduino.is_none());
        // Check hardware abstraction layer features
        assert_eq!(cargo.dependencies.arduino_hal.features, Some(vec!["arduino-uno".to_string()]));
    }

    #[test]
    fn test_cargo_toml_generation_hybrid_mode() {
        let cargo = CargoToml::new(&"hybrid_app".to_string(), &"arduino-mega2560".to_string(), &true);
        
        // Ensure prustio_arduino IS included in hybrid mode
        assert!(cargo.dependencies.prustio_arduino.is_some());
        assert_eq!(cargo.dependencies.arduino_hal.features, Some(vec!["arduino-mega2560".to_string()]));
    }
}