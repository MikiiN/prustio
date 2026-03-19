use clap::Parser;

use crate::ui;
use crate::ui::device::DeviceCommands;
use crate::ui::project::ProjectCommands;
use crate::wrapper::platformio;

mod ctr_board;
mod ctr_project;
mod ctr_run;

pub fn execute() {
    if !platformio::check_pio_installation() {
        match platformio::setup_platformio() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Can not install application's instance of PlatformIO:\n{}", e);
                return;
            }
        }
    }

    let cli = ui::Cli::parse();
    run_command(&cli);

}

fn run_command(cli: &ui::Cli) {
    match &cli.command {
        ui::TopLevelCommands::Boards { filter, json_output } => {
            ctr_board::board(filter.as_ref(), json_output);
        },

        ui::TopLevelCommands::Device { command } => match command {
            DeviceCommands::List { json_output } => {
                let _ = json_output;
            },
        },

        ui::TopLevelCommands::Project { command } => match command {
            ProjectCommands::Add { package, json_output } => {
                let _ = package;
                let _ = json_output;
            },
            ProjectCommands::Init { name, board, hybrid, json_output } => {
                ctr_project::init_project(name, board, hybrid, json_output);
            },
            ProjectCommands::Remove { package, json_output } => {
                let _ = package;
                let _ = json_output;
            },
            ProjectCommands::Tasks { json_output } => {
                let _ = json_output;
            },
        },

        ui::TopLevelCommands::Run {target, environment, json_output} => {
            ctr_run::run(target, environment.as_ref(), json_output);
        },

        ui::TopLevelCommands::Activate { environment, json_output } => {
            let _ = environment;
            let _ = json_output;
        },

        ui::TopLevelCommands::Refresh { json_output } => {
            let _ = json_output;
        },
    }
}