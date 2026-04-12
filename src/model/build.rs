use std::fs;
use std::path::PathBuf;

use crate::utils::{
    get_project_app_dir,
    COMPILED_LIBS_DIR_NAME
};

const BUILD_FILE_NAME: &str = "build.rs";

pub fn write_build_configuration(proj_path: &PathBuf, lib_names: &Vec<String>) -> Result<(), String> {
    let build_file = proj_path.join(BUILD_FILE_NAME);
    let libs_dir = get_project_app_dir(proj_path)?.join(COMPILED_LIBS_DIR_NAME);
    match fs::write(build_file, get_build_content(&libs_dir, lib_names)) {
        Ok(_) => (),
        Err(_) => {
            return Err(String::from("Failed to write build.rs file."));
        }
    };

    Ok(())
}

fn get_build_content(lib_dir: &PathBuf, lib_names: &Vec<String>) -> String {
    let mut content = String::from("fn main() {\n");
    content += format!("    println!(\"cargo:rustc-link-search=native={}\");\n", lib_dir.display()).as_str();

    content += format!("    println!(\"cargo:rustc-link-arg={}/wrapper.cpp.o\");", lib_dir.display()).as_str();

    for name in lib_names {
        content += format!("\n    println!(\"cargo:rustc-link-arg=-l{}\");", name).as_str();
    }

    content += "\n}";

    content
}