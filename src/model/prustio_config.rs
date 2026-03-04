use std::fs;
use std::path::PathBuf;
use toml_edit::{DocumentMut, Item, Table, Array, ArrayOfTables, value};

use crate::model::boards;

const PRUSTIO_CONFIG_FILE_NAME: &str = "Prustio.toml";

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

    if board_id != boards::UNSPECIFIED_BOARD_PARAM {
        add_prustio_config_target(&mut toml, board_id, board_id, framework);
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
    package["mode"] = value(*hybrid_mode); 

    toml["package"] = Item::Table(package);
}

pub fn add_prustio_config_target(
    toml: &mut DocumentMut,
    target_name: &String,
    // target_platform: &String,
    target_board: &String,
    target_framework: &Option<String>,
) {
    let mut target = Table::new();
    // target["platform"] = value(target_platform);
    target["board"] = value(target_board);
    match target_framework {
        Some(f) => target["framework"] = value(f),
        None => (),
    };
    
    if let Some(t) = toml.get("target") {
        if !t.is_table() {
            // TODO
            println!("Error: target is invalid element");
        }
    }
    else {
        toml["target"] = Item::Table(Table::new());
    }
    toml["target"][target_name] = Item::Table(target);

}