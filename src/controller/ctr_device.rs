//! Controller for managing connected hardware devices.
//!
//! This module provides the logic for the `prustio device` subcommand, including
//! listing available microcontrollers and launching a PlatformIO interactive serial 
//! monitor to communicate with the hardware.

use std::env;

use crate::model::device;
use crate::ui::display;
use crate::ui::device::{Parity, EOL};
use crate::wrapper::platformio;

/// Fetches and displays a list of connected serial devices.
///
/// This function queries the host system for attached hardware and prints
/// the list to the console.
///
/// # Arguments
/// * `json_output` - If `true`, prints the output as a JSON string instead of a human-readable table.
pub fn device_list(json_output: &bool) {
    let devices = match device::get_port_list() {
        Ok(list) => list,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };
    if *json_output {
        display::print_devices_json(&devices);
    } else {
        display::print_devices_table(&devices);
    }
}

/// Launches a serial monitor for the connected device.
///
/// This function acts as a wrapper around the PlatformIO serial monitor, passing
/// all user-specified terminal configuration options (baud rate, parity, etc.) 
/// down to the `pio device monitor` command.
///
/// # Arguments
/// * `port` - The specific serial port to connect to.
/// * `baud` - The baud rate for the connection.
/// * `parity` - The parity checking mode.
/// * `rtscts` - Enable RTS/CTS hardware flow control.
/// * `xonxoff` - Enable software flow control.
/// * `rts` - Initial state of the RTS line (0 or 1).
/// * `dtr` - Initial state of the DTR line (0 or 1).
/// * `echo` - Enable local echo in the terminal.
/// * `encoding` - Set the terminal encoding.
/// * `filter` - Apply a PlatformIO output filter.
/// * `eol` - Set the End-Of-Line character sequence.
/// * `raw` - Do not apply any encodings or transformations.
/// * `exit_char` - The ASCII code for the exit shortcut.
/// * `menu_char` - The ASCII code for the menu shortcut.
/// * `quiet` - Suppress diagnostic outputs from the monitor.
/// * `no_reconnect` - Do not attempt to automatically reconnect if the port drops.
///
/// # Errors
/// Returns an error string if:
/// * The current working directory cannot be determined.
/// * The PlatformIO monitor exits with an error status.
pub fn device_monitor(
    port: &Option<String>, 
    baud: &Option<u32>, 
    parity: &Option<Parity>, 
    rtscts: &bool, 
    xonxoff: &bool, 
    rts: &Option<u8>, 
    dtr: &Option<u8>, 
    echo: &bool, 
    encoding: &Option<String>, 
    filter: &Option<String>, 
    eol: &Option<EOL>, 
    raw: &bool, 
    exit_char: &Option<u8>, 
    menu_char: &Option<u8>, 
    quiet: &bool, 
    no_reconnect: &bool 
) ->Result<(), String> {
    let proj_path = match env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            return Err("Error: Failed to get current working directory.".to_string());
        },
    };

    platformio::device_monitor(
        &proj_path, port, baud, parity, rtscts, xonxoff, rts, dtr,
        echo, encoding, filter, eol, raw, exit_char, menu_char, quiet, no_reconnect
    )?;
    Ok(())
}