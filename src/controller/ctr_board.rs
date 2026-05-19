//! Controller for displaying supported hardware boards.
//!
//! When a user runs the `prustio boards` command, this module handles fetching
//! the list of supported microcontrollers (either all of them or filtered by a 
//! search string) and formatting the output for the console.

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};

use crate::model::board;

/// Fetches and displays the list of supported microcontroller boards.
///
/// This function acts as the bridge between the board data model and the UI 
/// display logic. It retrieves the requested boards and prints them either as 
/// a human-readable table or as a raw JSON string.
///
/// # Arguments
/// * `filter` - An optional string to filter the boards by name or ID. If `None`, 
///   all statically supported boards are returned.
/// * `json_output` - If `true`, prints the output as a JSON array instead of a formatted table. 
///
/// # Errors
/// Returns an error when an invalid board ID occurs.
pub fn board(
    filter: Option<&String>,
    json_output: &bool,
) -> Result<String, String> {
    let boards = board::get_boards(filter)?;

    if *json_output {
        Ok(format_boards_json(&boards)?)
    } else {
        Ok(format_boards_table(&boards)?)
    }
}
/// Formats the vector of boards to JSON format.
///
/// # Arguments
/// * `boards` - A vector of `Board` structs.
/// 
/// # Errors
/// Returns an error, if formatting fails.
fn format_boards_json(boards: &Vec<board::Board>) -> Result<String, String> {
    let json_string = match serde_json::to_string_pretty(boards) {
        Ok(s) => s,
        Err(_) => {
            return Err("Failed to parse boards to the json format.".to_string());
        }
    };
    Ok(json_string)
}

/// Formats the vector of boards to the table string.
///
/// # Arguments
/// * `boards` - A vector of `Board` structs to format.
/// 
/// # Errors
/// Returns an error, if formatting fails.
fn format_boards_table(boards: &Vec<board::Board>) -> Result<String, String> {
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

    Ok(table.to_string())
}