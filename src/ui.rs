use clap::{Parser, Subcommand};

pub mod project;
pub mod device;

#[derive(Parser)]
#[command(name = "PrustIO")]
#[command(about = "A project manager for Rust embedded projects.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: TopLevelCommands,
}

// The main categories
#[derive(Subcommand)]
pub enum TopLevelCommands {
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
        #[arg(short, long)]
        target: Option<String>,
        #[arg(short, long)]
        environment: Option<String>,

        #[arg(long)]
        json_output: bool,
    }
}
