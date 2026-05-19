//! Main controller module for the pRustIO application.
//!
//! This module acts as the central hub of the application. It connects 
//! the user interface (CLI commands) to the underlying logic. It routes 
//! the user's commands to specific sub-controllers, such as managing 
//! projects, running builds, or communicating with hardware devices.

use clap::Parser;

use crate::ui;
use crate::ui::device::DeviceCommands;
use crate::ui::display;
use crate::ui::project::ProjectCommands;
use crate::wrapper::platformio;

mod ctr_activate;
mod ctr_board;
mod ctr_clean;
mod ctr_device;
mod ctr_project;
mod ctr_refresh;
mod ctr_run;

/// Executes the main logic of the application and returns the corresponding
/// exit code.
///
/// This function is the main entry point after the program starts. 
/// First, it checks if PlatformIO is installed and tries to set it up 
/// if it is missing. Then, it parses the command-line arguments 
/// provided by the user and runs the corresponding command.
///
/// Finally, it prints a success message or an error message based on 
/// the result.
///
pub fn execute() -> i32 {
    if !platformio::check_pio_installation() {
        if let Err(_) = platformio::setup_platformio() {
            display::error("Can not install application's instance of PlatformIO");
            return 1;
        }
    }

    let cli = ui::Cli::parse();
    match run_command(&cli) {
        Ok(output) => {
            if let Some(msg) = output {
                display::success(msg.as_str());
            } else {
                display::success("Successfully executed command.");
            }
        },
        Err(msg) => {
            display::error(msg.as_str());
            return 1;
        }
    }

    return 0;
}

/// Routes the parsed CLI command to the correct sub-controller.
///
/// This function takes the parsed command-line interface (CLI) data 
/// and uses a `match` statement to find out which action the user 
/// wants to perform. It then calls the right function from the
/// specialized sub-controllers to do the actual work.
///
/// # Arguments
/// * `cli` - A reference to the parsed CLI arguments.
///
/// # Returns
/// * `Ok(Some(String))` - A custom success message to show to the user.
/// * `Ok(None)` - The command succeeded, and a default success message will be used.
/// * `Err(String)` - An error message explaining why the command failed.
fn run_command(cli: &ui::Cli) -> Result<Option<String>, String> {
    match &cli.command {
        ui::TopLevelCommands::Boards { filter, json_output } => {
            ctr_board::board(filter.as_ref(), json_output)?;
        },

        ui::TopLevelCommands::Device { command } => match command {
            DeviceCommands::List { json_output } => {
                ctr_device::device_list(json_output)?;
            },
            DeviceCommands::Monitor { 
                port, baud, parity, rtscts, xonxoff, rts, 
                dtr, echo, encoding, filter, eol, raw, exit_char, 
                menu_char, quiet, no_reconnect 
            } => {
                ctr_device::device_monitor(
                    port, baud, parity, rtscts, xonxoff, rts, dtr, echo, 
                    encoding, filter, eol, raw, exit_char, menu_char, quiet, no_reconnect
                )?;
            }
        },

        ui::TopLevelCommands::Project { command } => match command {
            ProjectCommands::Init { name, board, hybrid, json_output } => {
                ctr_project::init_project(name, board, hybrid, json_output)?;
                return Ok(Some("The project was successfully initialized.".to_string()));
            },
        },

        ui::TopLevelCommands::Run {target, environment, json_output} => {
            ctr_run::run(target, environment.as_ref(), json_output)?;
        },

        ui::TopLevelCommands::Activate { environment, json_output } => {
            ctr_activate::activate_environment(environment, json_output)?;
            return Ok(Some(format!("Successfully activated environment: {}.", environment)));
        },

        ui::TopLevelCommands::Refresh { json_output } => {
            ctr_refresh::refresh(json_output)?;
            return Ok(Some("The project configuration was refreshed.".to_string()));
        },

        ui::TopLevelCommands::Clean { json_output } => {
            ctr_clean::clean(json_output)?;
            return Ok(Some("Clean command run successfully.".to_string()));
        },
    }
    Ok(None)
}