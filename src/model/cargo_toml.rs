use std::fs;
use std::path::PathBuf;
use toml_edit::{DocumentMut, Item, Table, Array, ArrayOfTables, value};

const CARGO_TOML_FILE_NAME: &str = "Cargo.toml";
const DEFAULT_BIN_NAME: &str = "bin";
const DEFAULT_MAIN_PATH: &str = "src/main.rs";

pub fn create_cargo_toml_config(proj_path: &PathBuf, board_feature: &str) -> Result<(), String> {
    let file_path = PathBuf::from(proj_path).join(CARGO_TOML_FILE_NAME);
    let cargo_content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(_) => {
            return Err("Failed to read Cargo.toml file.".to_string());
        }
    };
    let mut toml = match cargo_content.parse::<DocumentMut>() {
        Ok(t) => t,
        Err(_) => {
            return Err("Failed to parse Cargo.toml file.".to_string());
        }
    };

    write_cargo_toml_content(&mut toml, board_feature);

    match fs::write(&file_path, toml.to_string()) {
        Ok(_) => {},
        Err(_) => {
            return Err("Failed to write updated Cargo.toml file.".to_string());
        }
    };
    Ok(())
}

fn write_cargo_toml_content(toml: &mut DocumentMut, board_feature: &str) {
    // get project name from cargo.toml
    // let proj_name = match toml["package"]["name"].as_str() {
    //     Some(name) => name,
    //     None => "project",
    // };

    // [bin]
    let mut bin_table = Table::new();
    bin_table["name"] = value(DEFAULT_BIN_NAME);
    bin_table["path"] = value(DEFAULT_MAIN_PATH);
    bin_table["test"] = value(false);
    bin_table["bench"] = value(false);
    let mut bin_array = ArrayOfTables::new();
    bin_array.push(bin_table);
    toml["bin"] = Item::ArrayOfTables(bin_array);

    // [dependencies]
    toml["dependencies"] = Item::Table(Table::new());
    toml["dependencies"]["panic-halt"] = value("1.0.0");
    toml["dependencies"]["ufmt"] = value("0.2.0");
    toml["dependencies"]["nb"] = value("1.1.0");
    toml["dependencies"]["embedded-hal"] = value("1.0");

    let mut arduino = Table::new();
    arduino["git"] = value("https://github.com/rahix/avr-hal");
    arduino["rev"] = value("e5c8f37fe48419956e722490a82b9ca9b9fc61a2");
    
    let mut features = Array::new();
    features.push(board_feature);
    arduino["features"] = value(features);
    toml["dependencies"]["arduino-hal"] = Item::Table(arduino);
    
    // [profile]
    toml["profile"] = Item::Table(Table::new());
    let mut dev = Table::new();
    dev["panic"] = value("abort");
    dev["lto"] = value(true);
    dev["opt-level"] = value("s");
    toml["profile"]["dev"] = Item::Table(dev);

    let mut release = Table::new();
    release["panic"] = value("abort");
    release["codegen-units"] = value(1 as i64);
    release["debug"] = value(true);
    release["lto"] = value(true);
    release["opt-level"] = value("s");
    toml["profile"]["release"] = Item::Table(release);
}