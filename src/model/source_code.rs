use std::fs;
use std::path::PathBuf;

const SRC_FOLDER: &str = "src";
const MAIN_FILE_NAME: &str = "main.rs";

pub fn write_example_code(proj_path: &PathBuf) -> Result<(), String> {
    let main_file = proj_path.join(SRC_FOLDER).join(MAIN_FILE_NAME);
    match fs::write(main_file, get_main_example_content().to_string()) {
        Ok(_) => (),
        Err(_) => {
            return Err(String::from("Failed to write code example."));
        }
    };

    Ok(())
}

fn get_main_example_content() -> String {
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