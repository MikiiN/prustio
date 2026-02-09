use clap::Parser;

use crate::ui;
use crate::ui::device::DeviceCommands;
use crate::ui::project::ProjectCommands;

pub fn execute() {
    let cli = ui::Cli::parse();

    match &cli.command {
        ui::TopLevelCommands::Boards { filter, json_output } => {

        },
        ui::TopLevelCommands::Device { command } => match command {
            DeviceCommands::List { json_output } => {

            },
            
        },
        ui::TopLevelCommands::Project { command } => match command {
            ProjectCommands::Add { package, json_output } => {

            },
            ProjectCommands::Init { name, board, hybrid, json_output } => {

            },
            ProjectCommands::Remove { package, json_output } => {

            },
            ProjectCommands::Tasks { json_output } => {

            },
        },
        ui::TopLevelCommands::Run {target, json_output} => {
            
        },
    }
}