use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};
use colored::Colorize;

use crate::model::board::Board;
use crate::model::device::PioDevice;

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