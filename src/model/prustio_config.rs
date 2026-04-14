use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::model::board::UNSPECIFIED_PARAM;


const PRUSTIO_CONFIG_FILE_NAME: &str = "Prustio.toml";


#[derive(Debug, Deserialize, Serialize)]
pub struct Configuration {
    package: Package,
    env: Option<BTreeMap<String, Env>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    version: String,
    pub hybrid_mode: bool
}

impl Package {
    pub fn new(name: &String, version: &String, hybrid_mode: &bool) -> Package {
        Package { 
            name: name.clone(), 
            version: version.clone(), 
            hybrid_mode: hybrid_mode.clone() 
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Env {
    #[serde(skip, default)]
    pub name: String,

    pub targets: Option<Vec<String>>,
    pub board: String,
    pub framework: Option<String>,
}

/*
 -----------------------
    Creating configuration
 -----------------------
*/

pub fn create_prustio_config(
    proj_path: &PathBuf,
    project_name: &String,
    hybrid_mode: &bool,
    board_id: &String,
    framework: Option<&String>,
) -> Result<(), String> {
    let file = PathBuf::from(proj_path).join(PRUSTIO_CONFIG_FILE_NAME);

    let content = match board_id.as_str() {
        UNSPECIFIED_PARAM => create_config_without_env(project_name, hybrid_mode),
        _ => create_config_with_env(project_name, hybrid_mode, board_id, framework)
    };

    let raw_toml = match toml::to_string_pretty(&content) {
        Ok(res) => res,
        Err(_) => return Err("Failed to serialize configuration.".to_string()),
    };

    let clean_toml = raw_toml.replace("[env]\n\n", "");

    match fs::write(&file, clean_toml) {
        Ok(_) => {},
        Err(_) => {
            return Err("Failed to write configuration.".to_string());
        }
    };

    Ok(())
}

fn create_config_without_env(
    project_name: &String,
    hybrid_mode: &bool,
) -> Configuration {
    Configuration {
        package: Package::new(project_name, &"0.1.0".to_string(), hybrid_mode),
        env: None      
    }
}

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

pub fn get_envs(proj_path: &PathBuf) -> Result<BTreeMap<String, Env>, String> {
    let config_file = proj_path.join(PRUSTIO_CONFIG_FILE_NAME);
    
    let content = read_prustio_config(&config_file)?;
    
    let config: Configuration = match toml_edit::de::from_str(&content) {
        Ok(c) => c,
        Err(_) => {
            return Err(String::from("Failed to parse PrustIO configuration file."));
        }
    };
    match config.env {
        Some(env) => Ok(env),
        None => Ok(BTreeMap::new())
    }
}

pub fn get_package_information(proj_path: &PathBuf) -> Result<Package, String> {
    let config_file = proj_path.join(PRUSTIO_CONFIG_FILE_NAME);
    
    let content = read_prustio_config(&config_file)?;

    let config: Configuration = match toml_edit::de::from_str(&content) {
        Ok(c) => c,
        Err(_) => {
            return Err(String::from("Failed to parse PrustIO configuration file."));
        }
    };

    Ok(config.package)
}

fn read_prustio_config(file_path: &PathBuf) -> Result<String, String> {
    if !file_path.exists() {
        return Err(String::from("Missing PrustIO configuration file."));
    }

    match fs::read_to_string(&file_path) {
        Ok(c) => Ok(c),
        Err(_) => Err(String::from("Failed to read PrustIO configuration file.")),
    }
}