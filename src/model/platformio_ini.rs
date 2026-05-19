//! Generates and manages the `platformio.ini` configuration file.
//!
//! When compiling in hybrid mode, `pRustIO` utilizes an internal, hidden PlatformIO
//! project to compile the Arduino C++ framework and libraries. This module creates 
//! the `platformio.ini` configuration file required by the PlatformIO Core CLI to 
//! understand what board, framework, and dependencies it needs to compile.

use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::PathBuf};

const PIO_CONFIG_FILE_NAME: &str = "platformio.ini";

/// Represents a single environment block (`[env:<board_id>]`) in `platformio.ini`.
#[derive(Deserialize, Serialize, Debug)]
pub struct PioEnvConfig {
    /// The target hardware platform.
    platform: String,
    /// The specific PlatformIO board identifier.
    board: String,
    /// The underlying C/C++ framework to compile.
    framework: String,
    /// Custom compiler flags passed to `avr-gcc` / `avr-g++`.
    /// 
    /// *Note: This is hardcoded to `-c` in pRustIO because we only want PlatformIO 
    /// to compile the source files into object files (`.o` and `.a`). We do not want 
    /// it to attempt the final linking phase, as Cargo handles linking for the Rust app.*
    build_flags: String,

    /// Optional custom PlatformIO packages to install before compilation.
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_packages: Option<String>,
    
    /// Optional list of external PlatformIO library dependencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    lib_deps: Option<String>,
}

impl PioEnvConfig {
    /// Constructs a new environment configuration block.
    ///
    /// # Arguments
    /// * `platform` - The hardware platform string.
    /// * `board_id` - The board identifier string.
    /// * `framework` - The C++ framework string.
    /// * `platform_packages` - An optional list of specific package versions or URLs.
    /// * `lib_deps` - An optional list of external libraries required by the C/C++ code.
    fn new(
        platform: &String,
        board_id: &String,
        framework: &String,
        platform_packages: Option<&Vec<String>>,
        lib_deps: Option<&Vec<String>>,
    ) -> PioEnvConfig {
        // Concatenate lists into single strings as required by the .ini format
        let packages_str = platform_packages.map(|pkgs| pkgs.join(""));
        let deps_str = lib_deps.map(|deps| deps.join(""));

        PioEnvConfig { 
            platform: platform.clone(), 
            board: board_id.clone(), 
            framework: framework.clone(), 
            build_flags: "-c".to_string(), 
            platform_packages: packages_str,
            lib_deps: deps_str,
        }
    }
}

/// Overwrites the `platformio.ini` file in the PlatformIO project directory.
///
/// This function generates the necessary `[env:<name>]` block and serializes it to 
/// the `.ini` format so that `pio run` knows exactly how to build the required 
/// C/C++ dependencies for the active board.
///
/// # Arguments
/// * `pio_proj` - The path to the hidden PlatformIO project directory.
/// * `platform` - The hardware platform string.
/// * `board_id` - The board ID string.
/// * `framework` - The C++ framework string.
/// * `platform_packages` - Optional PlatformIO packages.
/// * `lib_deps` - Optional PlatformIO library dependencies.
///
/// # Errors
/// Returns an error string if:
/// * The internal PlatformIO project directory does not exist.
/// * The data cannot be serialized to the INI format.
/// * Writing the file to disk fails.
pub fn rewrite_pio_config(
    pio_proj: &PathBuf,
    platform: &String,
    board_id: &String,
    framework: &String,
    platform_packages: Option<&Vec<String>>,
    lib_deps: Option<&Vec<String>>,
) -> Result<(), String> {
    let config_file = pio_proj.join(PIO_CONFIG_FILE_NAME);
    if !config_file.exists() {
        return Err("PlatformIO project does not exist on given path.".to_string());
    }

    let mut config: BTreeMap<String, PioEnvConfig> = BTreeMap::new();
    let env_conf = PioEnvConfig::new(platform, board_id, framework, platform_packages, lib_deps);
    
    // section header to be formatted like [env:uno]
    let section_name = format!("env:{}", board_id);
    config.insert(section_name, env_conf);

    let config_str = match serde_ini::to_string(&config) {
        Ok(s) => s,
        Err(_) => {
            return Err("Failed to parse PlatformIO's configuration.".to_string());
        }
    };

    if let Err(_) = fs::write(config_file, config_str) {
        return Err("Failed to write configuration to the PlatformIO's configuration file.".to_string());        
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
    fn test_pio_env_config_new() {
        let platform = "atmelavr".to_string();
        let board = "uno".to_string();
        let framework = "arduino".to_string();
        let packages = vec!["pkg1".to_string(), "pkg2".to_string()];
        
        let config = PioEnvConfig::new(&platform, &board, &framework, Some(&packages), None);
        
        assert_eq!(config.platform, "atmelavr");
        assert_eq!(config.board, "uno");
        assert_eq!(config.build_flags, "-c");
        // ensure vector joined successfully
        assert_eq!(config.platform_packages, Some("pkg1pkg2".to_string()));
        // ensure missing arrays remain None
        assert_eq!(config.lib_deps, None);
    }
}