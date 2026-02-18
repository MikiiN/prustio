use std::fs;
use std::io::Write;
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
    let mut file = fs::File::create(file_path)?; 

    let content = get_config_toml_content(target_architecture, target_mcu);
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn create_toolchain_config(proj_path: &String) -> std::io::Result<()> {
    let file_path = PathBuf::from(proj_path).join(RUST_TOOLCHAIN_FILE_NAME);
    let mut file = fs::File::create(&file_path)?;
    let content = get_toolchain_content();
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn init_cargo_toml_config(proj_path: &String) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(proj_path).join(CARGO_TOML_FILE_NAME);
    let cargo_content = fs::read_to_string(&file_path)?;
    let mut toml = cargo_content.parse::<DocumentMut>()?;

    let mut bin_table = Table::new();
    bin_table["name"] = value("teplate-proj");
    bin_table["test"] = value(false);
    bin_table["bench"] = value(false);

    let mut bin_array = ArrayOfTables::new();
    bin_array.push(bin_table);
    toml["bin"] = Item::ArrayOfTables(bin_array);

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

    fs::write(&file_path, toml.to_string())?;
    Ok(())
}

fn get_toolchain_content() -> String {
    String::from(
"[toolchain]
channel = \"nightly-2025-04-27\"
components = [\"rust-src\"]
profile = \"minimal\"
")
}

fn get_config_toml_content(
    target_architecture: &String,
    target_mcu: &String,
) -> String {
    let arch = target_architecture.to_ascii_lowercase();
    let mcu = target_mcu.to_ascii_lowercase();
    format!("[build]
target = \"{arch}\"
rustflags = [\"-C\", \"target-cpu={mcu}\"]

[unstable]
build-std = [\"core\"]
")
}