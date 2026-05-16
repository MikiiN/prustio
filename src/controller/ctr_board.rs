//! Controller for displaying supported hardware boards.
//!
//! When a user runs the `prustio boards` command, this module handles fetching
//! the list of supported microcontrollers (either all of them or filtered by a 
//! search string) and formatting the output for the console.

use crate::model::board;
use crate::ui::display;

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
) -> Result<(), String> {
    let boards = board::get_boards(filter)?;

    if *json_output {
        display::print_boards_json(&boards);
    } else {
        display::print_boards_table(&boards);
    }
    Ok(())
}