//! Handles console output formatting, colors, and tables.
//!
//! This module centralizes all visual feedback provided to the user. It uses 
//! `comfy-table` for rendering structured data and `colored` for semantic 
//! logging (errors, warnings, successes). It also handles JSON serialization 
//! for external tool integration.

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};
use colored::Colorize;

use crate::model::board::Board;
use crate::model::device::PioDevice;

/// Prints a formatted, human-readable table of supported boards.
///
/// # Arguments
/// * `boards` - A vector of `Board` structs to display.
pub fn print_boards_table(boards: &Vec<Board>) {
    let mut table = Table::new();

    table.load_preset(UTF8_FULL)
         .apply_modifier(UTF8_ROUND_CORNERS)
         .set_header(vec![
            Cell::new("ID"),
            Cell::new("MCU"),
            Cell::new("Frequency"),
            Cell::new("Flash"),
            Cell::new("RAM"),
            Cell::new("Name"),
         ]);
    
    for board in boards {
        let fcpu_mhz = board.fcpu/1_000_000;
        let rom_kb = board.rom/1_000;
        let ram_kb = board.ram/1_000;

        table.add_row(vec![
            Cell::new(board.id.clone()).fg(Color::Cyan),
            Cell::new(board.mcu.clone()),
            Cell::new(format!("{} MHz", fcpu_mhz)),
            Cell::new(format!("{} KB", rom_kb)),
            Cell::new(format!("{} KB", ram_kb)),
            Cell::new(board.name.clone()),
        ]);
    }

    println!("{table}");
    println!("{}", format!("Found {} boards.", table.row_iter().count()).green());
}

/// Prints a formatted list of connected serial devices.
///
/// # Arguments
/// * `devices` - A vector of `PioDevice` structs representing connected hardware.
pub fn print_devices_table(devices: &Vec<PioDevice>) {
    for device in devices {
        let hwid = match &device.hwid {
            Some(id) => id.clone(),
            None => "n/a".to_string()
        };
        let description = match &device.description {
            Some(desc) => desc.clone(),
            None => "n/a".to_string()
        };
        println!("{}", device.port);
        println!("----------------");
        println!("Hardware ID: {}", hwid);
        println!("Description: {}\n", description);
    }
}

/// Prints the supported boards as a raw JSON string.
/// 
/// # Arguments
/// * `boards` - The list of boards.
pub fn print_boards_json(boards: &Vec<Board>) {
    let json_string = match serde_json::to_string_pretty(boards) {
        Ok(s) => s,
        Err(_) => {
            println!("Error: Failed to parse boards to the json format.");
            return;
        }
    };
    println!("{}", json_string);
}

/// Prints the connected devices as a raw JSON string.
/// 
/// # Arguments
/// * `devices` - The list of devices.
pub fn print_devices_json(devices: &Vec<PioDevice>) {
    let json_string = match serde_json::to_string_pretty(devices) {
        Ok(json) => json,
        Err(_) => {
            println!("Error: Failed to parse devices to the json format.");
            return;
        }
    };
    println!("{}", json_string);
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