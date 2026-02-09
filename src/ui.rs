use clap::{Parser, Subcommand};

mod project;
mod device;

#[derive(Parser)]
#[command(name = "PrustIO")]
#[command(about = "A project manager for Rust embedded projects.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: TopLevelCommands,
}

// The main categories
#[derive(Subcommand)]
enum TopLevelCommands {
    Boards {
        filter: Option<String>,

        #[arg(long)]
        json_output: bool,
    },

    Device {
        #[command(subcommand)]
        command: device::DeviceCommands,
    },

    // Manage project
    Project {
        #[command(subcommand)]
        command: project::ProjectCommands,
    },
    
    // Run targets
    Run {
        target: Option<String>,

        #[arg(long)]
        json_output: bool,
    }
}

// impl Cli {
//     pub fn get_arguments(&self) -> CommandArgs {
//         match &self.command {
//             TopLevelCommands::Project { command } => match command {
//                 project::ProjectCommands::Add {package, json_output} => {
//                     CommandArgs::ProjAdd(ProjAddRemoveArguments::new(package, json_output))
//                 },
//                 project::ProjectCommands::Init {name, board, hybrid, json_output} => {
//                     CommandArgs::ProjInit(
//                         ProjInitArguments::new(name, board, hybrid, json_output)
//                     )
//                 },
//                 project::ProjectCommands::Remove {package, json_output} => {
//                     CommandArgs::ProjRemove(
//                         ProjAddRemoveArguments::new(package,json_output)
//                     )
//                 },
//                 project::ProjectCommands::Tasks {json_output} => {
//                     CommandArgs::ProjTasks(
//                         ProjTasksArguments::new(json_output)
//                     )
//                 },
//             },
//             TopLevelCommands::Run {target, json_output} => {
//                 CommandArgs::Run(
//                     RunArguments::new(target, json_output)
//                 )
//             },
//         }
//     }
// }
