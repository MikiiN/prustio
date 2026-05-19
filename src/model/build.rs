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

//
// Unit Tests
//

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_get_build_content_empty_libs() {
        let lib_dir = PathBuf::from("/mock/built_libs");
        let lib_names = vec![];
        
        let content = get_build_content(&lib_dir, &lib_names);
        
        assert!(content.contains(&format!("cargo:rustc-link-search=native={}", lib_dir.display())));
        assert!(content.contains(&format!("cargo:rustc-link-arg={}/wrapper.cpp.o", lib_dir.display())));
        
        assert!(content.contains("cargo:rustc-link-arg=-lc"));
        assert!(content.contains("cargo:rustc-link-arg=-lm"));
        assert!(content.contains("cargo:rustc-link-arg=-lgcc"));
    }

    #[test]
    fn test_get_build_content_with_libs() {
        let lib_dir = PathBuf::from("/mock/built_libs");
        let lib_names = vec!["FrameworkArduino".to_string(), "Wire".to_string()];
        
        let content = get_build_content(&lib_dir, &lib_names);
        
        assert!(content.contains("cargo:rustc-link-arg=-lFrameworkArduino"));
        assert!(content.contains("cargo:rustc-link-arg=-lWire"));
        
        assert!(content.contains("cargo:rustc-link-arg=-lc"));
    }

    #[test]
    fn test_write_build_configuration_success() {
        let temp_dir = tempdir().unwrap();
        let proj_path = temp_dir.path().to_path_buf();
        let lib_names = vec!["custom_lib".to_string()];

        // write configuration
        let result = write_build_configuration(&proj_path, &lib_names);
        assert!(result.is_ok());

        // verify the file was created
        let build_file_path = proj_path.join(BUILD_FILE_NAME);
        assert!(build_file_path.exists());

        // verify the file contains expected content
        let content = fs::read_to_string(build_file_path).expect("Could not read generated build.rs");
        assert!(content.contains("cargo:rustc-link-arg=-lcustom_lib"));
        
        // verify .prio directory was created internally by get_project_app_dir
        assert!(proj_path.join(crate::utils::PROJECT_APP_DIR_NAME).exists());
    }

    #[test]
    fn test_write_build_configuration_failure() {
        // provide an invalid root path to trigger an error
        let proj_path = PathBuf::from("/invalid/path/that/does/not/exist");
        let lib_names = vec!["custom_lib".to_string()];

        let result = write_build_configuration(&proj_path, &lib_names);
        
        // should fail, because it can't create .prio or because it can't write build.rs
        assert!(result.is_err());
    }
}