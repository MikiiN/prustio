use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use toml_edit::{Array, DocumentMut, Item, Table, value, ser, Value};

use crate::model::boards;

const PRUSTIO_CONFIG_FILE_NAME: &str = "Prustio.toml";


#[derive(Debug, Deserialize, Serialize)]
pub struct Configuration {
    package: Package,
    env: Option<HashMap<String, Env>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    name: String,
    version: String,
    hybrid_mode: bool
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
    pub targets: Option<Vec<String>>,
    pub board: String,
    pub framework: Option<String>,
}

/*
 -----------------------
    Creating/Updating configuration
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

    // TODO
    let content = Configuration {
        package: Package::new(project_name, &"0.1.0".to_string(), hybrid_mode),
        env: match board_id.as_str() {
            boards::UNSPECIFIED_PARAM => None,
            _ => {
                let mut e = HashMap::new();
                 let env = Env {
                    board: board_id.clone(),
                    targets: None,
                    framework: framework.cloned()
                }; 
                e.insert(board_id.clone(), env);
                Some(e)        
            }
        }
    };

    let mut doc = match ser::to_document(&content) {
            Ok(res) => res,
            Err(_) => {
                return Err("Failed to serialize configuration.".to_string());
            }
        };

    if let Item::Value(Value::InlineTable(inline)) = &doc["package"] {
        doc["package"] = Item::Table(inline.clone().into_table());
    }

    if board_id != boards::UNSPECIFIED_PARAM {
        if let Item::Value(Value::InlineTable(inline)) = &doc["env"] {
            let mut env_table = inline.clone().into_table();
            env_table.set_implicit(true);
    
            if let Item::Value(Value::InlineTable(uno_inline)) = &env_table[board_id] {
                env_table[board_id] = Item::Table(uno_inline.clone().into_table());
            }
    
            doc["env"] = Item::Table(env_table);
        }
    }

    match fs::write(&file, doc.to_string()) {
        Ok(_) => {},
        Err(_) => {
            return Err("Failed to write configuration.".to_string());
        }
    };

    Ok(())
}

/*
 ---------------------------
    Reading configuration
 ---------------------------
*/

pub fn get_env(proj_path: &PathBuf) -> Result<HashMap<String, Env>, String> {
    let config_file = proj_path.join(PRUSTIO_CONFIG_FILE_NAME);
    if !config_file.exists() {
        return Err(String::from("Missing PrustIO configuration file."));
    }

    let content = match fs::read_to_string(&config_file) {
        Ok(c) => c,
        Err(_) => {
            return Err(String::from("Failed to read PrustIO configuration file."));
        },
    };
    
    let config: Configuration = match toml_edit::de::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            println!("Error: {:?}",e);
            return Err(String::from("Failed to parse PrustIO configuration file."));
        }
    };
    match config.env {
        Some(env) => Ok(env),
        None => Ok(HashMap::new())
    }
}