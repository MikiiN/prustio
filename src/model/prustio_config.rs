use std::fs;
use std::path::PathBuf;
use toml_edit::{DocumentMut, Item, Table, Array, ArrayOfTables, value};

const PRUSTIO_CONFIG_FILE_NAME: &str = "Prustio.toml";

pub fn create_prustio_config(
    proj_path: &String,
    project_name: &String,
    hybrid_mode: &bool,

) -> std::io::Result<()> {
    let file = PathBuf::from(proj_path).join(PRUSTIO_CONFIG_FILE_NAME);

    let mut toml = DocumentMut::new();
    write_prustio_init_config(&mut toml, &project_name, &hybrid_mode);

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