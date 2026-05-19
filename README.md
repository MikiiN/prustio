# pRustIO

pRustIO is a Command-Line Interface (CLI) tool that makes it easier to develop embedded projects using Rust. It connects the Rust `cargo` build system with the C/C++ embedded world by automatically linking the Arduino framework to your application. This tool is being developed as part of a diploma thesis at the Faculty of Information Technology, Brno University of Technology.

## Core Concepts

pRustIO has two different build modes, depending on what your project needs:
* **Pure Mode (Default):** This mode works completely inside the Rust system using the `arduino-hal` crate. It does not use any extra C or C++ code, which gives you memory-safe applications.
* **Hybrid Mode (`--hybrid`):** This mode sets up your project to use the standard Arduino C++ framework. pRustIO creates a hidden PlatformIO workspace, compiles the C/C++ libraries, and allows you to use Arduino functions (like `pinMode` and `digitalWrite`) in your Rust code using the [`prustio-arduino`](https://github.com/MikiiN/prustio-arduino-crate) crate.

pRustIO supports many popular Arduino and AVR boards, including the Uno, Nano, Mega 2560, Leonardo, and Micro. 

## Prerequisites and Installation

Before you install pRustIO, your computer must meet these basic requirements:
* **Rust and Cargo:** You must have the Rust programming language and Cargo installed.
* **Python 3:** pRustIO uses Python 3 to automatically download and run a local version of PlatformIO. Make sure Python 3 is available in your system path.

**Installation from crates.io:**
```bash
cargo install prustio
```

**Installation from the repository:**
```bash
git clone https://github.com/MikiiN/prustio
cd prustio
cargo install --path .
```

You can also use pRustIO as a [Visual Studio Code extension](https://github.com/MikiiN/prustio-vscode-extension) by searching for it in the VS Code extensions marketplace.

[!NOTE]
**Note on Automatic Setup**: You do not need to manually install `PlatformIO`, `AVRDUDE`, or the `avr-gcc` compiler. The first time you run a `pRustIO` command, the tool will automatically create a private Python virtual environment in your home directory (`~/.prustio/`) and download everything it needs.

## Project Management & Commands

Here is a list of commands you can use to manage your embedded projects:

### Project Management

#### Create project
```bash
prustio project init [PROJECT_NAME] [OPTIONS]
```
This creates all the necessary setup files and a basic starting code file. You can use `-b <BOARD>` to specify the target board (like uno or nano) and `--hybrid` to create a hybrid project.

#### Clean project
```bash
prustio clean
```
This deletes the created build files to save disk space or start a fresh build.

### Building and Uploading

#### Run a target
```bash
prustio run [OPTIONS]
```
This command checks your setup, compiles the code, and uploads it to the board. Use `-t build` to only compile the code, or `-t upload` to compile and upload it to the board. To specify compilation environment, use `-e <NAME>`. 

### Device & Serial Monitor

#### Find Connected Devices
```bash
prustio device list
```
Check which microcontrollers and serial devices are connected to your computer.

#### Serial Monitor
```bash
prustio device monitor [OPTIONS]
```
Open the built-in serial monitor to communicate with your device. You can specify the port (`-p <PORT>`), communication speed (`-b <BAUD>`), and much more.

### Environment Management

#### List Boards
```bash
prustio boards [FILTER]
```
See all the microcontrollers that pRustIO supports. Optionally results can be filtered.

#### Active Environment
```bash 
prustio activate <ENVIRONMENT_NAME>
```
Change the active board, which updates the settings for the new hardware.

#### Update Configuration
```bash
prustio refresh
```
If you change your `Prustio.toml` file manually, this forces pRustIO to update the project settings to match.


## Example: Blinking an LED
This basic starting code takes control of the device hardware and turns the built-in LED (usually pin 13) on and off in Pure Rust mode:

```rust
#![no_std]
#![no_main]

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    // Get the device hardware
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Set pin D13 as a digital output
    let mut led = pins.d13.into_output();

    loop {
        led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
```

More examples can be seen in the pRustIO [documentation](https://mikiin.github.io/prustio/).