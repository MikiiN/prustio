use std::fs;

use serde::Deserialize;

use crate::wrapper::platformio;

pub const UNSPECIFIED_BOARD_PARAM: &str = "UNSPECIFIED";
const PLATFORMS_DIR: &str = "platforms";
const ATMELAVR_BOARDS_DIR: &str = "atmelavr/boards";

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

#[derive(Deserialize, Debug)]
struct BoardManifest {
    name: String,
    upload: UploadConfig,
    build: Option<BuildConfig>,
}

#[derive(Deserialize, Debug)]
pub struct UploadConfig {
    pub speed: u32, 
    pub protocol: String,
}

#[derive(Deserialize, Debug)]
struct BuildConfig {
    mcu: Option<String>,
}

// TODO make general (now only support atmel AVR)
pub fn get_upload_config(board_id: &String) -> Result<UploadConfig, String> {
    let (_, core_dir) = platformio::get_pio_dirs()?;
    let confs_path = core_dir.join(PLATFORMS_DIR).join(ATMELAVR_BOARDS_DIR);
    if !confs_path.exists() {
        platformio::download_pio_platform("atmelavr")?;
    }
    
    let board_path = confs_path.join(format!("{board_id}.json"));
    if !board_path.exists() {
        return Err(String::from("Unknown board ID."));
    }
    let file_contents = fs::read_to_string(&board_path)
            .expect("Failed to read board JSON file");

    let manifest: BoardManifest = serde_json::from_str(&file_contents)
            .expect("Failed to parse board JSON");
    Ok(manifest.upload)
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
    let result = platformio::get_boards(filter);
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
