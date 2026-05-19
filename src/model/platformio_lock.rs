//! Manages the `platformio.lock` dependency tracking file.
//!
//! To ensure reproducible builds in hybrid mode, `pRustIO` captures the exact 
//! versions of all frameworks, libraries, and tools resolved by PlatformIO. 
//! This module parses the output of `pio pkg list`, categorizes the dependencies, 
//! and serializes them into a `platformio.lock` TOML file in the project directory.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const FILE_NAME: &str = "platformio.lock";

/// Represents the classification of a PlatformIO dependency.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Category {
    Platform,
    Framework,
    Tool,
    Library,
}

/// Represents a single pinned dependency.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    /// The name of the package.
    pub name: String,
    /// The version string.
    pub version: String,
    /// The classification of the package.
    pub category: Category,
}

/// The root structure representing the `platformio.lock` file.
#[derive(Debug, Serialize, Deserialize)]
pub struct Lockfile {
    /// The version of the lock file.
    pub version: u8,
    /// The list of all locked dependencies.
    pub dependencies: Vec<Dependency>,
}

impl Lockfile {
    /// Creates a new `Lockfile` instance in memory.
    pub fn new(dependencies: Vec<Dependency>, version: u8) -> Lockfile {
        Lockfile { version, dependencies }
    }

    /// Updates the existing dependencies and increments the lockfile version.
    /// 
    /// # Arguments
    /// * `dependencies` - The list of the current PlatformIO project dependencies. 
    pub fn update(&mut self, dependencies: Vec<Dependency>) {
        self.version += 1;
        self.dependencies = dependencies;
    }

    /// Loads and parses a `platformio.lock` file from the project directory.
    ///
    /// # Arguments
    /// * `proj_dir` - The root directory of the `pRustIO` project.
    ///
    /// # Errors
    /// Returns an error if the lockfile does not exist, cannot be read, or contains invalid TOML.
    pub fn load(proj_dir: &PathBuf) -> Result<Lockfile, String> {
        let lock_path = proj_dir.join(FILE_NAME);
        if !lock_path.exists() {
            return Err("PlatformIO lock file does not exists.".to_string());
        }

        if let Ok(contents) = fs::read_to_string(lock_path) {
            let data: Lockfile = match toml::from_str(&contents) {
                Ok(d) => d,
                Err(_) => {
                    return Err("Failed to parse PlatformIO lock file.".to_string());
                }
            };
            Ok(data)
        } else {
            Err("Failed to read PlatformIO lock file.".to_string())
        }
    }

    /// Formats the locked framework dependencies for injection into `platformio.ini`.
    ///
    /// Returns a list of strings formatted as `\n    <name> @ <version>`.
    pub fn get_platform_packages(&self) -> Vec<String> {
        self.dependencies
            .iter()
            .filter(|dep| dep.category == Category::Framework)
            .map(|dep| format!("\n    {} @ {}", dep.name, dep.version))
            .collect()
    }

    /// Formats the locked library dependencies for injection into `platformio.ini`.
    ///
    /// Returns a list of strings formatted as `\n    <name> @ <version>`.
    pub fn get_lib_deps(&self) -> Vec<String> {
        self.dependencies
            .iter()
            .filter(|dep| dep.category == Category::Library)
            .map(|dep| format!("/n    {} @ {}", dep.name, dep.version))
            .collect()
    }

    /// Serializes the lockfile to TOML and saves it to disk.
    ///
    /// # Arguments
    /// * `proj_dir` - The root directory of the `pRustIO` project.
    ///
    /// # Errors
    /// Returns an error string if serialization or file writing fails.
    pub fn save(&self, proj_dir: &PathBuf) -> Result<(), String> {
        let lock_path = proj_dir.join(FILE_NAME);
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize lockfile: {}", e))?;
        fs::write(lock_path, toml_string)
            .map_err(|e| format!("Failed to write lockfile: {}", e))
    }
}

/// Helper function to retrieve the standard path for the `platformio.lock` file.
pub fn get_pio_lock_path(proj_dir: &PathBuf) -> PathBuf { 
    proj_dir.join(FILE_NAME)
}

/// Parses the standard output of the `pio pkg list` command to extract dependency data.
///
/// This function uses a regular expression to match `<package_name> @ <version>` 
/// and categorizes the dependency based on contextual clues (like section headers 
/// or naming prefixes).
///
/// # Arguments
/// * `stdout` - The raw string output from the PlatformIO command line.
///
/// # Errors
/// Returns an error if the regular expression fails to compile or if critical 
/// capture groups are unexpectedly missing.
pub fn parse_pio_list_output(stdout: &str) -> Result<Vec<Dependency>, String> {
    let package_regex = match Regex::new(r"([a-zA-Z0-9\-_/]+)\s+@\s+([a-zA-Z0-9\.\-\+]+)") {
        Ok(regex) => regex,
        Err(_) => {
            return Err("Failed to parse regex expression".to_string());
        }
    };
    
    let mut locked_deps = Vec::new();
    let mut in_libraries_section = false;

    for line in stdout.lines() {
        // check if entered the libraries section
        if line.starts_with("Libraries") {
            in_libraries_section = true;
            continue;
        }

        // skip empty lines or headers
        if line.trim().is_empty() || !line.contains('@') {
            continue;
        }

        // extract the name and version
        if let Some(captures) = package_regex.captures(line) {
            let name = match captures.get(1) {
                Some(n) => n.as_str().to_string(),
                None => {
                    return Err("Internal error while getting name.".to_string());
                }
            };
            let version = match captures.get(2) {
                Some(v) => v.as_str().to_string(),
                None => {
                    return Err("Internal error while getting version.".to_string());
                }
            }; 

            // determine the category based on context and prefixes
            let category = if line.starts_with("Platform") {
                Category::Platform
            } else if name.starts_with("framework-") {
                Category::Framework
            } else if name.starts_with("tool-") || name.starts_with("toolchain-") {
                Category::Tool
            } else if in_libraries_section {
                Category::Library
            } else {
                // fallback
                Category::Library 
            };

            locked_deps.push(Dependency { name, version, category });
        }
    }

    Ok(locked_deps)
}


// 
// Unit Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pio_list_output() {
        let sample_output = "
Platform atmelavr @ 5.0.0
Libraries
├── SomeLib @ 1.2.3
└── tool-avrdude @ 1.0.0
";
        let deps = parse_pio_list_output(sample_output).unwrap();
        
        assert_eq!(deps.len(), 3);
        
        // Assert Platform extraction
        assert_eq!(deps[0].name, "atmelavr");
        assert_eq!(deps[0].version, "5.0.0");
        assert_eq!(deps[0].category, Category::Platform);
        
        // Assert Library extraction
        assert_eq!(deps[1].name, "SomeLib");
        assert_eq!(deps[1].version, "1.2.3");
        assert_eq!(deps[1].category, Category::Library);
        
        // Assert Tool extraction (prefix based)
        assert_eq!(deps[2].name, "tool-avrdude");
        assert_eq!(deps[2].category, Category::Tool);
    }
}