use std::fmt;
use std::fs;
use serde::{Deserialize, Serialize};

use crate::wrapper::platformio;

pub const UNSPECIFIED_PARAM: &str = "UNSPECIFIED";
const UNSPECIFIED_RUSTC_VERSION: &str = "nightly-2025-04-27";
const PLATFORMS_DIR: &str = "platforms";
const BOARDS_DIR: &str = "boards";

const SUPPORTED_BOARD_IDS: &[&str] = &["uno"];

#[derive(Debug, Serialize)]
pub struct Board {
    pub id: String,
    pub mcu: String,
    pub platform: Platform,
    pub cargo_feature: String,
    pub bus_speed: u32,
    pub upload_protocol: String,
    pub rustc_version: String,
    pub fcpu: u32,
    pub ram: u32,
    pub rom: u32,
    pub name: String,
}

impl Board {
    pub fn new(
        id: &str, 
        cargo_feature: &str, 
        rustc_version: &str,
    ) -> Result<Board, String> {
        let board = get_pio_board(id)?;
        let upload_config = get_pio_upload_config(id, &board.platform)?;

        Ok(Board {
            id: board.id, 
            mcu: board.mcu, 
            platform: Platform::from(board.platform.as_str()), 
            cargo_feature: String::from(cargo_feature), 
            bus_speed: upload_config.speed, 
            upload_protocol: upload_config.protocol, 
            rustc_version: String::from(rustc_version),
            fcpu: board.fcpu,
            ram: board.ram,
            rom: board.rom,
            name: board.name,
        })
    }

    pub fn new_custom(
        id: &str, 
        mcu: &str,
        platform: &str,
        cargo_feature: &str, 
        bus_speed: u32,
        upload_protocol: &str,
        rustc_version: &str,
        fcpu: u32,
        ram: u32,
        rom: u32,
        name: &str,
    ) -> Board {
        Board {
            id: String::from(id), 
            mcu: String::from(mcu), 
            platform: Platform::from(platform), 
            cargo_feature: String::from(cargo_feature), 
            bus_speed: bus_speed, 
            upload_protocol: String::from(upload_protocol), 
            rustc_version: String::from(rustc_version),
            fcpu: fcpu,
            ram: ram,
            rom: rom,
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum Platform {
    ATMELAVR,
    UNKNOWN,
}

impl Platform {
    pub fn from(value: &str) -> Platform {
        match value {
            "atmelavr" => Platform::ATMELAVR,
            _ => Platform::UNKNOWN
        }
    }

    pub fn to_string(&self) ->String {
        match self {
            Self::ATMELAVR => "atmelavr".to_string(),
            Self::UNKNOWN => UNSPECIFIED_PARAM.to_string()
        }
    }

    pub fn to_cargo_arch(&self) -> String {
        match self {
            Self::ATMELAVR => String::from("avr-none"),
            Self::UNKNOWN => UNSPECIFIED_PARAM.to_string()
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Platform::ATMELAVR => write!(f, "atmelavr"),
            Platform::UNKNOWN => write!(f, "Unknown"),
        }
    }
}


/*
 ------------------------------- 
    PIO board output parsing
 ------------------------------- 
*/

#[derive(Debug, Deserialize, Clone)]
struct PioBoard {
    id: String,
    mcu: String,
    fcpu: u32,
    ram: u32,
    rom: u32,
    name: String,
    platform: String,
}

fn get_pio_board(id: &str) -> Result<PioBoard, String> {
    match get_pio_boards(id) {
        Ok(boards) => {
            if boards.is_empty() {
                return Err(String::from("Invalid board ID"));
            }
            let best_match = boards[0].clone();
            if best_match.id == id {
                return Ok(best_match);
            }
            return Err(String::from("Invalid board ID"));
        },
        Err(e) => { return Err(e) },
    }
}

fn get_pio_boards(filter: &str) -> Result<Vec<PioBoard>, String> {
    let result = platformio::get_boards(filter);
    match result {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            match serde_json::from_str::<Vec<PioBoard>>(&output_str) {
                Ok(boards) => { return Ok(boards); },
                Err(_) => { 
                    return Err(String::from("Failed to parse JSON from platformIO.")); 
                }
            }
        },
        Err(_) => {
            return Err(String::from("PlatformIO failed to find the specified board"));
        }
    }
}

/*
 ------------------------------- 
    PIO upload config parsing
 ------------------------------- 
*/

#[derive(Deserialize, Debug)]
struct PioBoardManifest {
    name: String,
    upload: PioUploadConfig,
    build: Option<PioBuildConfig>,
}

#[derive(Deserialize, Debug)]
struct PioUploadConfig {
    speed: u32, 
    protocol: String,
}

#[derive(Deserialize, Debug)]
struct PioBuildConfig {
    mcu: Option<String>,
}

fn get_pio_upload_config(board_id: &str, platform: &str) -> Result<PioUploadConfig, String> {
    let (_, core_dir) = platformio::get_pio_dirs()?;
    let confs_path = core_dir.join(PLATFORMS_DIR).join(platform).join(BOARDS_DIR);
    if !confs_path.exists() {
        platformio::download_pio_platform(platform)?;
    }
    
    let board_path = confs_path.join(format!("{board_id}.json"));
    if !board_path.exists() {
        return Err(String::from("Unknown board ID."));
    }
    let file_contents = match fs::read_to_string(&board_path) {
        Ok(s) => s,
        Err(_) => {
            return Err(String::from("Failed to read configuration file."));
        }
    };

    let manifest: PioBoardManifest = match serde_json::from_str(&file_contents) {
        Ok(json) => json,
        Err(_) => {
            return Err(String::from("Failed to parse board configuration."));
        }
    };
    
    Ok(manifest.upload)
}

/* 
 -------------------------------
    obtaining supported board
 ------------------------------- 
*/

pub fn get_board(id: &str) -> Result<Board, String> {
    let board = match id {
        "uno" => Board::new(id, "arduino-uno", "nightly-2025-04-27")?,
        _ => {
            return Err("Unsupported board ID.".to_string());
        }
    };
    Ok(board)
}

pub fn get_boards(filter: Option<&String>) -> Vec<Board> {
    let board_ids = match filter {
        Some(f) => get_filtered_boards(f),
        None => Vec::from(SUPPORTED_BOARD_IDS)
    };
    
    let mut boards = Vec::new();
    for id in board_ids {
        match get_board(id) {
            Ok(b) => boards.push(b),
            Err(_) => {}
        }
    }
    boards
}

fn get_filtered_boards(filter: &String) -> Vec<&str> {
    let parsed_filter = filter.to_lowercase();
    let board_ids: Vec<&str> = SUPPORTED_BOARD_IDS.iter()
                                       .copied()
                                       .filter(|board_id| {
                                            board_id.to_lowercase().contains(&parsed_filter)
                                       })
                                       .collect();
    board_ids
}

pub fn get_unspecified_board() -> Board {
    Board::new_custom(
        UNSPECIFIED_PARAM, 
        UNSPECIFIED_PARAM, 
        UNSPECIFIED_PARAM, 
        UNSPECIFIED_PARAM, 
        0,
        UNSPECIFIED_PARAM, 
        UNSPECIFIED_RUSTC_VERSION, 
        0,
        0,
        0,
        UNSPECIFIED_PARAM,
    )
}