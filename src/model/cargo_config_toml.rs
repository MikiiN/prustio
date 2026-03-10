use std::fs;
use std::path::PathBuf;
use toml_edit::{DocumentMut, Item, Table, Array, value};

const CONFIGURATION_DIR_NAME: &str = ".cargo";
const CONFIGURATION_FILE_NAME: &str = "config.toml";


pub fn create_cargo_config(
    proj_path: &PathBuf,
    target_architecture: &String,
    target_mcu: &String,
) -> std::io::Result<()> {
    let dir_path: PathBuf = proj_path.join(CONFIGURATION_DIR_NAME);
    fs::create_dir_all(&dir_path)?;

    let file_path = dir_path.join(CONFIGURATION_FILE_NAME);
    let mut toml = DocumentMut::new();

    write_config_toml_content(&mut toml, target_architecture, target_mcu);
    
    fs::write(file_path, toml.to_string())?;
    Ok(())
}

pub fn update_cargo_config(
    proj_path: &PathBuf,
    target_architecture: &String,
    target_mcu: &String,
) -> std::io::Result<()> {
    let file_path = proj_path
        .join(CONFIGURATION_DIR_NAME)
        .join(CONFIGURATION_FILE_NAME);
    if !file_path.exists() {
        return create_cargo_config(proj_path, target_architecture, target_mcu);
    }
    let content = fs::read_to_string(&file_path)?;
    let mut toml = match content.parse::<DocumentMut>() {
        Ok(t) => t,
        Err(_) => {
            // TODO handle errors
            return Ok(());
        }
    };

    write_cargo_toml_target(&mut toml, target_architecture, target_mcu);
    fs::write(&file_path, toml.to_string())?;
    Ok(())
}

fn write_config_toml_content(
    toml: &mut DocumentMut,
    target_architecture: &String,
    target_mcu: &String,
) {
    write_cargo_toml_target(toml, target_architecture, target_mcu);

    let mut unstable = Table::new();
    let mut builds = Array::new();
    builds.push("core");
    unstable["build-std"] = value(builds);
    toml["unstable"] = Item::Table(unstable);
}

fn write_cargo_toml_target(
    toml: &mut DocumentMut,
    target_architecture: &String,
    target_mcu: &String,
) {
    let arch = target_architecture.to_ascii_lowercase();
    let mcu = target_mcu.to_ascii_lowercase();

    let mut build = Table::new();
    build["target"] = value(arch);
    let mut flags = Array::new();
    flags.push("-C");
    flags.push(format!("target-cpu={mcu}").as_str());
    build["rustflags"] = value(flags);
    toml["build"] = Item::Table(build);
}