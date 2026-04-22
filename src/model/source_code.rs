use std::fs;
use std::path::PathBuf;

const SRC_FOLDER: &str = "src";
const MAIN_FILE_NAME: &str = "main.rs";

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