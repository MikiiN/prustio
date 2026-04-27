use clap::Parser;

use crate::ui;
use crate::ui::device::DeviceCommands;
use crate::ui::display;
use crate::ui::project::ProjectCommands;
use crate::wrapper::platformio;

mod ctr_activate;
mod ctr_board;
mod ctr_clear;
mod ctr_device;
mod ctr_project;
mod ctr_refresh;
mod ctr_run;

pub fn execute() -> i32 {
    if !platformio::check_pio_installation() {
        if let Err(_) = platformio::setup_platformio() {
            display::error("Can not install application's instance of PlatformIO");
            return 1;
        }
    }

    let cli = ui::Cli::parse();
    if let Err(msg) = run_command(&cli) {
        display::error(msg.as_str());
        return 1;
    }
    display::success("Successfully executed command.");
    return 0;
}

fn run_command(cli: &ui::Cli) -> Result<(), String> {
    match &cli.command {
        ui::TopLevelCommands::Boards { filter, json_output } => {
            ctr_board::board(filter.as_ref(), json_output)?;
        },

        ui::TopLevelCommands::Device { command } => match command {
            DeviceCommands::List { json_output } => {
                ctr_device::device_list(json_output);
            },
            DeviceCommands::Monitor { 
                port, 
                baud, 
                parity, 
                rtscts, 
                xonxoff, 
                rts, 
                dtr, 
                echo, 
                encoding, 
                filter, 
                eol, 
                raw, 
                exit_char, 
                menu_char, 
                quiet, 
                no_reconnect 
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
            },
        },

        ui::TopLevelCommands::Run {target, environment, json_output} => {
            ctr_run::run(target, environment.as_ref(), json_output)?;
        },

        ui::TopLevelCommands::Activate { environment, json_output } => {
            ctr_activate::activate_environment(environment, json_output)?;
        },

        ui::TopLevelCommands::Refresh { json_output } => {
            ctr_refresh::refresh(json_output)?;
        },
        
        ui::TopLevelCommands::Clear { json_output } => {
            ctr_clear::clear(json_output)?;
        },
    }
    Ok(())
}