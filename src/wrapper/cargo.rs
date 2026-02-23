use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio, ExitStatus};
use toml_edit::{DocumentMut, Item, Table, Array, ArrayOfTables, value};

const CONFIGURATION_DIR_NAME: &str = ".cargo";
const CONFIGURATION_FILE_NAME: &str = "config.toml";
const CARGO_TOML_FILE_NAME: &str = "Cargo.toml";
const RUST_TOOLCHAIN_FILE_NAME: &str = "rust-toolchain.toml";

pub fn init_cargo(proj_path: &String) -> std::io::Result<ExitStatus> {
    let mut cmd = Command::new("cargo");
    cmd.arg("init").arg(proj_path);

    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    return status;
}

pub fn create_cargo_config(
    proj_path: &String,
    target_architecture: &String,
    target_mcu: &String,
) -> std::io::Result<()> {
    let dir_path: PathBuf = PathBuf::from(proj_path).join(CONFIGURATION_DIR_NAME);
    fs::create_dir_all(&dir_path)?;

    let file_path = dir_path.join(CONFIGURATION_FILE_NAME);
    let mut toml = DocumentMut::new();

    get_config_toml_content(&mut toml, target_architecture, target_mcu);
    
    fs::write(file_path, toml.to_string())?;
    Ok(())
}

pub fn create_toolchain_config(proj_path: &String) -> std::io::Result<()> {
    let file_path = PathBuf::from(proj_path).join(RUST_TOOLCHAIN_FILE_NAME);
    let mut toml = DocumentMut::new();

    get_toolchain_content(&mut toml);

    fs::write(&file_path, toml.to_string())?;
    Ok(())
}

pub fn create_cargo_toml_config(proj_path: &String) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(proj_path).join(CARGO_TOML_FILE_NAME);
    let cargo_content = fs::read_to_string(&file_path)?;
    let mut toml = cargo_content.parse::<DocumentMut>()?;

    get_cargo_toml_content(&mut toml);

    fs::write(&file_path, toml.to_string())?;
    Ok(())
}

fn get_cargo_toml_content(toml: &mut DocumentMut) {
    // get project name from cargo.toml
    let proj_name = match toml["package"]["name"].as_str() {
        Some(name) => name,
        None => "project",
    };

    // [bin]
    let mut bin_table = Table::new();
    bin_table["name"] = value(proj_name);
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
    features.push("arduino-uno");
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

fn get_toolchain_content(toml: &mut DocumentMut) {
    let mut toolchain = Table::new();
    toolchain["channel"] = value("nightly-2025-04-27");
    let mut components = Array::new();
    components.push("rust-src");
    toolchain["components"] = value(components);
    toolchain["profile"] = value("minimal");
    toml["toolchain"] = Item::Table(toolchain);    
}

fn get_config_toml_content(
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

    let mut unstable = Table::new();
    let mut builds = Array::new();
    builds.push("core");
    unstable["build-std"] = value(builds);
    toml["unstable"] = Item::Table(unstable);
}