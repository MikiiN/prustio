//! Defines hardware board targets and PlatformIO manifest parsing.
//!
//! This module holds the definitions for supported microcontrollers, their memory 
//! constraints, clock speeds, and upload configurations. It uses PlatformIO's Core CLI 
//! in the background to dynamically fetch detailed board specifications.

use std::fmt;
use std::fs;
use serde::{Deserialize, Serialize};

use crate::wrapper::platformio;

pub const UNSPECIFIED_PARAM: &str = "UNSPECIFIED";
const UNSPECIFIED_RUSTC_VERSION: &str = "nightly-2025-04-27";
const PLATFORMS_DIR: &str = "platforms";
const BOARDS_DIR: &str = "boards";

/// A static list of currently supported Arduino/AVR board identifiers.
const SUPPORTED_BOARD_IDS: &[&str] = &[
    "diecimilaatmega168",
    "leonardo",
    "ATmega2560",
    "ATmega1280",
    "nanoatmega328",
    "nanoatmega328new",
    "uno",
    "micro",
    "sparkfun_promicro8",
    "nanoatmega168",
];

/// Represents a supported microcontroller board and its build configurations.
///
/// This struct holds all the necessary data to configure Cargo, PlatformIO, 
/// and avrdude for a specific piece of hardware.
#[derive(Debug, Serialize)]
pub struct Board {
    /// The unique PlatformIO identifier for the board
    pub id: String,
    /// The target microcontroller 
    pub mcu: String,
    /// The hardware platform category
    pub platform: Platform,
    /// The specific arduino-hal feature flag
    pub cargo_feature: String,
    /// The bus speed for binary upload
    pub bus_speed: u32,
    /// The protocol used for uploading binary
    pub upload_protocol: String,
    /// The rust compiler supported version
    pub rustc_version: String,
    /// The mcu frequency
    pub fcpu: u32,
    /// The RAM size
    pub ram: u32,
    /// The ROM size
    pub rom: u32,
    /// The name of the board
    pub name: String,
}

impl Board {
    /// Creates a new `Board` by fetching its hardware specifications from PlatformIO.
    ///
    /// This function calls PlatformIO to dynamically retrieve CPU, memory, and upload
    /// configuration limits based on the provided `id`.
    ///
    /// # Arguments
    /// * `id` - The PlatformIO board identifier.
    /// * `cargo_feature` - The corresponding feature flag for `arduino-hal`.
    /// * `rustc_version` - The Rust nightly toolchain version to use.
    ///
    /// # Errors
    /// Returns an error if PlatformIO fails to find the board or parse its manifest.
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

    /// Creates a new custom `Board` without querying PlatformIO.
    ///
    /// This is typically used for generating a fallback or "unspecified" board configuration.
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

/// Represents the broader hardware architecture platform.
#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum Platform {
    ATMELAVR,
    UNKNOWN,
}

impl Platform {
    /// Parses a string into a `Platform` enum variant.
    pub fn from(value: &str) -> Platform {
        match value {
            "atmelavr" => Platform::ATMELAVR,
            _ => Platform::UNKNOWN
        }
    }

    /// Returns string representation of the platform.
    pub fn to_string(&self) ->String {
        match self {
            Self::ATMELAVR => "atmelavr".to_string(),
            Self::UNKNOWN => UNSPECIFIED_PARAM.to_string()
        }
    }

    /// Returns the Rust target architecture string used in cargo project configuration.
    pub fn to_cargo_arch(&self) -> String {
        match self {
            Self::ATMELAVR => String::from("avr-none"),
            Self::UNKNOWN => UNSPECIFIED_PARAM.to_string()
        }
    }

    /// Returns the Platformio C++ framework for architecture. Currently tool support a single
    /// architecture.
    pub fn to_framework(&self) -> String {
        match self {
            _ => String::from("arduino"),
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

/// Internal struct for deserializing board summary data from `pio boards`.
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

/// Queries PlatformIO for a specific board ID and returns its parsed data.
///
/// # Arguments
/// * `id` - The PlatformIO board ID.
/// 
/// # Errors
/// Returns an error if the board ID is invalid or PlatformIO execution fails.
fn get_pio_board(id: &str) -> Result<PioBoard, String> {
    match get_pio_boards(id) {
        Ok(boards) => {
            if boards.is_empty() {
                return Err("Unsupported board ID".to_string());
            }
            for board in &boards {
                if board.id == id {
                    return Ok(board.clone());
                }
            }

            let best_match = boards[0].clone();
            return Ok(best_match);
        },
        Err(e) => { return Err(e) },
    }
}

/// Queries PlatformIO for list of boards matching filter string.
///
/// # Arguments
/// * `filter` - The string for boards filtering.
/// 
/// # Errors
/// Returns an error if the PlatformIO execution fails.
fn get_pio_boards(filter: &str) -> Result<Vec<PioBoard>, String> {
    let result = platformio::get_boards(filter)?;
    match serde_json::from_str::<Vec<PioBoard>>(result.as_str()) {
        Ok(boards) => Ok(boards),
        Err(_) =>  Err("Failed to parse JSON from platformIO.".to_string())
    }
}

/*
 ------------------------------- 
    PIO upload config parsing
 ------------------------------- 
*/

/// Internal struct representing a PlatformIO board manifest JSON file.
#[derive(Deserialize, Debug)]
struct PioBoardManifest {
    name: String,
    upload: PioUploadConfig,
    build: Option<PioBuildConfig>,
}

/// Internal struct representing the upload specifications for a board.
#[derive(Deserialize, Debug)]
struct PioUploadConfig {
    speed: u32, 
    protocol: String,
}

/// Internal struct representing the build specifications for a board.
#[derive(Deserialize, Debug)]
struct PioBuildConfig {
    mcu: Option<String>,
}

/// Reads the PlatformIO JSON manifest for a specific board to extract its upload configuration.
///
/// If the required platform files are missing locally, it triggers a download via PlatformIO.
///
/// # Arguments
/// * `board_id` - The board ID.
/// * `platform` - The platform on which board is built.
/// 
/// # Errors
/// Returns an error if the board manifest file cannot be found, read, or parsed.
fn get_pio_upload_config(board_id: &str, platform: &str) -> Result<PioUploadConfig, String> {
    // download platform's configurations if missing
    let (_, core_dir) = platformio::get_pio_dirs()?;
    let confs_path = core_dir.join(PLATFORMS_DIR).join(platform).join(BOARDS_DIR);
    if !confs_path.exists() {
        platformio::download_pio_platform(platform)?;
    }
    
    // get specific configuration
    let board_path = confs_path.join(format!("{board_id}.json"));
    if !board_path.exists() {
        return Err("Unsupported board identifier.".to_string());
    }
    let file_contents = match fs::read_to_string(&board_path) {
        Ok(s) => s,
        Err(_) => {
            return Err("Failed to read board's configuration file.".to_string());
        }
    };

    // parse configuration to struct
    let manifest: PioBoardManifest = match serde_json::from_str(&file_contents) {
        Ok(json) => json,
        Err(_) => {
            return Err("Failed to parse board's configuration file.".to_string());
        }
    };
    
    Ok(manifest.upload)
}

/* 
 -------------------------------
    obtaining supported board
 ------------------------------- 
*/

/// Retrieves the complete `Board` configuration for a supported ID.
///
/// This maps predefined board IDs to their respective `arduino-hal` Cargo features.
///
/// # Arguments
/// * `id` - The board ID.
/// 
/// # Errors
/// Returns an error if the requested board ID is not in the `SUPPORTED_BOARD_IDS` list.
pub fn get_board(id: &str) -> Result<Board, String> {
    let board = match id {
        "diecimilaatmega168" => Board::new(id, "arduino-diecimila", "nightly-2025-04-27")?,
        "leonardo" => Board::new(id, "arduino-leonardo", "nightly-2025-04-27")?,
        "ATmega2560" => Board::new(id, "arduino-mega2560", "nightly-2025-04-27")?,
        "ATmega1280" => Board::new(id, "arduino-mega1280", "nightly-2025-04-27")?,
        "nanoatmega328" => Board::new(id, "arduino-nano", "nightly-2025-04-27")?,
        "nanoatmega328new" => Board::new(id, "arduino-nano", "nightly-2025-04-27")?,
        "micro" => Board::new(id, "arduino-micro", "nightly-2025-04-27")?,
        "uno" => Board::new(id, "arduino-uno", "nightly-2025-04-27")?,
        "sparkfun_promicro8" => Board::new(id, "sparkfun-promicro", "nightly-2025-04-27")?,
        "nanoatmega168" => Board::new(id, "nano168", "nightly-2025-04-27")?,
        _ => {
            return Err("Unsupported board ID.".to_string());
        }
    };
    Ok(board)
}

/// Retrieves a list of supported boards, optionally filtered by a substring match on the ID.
/// 
/// # Arguments
/// * `filter` - The optional filter string.
/// 
/// # Errors
/// Returns an error if the invalid board ID occurs.
pub fn get_boards(filter: Option<&String>) -> Result<Vec<Board>, String> {
    let board_ids = match filter {
        Some(f) => get_filtered_board_ids(f),
        None => Vec::from(SUPPORTED_BOARD_IDS)
    };
    
    let mut boards = Vec::new();
    for id in board_ids {
        let board = get_board(id)?;
        boards.push(board);
    }
    Ok(boards)
}

/// Filters the supported board IDs by checking if they contain the given filter string (case-insensitive).
/// 
/// # Arguments
/// * `filter` - The filter string for board identifiers.
fn get_filtered_board_ids(filter: &String) -> Vec<&str> {
    let parsed_filter = filter.to_lowercase();
    let board_ids: Vec<&str> = SUPPORTED_BOARD_IDS.iter()
                                       .copied()
                                       .filter(|board_id| {
                                            board_id.to_lowercase().contains(&parsed_filter)
                                       })
                                       .collect();
    board_ids
}

/// Returns a placeholder "unspecified" board configuration.
///
/// This is used during project initialization if the user does not specify a target hardware board.
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