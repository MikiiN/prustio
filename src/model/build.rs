use std::fs;
use std::path::PathBuf;

const BUILD_FILE_NAME: &str = "build.rs";

pub fn write_build_configuration(proj_path: &PathBuf) -> Result<(), String> {
    let build_file = proj_path.join(BUILD_FILE_NAME);
    match fs::write(build_file, get_build_content()) {
        Ok(_) => (),
        Err(_) => {
            return Err(String::from("Failed to write build.rs file."));
        }
    };

    Ok(())
}

fn get_build_content() -> String {
    String::from("fn main() {
    println!(\"cargo:rustc-link-search=native=lib\");
    println!(\"cargo:rustc-link-lib=static=FrameworkArduino\");
    println!(\"cargo:rustc-link-lib=static=FrameworkArduinoVariant\");
}")
}