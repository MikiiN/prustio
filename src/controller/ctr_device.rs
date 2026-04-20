use std::env;

use crate::model::device;
use crate::ui::display;
use crate::ui::device::{Parity, EOL};
use crate::wrapper::platformio;

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
) {
    let proj_path = match env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            eprintln!("Error: Failed to get current working directory.");
            return;
        },
    };

    let result = platformio::device_monitor(
        &proj_path, port, baud, parity, rtscts, xonxoff, rts, dtr,
        echo, encoding, filter, eol, raw, exit_char, menu_char, quiet, no_reconnect
    );
}