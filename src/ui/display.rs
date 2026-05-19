//! Handles console output formatting, colors, and tables.
//!
//! This module centralizes all visual feedback provided to the user. It uses 
//! `comfy-table` for rendering structured data and `colored` for semantic 
//! logging (errors, warnings, successes). It also handles JSON serialization 
//! for external tool integration.

use colored::Colorize;

/// Prints an unformatted message.
///
/// # Arguments
/// * `msg` - The message to print.
pub fn unformatted_print(msg: &String) {
    println!("{}", msg);
}

/// Prints a standard informational message to the console.
/// 
/// # Arguments 
/// * `msg` - The message to print.
pub fn info(msg: &str) {
    println!("{}", msg);
}

/// Prints a success message to the STDOUT in text or JSON format.
/// 
/// # Arguments 
/// * `msg` - The message to print.
/// * `json_output` - The JSON output flag.
pub fn success(msg: &str, json_output: &bool) {
    if *json_output {
        success_json(msg);
    } else {
        success_stdout(msg);
    }
}

/// Prints a success message to the console in green text.
/// 
/// # Arguments 
/// * `msg` - The message to print.
pub fn success_stdout(msg: &str) {
    println!("{}", msg.green());
}

/// Prints a success message to the console in JSON format.
/// 
/// # Arguments 
/// * `msg` - The message to print.
pub fn success_json(msg: &str) {
    let json_msg = serde_json::json!({
                    "status": "success",
                    "message": msg
                });
    println!("{}", json_msg);
}

/// Prints a error message to the STDERR or in the JSON format to the STDOUT.
/// 
/// # Arguments 
/// * `msg` - The message to print.
/// * `json_output` - The JSON output flag.
pub fn error(msg: &str, json_output: &bool) {
    if *json_output {
        error_json(msg);
    } else {
        error_stderr(msg);
    }
}

/// Prints an error message to standard error (`stderr`) in red text.
/// 
/// # Arguments 
/// * `msg` - The message to print.
pub fn error_stderr(msg: &str) {
    eprintln!("{} {}", "Error: ".red().bold(), msg.red());
}

/// Prints a error message to the console in JSON format.
/// 
/// # Arguments 
/// * `msg` - The message to print.
pub fn error_json(msg: &str) {
    let json_msg = serde_json::json!({
                    "status": "error",
                    "message": msg
                });
    println!("{}", json_msg);
}