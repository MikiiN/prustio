//! Generates the `build.rs` build script for the Rust project.
//!
//! When running in hybrid mode, the Rust application must link against the 
//! Arduino framework libraries compiled by PlatformIO. This module generates
//! a `build.rs` file that tells Cargo exactly where to find those compiled  
//! archives (`.a` files) and object files (`.o`), and instructs the rustc 
//! linker to include them.

use std::fs;
use std::path::PathBuf;

use crate::utils::{
    get_project_app_dir,
    COMPILED_LIBS_DIR_NAME
};

const BUILD_FILE_NAME: &str = "build.rs";

/// Writes the `build.rs` configuration file to the project's root directory.
///
/// This function calculates the path to the internal `.prio/builded_libs` directory
/// and generates a build script that instructs Cargo to link the specified libraries.
///
/// # Arguments
/// * `proj_path` - The path of the pRustIO project.
/// * `lib_names` - A list of C/C++ library names (without the `lib` prefix or `.a` extension) 
///   that were compiled by PlatformIO and need to be linked.
///
/// # Errors
/// Returns an error string if the `build.rs` file cannot be written to disk.
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

/// Generates the raw Rust source code string for the `build.rs` file.
/// 
/// The generated script uses standard `cargo:` instructions printed to standard output. 
/// It specifies the native library search path, links the compiled `wrapper.cpp.o` object file,
/// links each user-specified library, and ensures standard C/math libraries (`libc`, `libm`, `libgcc`) 
/// are also linked.
/// 
/// # Arguments
/// * `lib_dir` - The path of directory containing compiled libraries.
/// * `lib_names` - The vector of libraries names
fn get_build_content(lib_dir: &PathBuf, lib_names: &Vec<String>) -> String {
    let mut content = String::from("fn main() {\n");
    content += format!("    println!(\"cargo:rustc-link-search=native={}\");\n", lib_dir.display()).as_str();

    content += format!("    println!(\"cargo:rustc-link-arg={}/wrapper.cpp.o\");", lib_dir.display()).as_str();

    for name in lib_names {
        content += format!("\n    println!(\"cargo:rustc-link-arg=-l{}\");", name).as_str();
    }

    content += "\n    println!(\"cargo:rustc-link-arg=-lc\");";  
    content += "\n    println!(\"cargo:rustc-link-arg=-lm\");";
    content += "\n    println!(\"cargo:rustc-link-arg=-lgcc\");";

    content += "\n}";

    content
}