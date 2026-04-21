use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::PathBuf};

const PIO_CONFIG_FILE_NAME: &str = "platformio.ini";

#[derive(Deserialize, Serialize, Debug)]
pub struct PioEnvConfig {
    platform: String,
    board: String,
    framework: String,
    build_flags: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    platform_packages: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    lib_deps: Option<String>,
}

impl PioEnvConfig {
    fn new(
        platform: &String,
        board_id: &String,
        framework: &String,
        platform_packages: Option<&Vec<String>>,
        lib_deps: Option<&Vec<String>>,
    ) -> PioEnvConfig {
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

    match fs::write(config_file, config_str) {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to write configuration to the PlatformIO's configuration file.".to_string())
    }
}