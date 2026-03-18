use std::fs;
use std::path::PathBuf;
use toml_edit::{DocumentMut, Item, Table, Array, value};

const RUST_TOOLCHAIN_FILE_NAME: &str = "rust-toolchain.toml";


pub fn create_toolchain_config(proj_path: &PathBuf, rustc_version: &String) -> Result<(), String> {
    let file_path = PathBuf::from(proj_path).join(RUST_TOOLCHAIN_FILE_NAME);
    let mut toml = DocumentMut::new();

    write_toolchain_init_content(&mut toml, rustc_version);

    match fs::write(&file_path, toml.to_string()) {
        Ok(_) => {},
        Err(_) => {
            return Err("Failed to write rust-toolchain.toml configuration.".to_string());
        } 
    };
    Ok(())
}

fn write_toolchain_init_content(toml: &mut DocumentMut, rustc_version: &String) {
    let mut toolchain = Table::new();
    toolchain["channel"] = value(rustc_version);
    let mut components = Array::new();
    components.push("rust-src");
    toolchain["components"] = value(components);
    toolchain["profile"] = value("minimal");
    toml["toolchain"] = Item::Table(toolchain);    
}