use serde::Deserialize;
use crate::wrapper;

#[derive(Debug, Deserialize, Clone)]
pub struct Board {
    pub id: String,
    pub mcu: String,
    platform: String,
}

impl Board {
    pub fn get_architecture(&self) -> Option<String> {
        match self.platform.as_str() {
            "atmelavr" => Some(String::from("avr-none")),
            _ => None,
        }
    }

    pub fn get_cargo_feature(&self) -> Option<&str> {
        match self.id.as_str() {
            "uno" => Some("arduino-uno"),
            _ => None,
        }
    }
}

pub fn get_board(id: &str) -> Result<Board, &str> {
    match get_boards(id) {
        Ok(boards) => {
            if boards.is_empty() {
                return Err("Invalid board ID");
            }
            let best_match = boards[0].clone();
            if best_match.id == id {
                return Ok(best_match);
            }
            return Err("Invalid board ID");
        },
        Err(e) => { return Err(e) },
    }
}

pub fn get_boards(filter: &str) -> Result<Vec<Board>, &str> {
    let result = wrapper::platformio::get_boards(filter);
    match result {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            match serde_json::from_str::<Vec<Board>>(&output_str) {
                Ok(boards) => { return Ok(boards); },
                Err(_) => { return Err("Failed to parse JSON"); }
            }
        },
        Err(_) => {
            return Err("PIO error");
        }
    }
}
