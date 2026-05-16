//! Manages the `Prustio.toml` configuration file.
//!
//! This module defines the data structures representing the project's configuration
//! and provides functions to create, read, update, and parse these settings.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::model::board::UNSPECIFIED_PARAM;


const PRUSTIO_CONFIG_FILE_NAME: &str = "Prustio.toml";


/// The root structure of the `Prustio.toml` configuration file.
#[derive(Debug, Deserialize, Serialize)]
pub struct Configuration {
    /// General information about the project.
    pub package: Package,
    /// A map of available environments (hardware targets), keyed by environment name.
    env: Option<BTreeMap<String, Env>>,
}

impl Configuration {
    /// Parses a TOML string into a `Configuration` struct.
    ///
    /// # Arguments
    /// * `content` - The raw string content of a `Prustio.toml` file.
    ///
    /// # Errors
    /// Returns an error if the string cannot be parsed into the expected TOML format.
    pub fn from(content: &String) -> Result<Configuration, String> {
        let mut config: Configuration = match toml_edit::de::from_str(content) {
            Ok(c) => c,
            Err(_) => {
                return Err(String::from("Failed to parse PrustIO configuration file."));
            }
        };

        if let Some(ref mut env_tree) = config.env {
            for (env_name, env_cofing) in env_tree.iter_mut() {
                env_cofing.name = env_name.clone();
            }
        }

        Ok(config)
    }

    /// Saves the current configuration state to disk as `Prustio.toml`.
    ///
    /// # Arguments
    /// * `proj_path` - The root directory of the project.
    ///
    /// # Errors
    /// Returns an error if serialization fails or if the file cannot be written.
    pub fn save(&self, proj_path: &PathBuf) -> Result<(), String> {
        let file = PathBuf::from(proj_path).join(PRUSTIO_CONFIG_FILE_NAME);
        let raw_toml = match toml::to_string_pretty(self) {
            Ok(res) => res,
            Err(_) => return Err("Failed to serialize configuration.".to_string()),
        };

        let clean_toml = raw_toml.replace("[env]\n\n", "");

        if let Err(_) = fs::write(&file, clean_toml) {
            return Err("Failed to write configuration.".to_string());
        }

        Ok(())
    }

    /// Sets the currently active environment for the project.
    ///
    /// # Arguments
    /// * `env` - The name of the environment to activate (e.g., "uno").
    ///
    /// # Errors
    /// Returns an error if the specified environment does not exist in the configuration.
    pub fn set_active_env(&mut self, env: &String) -> Result<(), String> {
        if let Some(envs) = &self.env {
            if envs.contains_key(env) {
                self.package.set_active_env(env);
                return Ok(());
            }
            return Err("Invalid environment name.".to_string());
        }
        Err("Empty environment list.".to_string())
    }
}

/// The structure containing project's metadata.
#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    /// The project name.
    pub name: String,
    /// The project version. 
    version: String,
    /// The hybrid mode flag.
    pub hybrid_mode: bool,
    /// The environment that project is configured for.
    pub active_env: Option<String>,
}

impl Package {
    /// Initializes package configuration structure from given arguments.
    /// 
    /// # Arguments
    /// * `name` - The name of the project.
    /// * `version` - Current version of the project.
    /// * `hybrid_mode` - The flag showing if project uses the hybrid mode.
    pub fn new(name: &String, version: &String, hybrid_mode: &bool) -> Package {
        Package { 
            name: name.clone(), 
            version: version.clone(), 
            hybrid_mode: hybrid_mode.clone(),
            active_env: None, 
        }
    }

    /// Changes currently active environment.
    /// 
    /// # Arguments
    /// * `env` - The name of the new active environment.
    fn set_active_env(&mut self, env: &String) {
        self.active_env = Some(env.clone());
    }
}

/// The structure representing single environment configuration.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Env {
    /// The name of environment.
    #[serde(skip)]
    pub name: String,

    /// The list of default targets for `prustio run` command.
    pub targets: Option<Vec<String>>,
    /// The ID specifying target board.
    pub board: String,
    /// The used framework.
    pub framework: Option<String>,
}

/*
 -----------------------
    Creating configuration
 -----------------------
*/


/// Creates a new `Prustio.toml` configuration file on disk.
///
/// # Arguments
/// * `proj_path` - The root directory of the project.
/// * `project_name` - The name of the package.
/// * `hybrid_mode` - Whether the project uses PlatformIO C/C++ bindings.
/// * `board_id` - The target board identifier.
/// * `framework` - The underlying framework (e.g., "arduino"), used in hybrid mode.
///
/// # Errors
/// Returns an error if the file cannot be written to the specified path.
pub fn create_prustio_config(
    proj_path: &PathBuf,
    project_name: &String,
    hybrid_mode: &bool,
    board_id: &String,
    framework: Option<&String>,
) -> Result<(), String> {
    let content = match board_id.as_str() {
        UNSPECIFIED_PARAM => create_config_without_env(project_name, hybrid_mode),
        _ => create_config_with_env(project_name, hybrid_mode, board_id, framework)
    };

    content.save(proj_path)
}

/// Creates a new configuration content without specified environment.
///
/// # Arguments
/// * `project_name` - The name of the package.
/// * `hybrid_mode` - Whether the project uses PlatformIO C/C++ bindings.
fn create_config_without_env(
    project_name: &String,
    hybrid_mode: &bool,
) -> Configuration {
    Configuration {
        package: Package::new(project_name, &"0.1.0".to_string(), hybrid_mode),
        env: None      
    }
}

/// Creates a new configuration content with specified environment.
///
/// # Arguments
/// * `project_name` - The name of the package.
/// * `hybrid_mode` - Whether the project uses PlatformIO C/C++ bindings.
/// * `board_id` - The target board identifier.
/// * `framework` - The underlying framework (e.g., "arduino"), used in hybrid mode.
fn create_config_with_env(
    project_name: &String,
    hybrid_mode: &bool,
    board_id: &String,
    framework: Option<&String>,
) -> Configuration {
    let mut envs = BTreeMap::new();
    let env = Env {
        name: board_id.clone(),
        board: board_id.clone(),
        targets: None,
        framework: framework.cloned()
    }; 
    envs.insert(board_id.clone(), env);
    Configuration {
        package: Package::new(project_name, &"0.1.0".to_string(), hybrid_mode),
        env: Some(envs)
    }
}

/*
 ---------------------------
    Reading configuration
 ---------------------------
*/

/// Retrieves environment configuration based on given environment name
/// 
/// # Arguments
/// * `proj_path` - The root directory of the project.
/// * `env_name` - The name of the environment.
/// 
/// # Errors
/// Returns an error when non-existing or invalid environment name is given.
pub fn get_env(proj_path: &PathBuf, env_name: Option<&String>) -> Result<Env, String> {
    let envs = get_envs(proj_path)?;
    match env_name {
        Some(name) => {
            for (key, env) in envs {
                if key == *name {
                    return Ok(env);
                }
            }
            return Err("Invalid environment name.".to_string());
        },
        None => {
            match envs.values().next() {
                Some(e) => {
                    return Ok(e.clone());
                },
                None => {
                    return Err("No environment specified in the configuration file.".to_string())
                }
            }
        }
    }
}

/// Retrieves the project's configuration from Prustio.toml file.
/// 
/// # Arguments
/// * `proj_path` - The root directory of the project.
/// 
/// # Errors
/// Returns error when invalid path or configuration file is given.
pub fn get_config(proj_path: &PathBuf) -> Result<Configuration, String> {
    let config_file = proj_path.join(PRUSTIO_CONFIG_FILE_NAME);
    
    let content = read_prustio_config(&config_file)?;
    Configuration::from(&content)
}

/// Retrieves all environment configurations from config file.
/// 
/// # Arguments
/// * `proj_path` - The root directory of the project.
/// 
/// # Errors
/// Returns error when invalid path or configuration file is given.
pub fn get_envs(proj_path: &PathBuf) -> Result<BTreeMap<String, Env>, String> {
    let config = get_config(proj_path)?;
    match config.env {
        Some(env) => Ok(env),
        None => Ok(BTreeMap::new())
    }
}

/// Retrieves the project's information.
/// # Arguments
/// * `proj_path` - The root directory of the project.
/// 
/// # Errors
/// Returns error when invalid path or configuration file is given.
pub fn get_package_information(proj_path: &PathBuf) -> Result<Package, String> {
    let config = get_config(proj_path)?;
    Ok(config.package)
}

/// Reads raw configuration string from given configuration file.
/// # Arguments
/// * `file_path` - The path of configuration file.
/// 
/// # Errors
/// Returns error when invalid configuration file is given.
fn read_prustio_config(file_path: &PathBuf) -> Result<String, String> {
    if !file_path.exists() {
        return Err(String::from("Missing PrustIO configuration file."));
    }

    match fs::read_to_string(&file_path) {
        Ok(c) => Ok(c),
        Err(_) => Err(String::from("Failed to read PrustIO configuration file.")),
    }
}

//
// Unit Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_configuration_without_env() {
        let toml_str = r#"
        [package]
        name = "test_project"
        version = "0.1.0"
        hybrid_mode = false
        "#;
        let config = Configuration::from(&toml_str.to_string()).unwrap();
        assert_eq!(config.package.name, "test_project");
        assert_eq!(config.package.hybrid_mode, false);
        assert!(config.env.is_none());
    }

    #[test]
    fn test_parse_configuration_with_env() {
        let toml_str = r#"
        [package]
        name = "hybrid_proj"
        version = "0.1.0"
        hybrid_mode = true

        [env.uno]
        board = "uno"
        "#;
        let config = Configuration::from(&toml_str.to_string()).unwrap();
        assert_eq!(config.package.hybrid_mode, true);
        assert!(config.env.is_some());
        assert!(config.env.unwrap().contains_key("uno"));
    }

    #[test]
    fn test_set_active_env_success_and_fail() {
        let mut config = create_config_with_env(&"proj".to_string(), &true, &"uno".to_string(), None);
        
        // Success case
        assert!(config.set_active_env(&"uno".to_string()).is_ok());
        assert_eq!(config.package.active_env, Some("uno".to_string()));

        // Fail case
        assert!(config.set_active_env(&"mega".to_string()).is_err());
    }
}