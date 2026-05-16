//! Generates the initial template source code for a new pRustIO project.
//!
//! When a user runs `prustio project init`, this module is responsible for 
//! scaffolding the `src/main.rs` file. It provides a standard "Blink" example 
//! customized for either a pure Rust environment (using `arduino-hal`) or a 
//! hybrid environment (using the C++ wrapper).

use std::fs;
use std::path::PathBuf;

const SRC_FOLDER: &str = "src";
const MAIN_FILE_NAME: &str = "main.rs";

/// Writes the `src/main.rs` file with a default blinking LED example.
///
/// This function creates the boilerplate code necessary to immediately build 
/// and verify the toolchain setup. It automatically selects the correct boilerplate 
/// based on the selected project mode.
///
/// # Arguments
///
/// * `proj_path` - The root directory of the project where `src/main.rs` will be created.
/// * `hybrid_mode` - If `true`, generates an example utilizing the Arduino C++ bindings. 
///   If `false`, generates a pure `arduino-hal` example.
///
/// # Errors
///
/// Returns an error string if the file cannot be written to disk.
pub fn write_example_code(proj_path: &PathBuf, hybrid_mode: &bool) -> Result<(), String> {
    let main_file = proj_path.join(SRC_FOLDER).join(MAIN_FILE_NAME);
    
    let content = if *hybrid_mode {
        get_main_example_content_hybrid_mode()
    } else {
        get_main_example_content_pure_mode()
    };

    match fs::write(main_file, content) {
        Ok(_) => (),
        Err(_) => {
            return Err(String::from("Failed to write code example."));
        }
    };

    Ok(())
}

/// Returns the raw Rust source code string for a pure `arduino-hal` blinking LED example.
///
/// This example operates entirely within the Rust ecosystem without relying on 
/// any external C/C++ libraries.
fn get_main_example_content_pure_mode() -> String {
    String::from("#![no_std]
#![no_main]

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut led = pins.d13.into_output();
    loop {
        led.toggle();
        arduino_hal::delay_ms(1000);
    }
}")
}

/// Returns the raw Rust source code string for a hybrid mode blinking LED example.
///
/// This example utilizes the `prustio_arduino` crate to access the standard Arduino 
/// C++ framework functions (`pinMode`, `digitalWrite`, `delay`) through FFI bindings.
fn get_main_example_content_hybrid_mode() -> String {
    String::from("#![no_std]
#![no_main]

use panic_halt as _;

use prustio_arduino::{init, Pin, pin_mode, delay, PinMode, PinState, digital_write};

#[arduino_hal::entry]
fn main() -> ! {
    init();
    let pin = Pin::new(13);
    pin_mode(&pin, PinMode::Output);

    loop {
        digital_write(&pin, PinState::High);
        delay(1000);
        digital_write(&pin, PinState::Low);
        delay(1000);
    }
}")
}