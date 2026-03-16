use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use toml_edit::{Array, DocumentMut, Item, Table, value};

use crate::model::boards;

const PRUSTIO_CONFIG_FILE_NAME: &str = "Prustio.toml";

#[derive(Debug, Deserialize, Clone)]
pub struct Env {
    pub targets: Option<Vec<String>>,
    pub board: Option<String>,
    pub framework: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GlobalEnv {
    pub targets: Option<Vec<String>>,
    pub board: Option<String>,
    pub framework: Option<String>,

    #[serde(flatten)]  
    pub envs: HashMap<String, Env>,
}

impl GlobalEnv {
    pub fn to_env(&self) -> Env {
        Env { 
            targets: self.targets.clone(), 
            board: self.board.clone(), 
            framework: self.framework.clone() 
        }
    }
}

#[derive(Debug, Deserialize)]
struct Configuration {
    env: GlobalEnv,
}

pub fn create_prustio_config(
    proj_path: &PathBuf,
    project_name: &String,
    hybrid_mode: &bool,
    board_id: &String,
    framework: &Option<String>,
) -> std::io::Result<()> {
    let file = PathBuf::from(proj_path).join(PRUSTIO_CONFIG_FILE_NAME);

    let mut toml = DocumentMut::new();
    write_prustio_init_config(&mut toml, &project_name, &hybrid_mode);

    if board_id != boards::UNSPECIFIED_PARAM {
        add_prustio_config_env(&mut toml, board_id, &Vec::new(), board_id, framework);
    }

    fs::write(&file, toml.to_string())?;

    Ok(())
}

pub fn write_prustio_init_config(
    toml: &mut DocumentMut, 
    project_name: &String, 
    hybrid_mode: &bool
) {
    let mut package = Table::new();
    package["name"] = value(project_name);
    package["version"] = value("0.1.0");
    package["hybrid_mode"] = value(*hybrid_mode); 

    toml["package"] = Item::Table(package);
}

pub fn add_prustio_config_env(
    toml: &mut DocumentMut,
    env_name: &String,
    env_targets: &Vec<String>,
    // env_platform: &String,
    env_board: &String,
    env_framework: &Option<String>,
) {
    let mut env = Table::new();
    if !env_targets.is_empty() {
        let mut targets = Array::new();
        for t in env_targets {
            targets.push(t.clone());
        }
        env["targets"] = value(targets);
    }

    // env["platform"] = value(env_platform);
    env["board"] = value(env_board);
    match env_framework {
        Some(f) => env["framework"] = value(f),
        None => (),
    };
    
    if let Some(t) = toml.get("env") {
        if !t.is_table() {
            // TODO
            println!("Error: env is invalid element");
        }
    }
    else {
        toml["env"] = Item::Table(Table::new());
    }
    toml["env"][env_name] = Item::Table(env);
}

pub fn get_env(proj_path: &PathBuf) -> Result<GlobalEnv, String> {
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

    return Ok(config.env);
}