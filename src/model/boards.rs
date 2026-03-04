use serde::Deserialize;
use crate::wrapper;

pub const UNSPECIFIED_BOARD_PARAM: &str = "UNSPECIFIED";

#[derive(Debug, Deserialize, Clone)]
pub struct Board {
    pub id: String,
    pub mcu: String,
    platform: String,
}

impl Board {
    pub fn new(id: String, mcu: String, platform: String) -> Board {
        Board { id, mcu, platform }
    }

    pub fn get_architecture(&self) -> Option<String> {
        match self.platform.as_str() {
            "atmelavr" => Some(String::from("avr-none")),
            UNSPECIFIED_BOARD_PARAM => Some(String::from(UNSPECIFIED_BOARD_PARAM)),
            _ => None,
        }
    }

    pub fn get_cargo_feature(&self) -> Option<String> {
        match self.id.as_str() {
            "uno" => Some(String::from("arduino-uno")),
            UNSPECIFIED_BOARD_PARAM => Some(String::from(UNSPECIFIED_BOARD_PARAM)),
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
                Err(_) => { return Err("Failed to parse JSON from platformIO."); }
            }
        },
        Err(_) => {
            return Err("PlatformIO failed to find the specified board");
        }
    }
}

pub fn get_unspecified_board() -> Board {
    Board::new(
        String::from(UNSPECIFIED_BOARD_PARAM),
        String::from(UNSPECIFIED_BOARD_PARAM),
        String::from(UNSPECIFIED_BOARD_PARAM)
    )
}
