//! Defines the main Command-Line Interface (CLI) structure.
//!
//! This module utilizes the `clap` crate to parse command-line arguments, 
//! flags, and subcommands. It acts as the entry point for user interaction, 
//! routing commands to the appropriate controllers.

use clap::{Parser, Subcommand};

pub mod device;
pub mod display;
pub mod project;

/// The root CLI parser for pRustIO.
#[derive(Parser)]
#[command(version)]
#[command(name = "PrustIO")]
#[command(about = "A project manager for Rust embedded projects.", long_about = None)]
pub struct Cli {
    /// The specific top-level command executed by the user.
    #[command(subcommand)]
    pub(crate) command: TopLevelCommands,
}

/// Represents the top-level subcommands available in the pRustIO CLI.
#[derive(Subcommand)]
pub enum TopLevelCommands {
    /// Lists all supported hardware boards and their specifications.
    Boards {
        filter: Option<String>,

        #[arg(long)]
        json_output: bool,
    },

    /// Manages connected hardware devices and serial monitoring.
    Device {
        #[command(subcommand)]
        command: device::DeviceCommands,
    },

    /// Manages pRustIO projects.
    Project {
        #[command(subcommand)]
        command: project::ProjectCommands,
    },
    
    /// Builds and/or uploads the project to a connected board.
    Run {
        #[arg(short, long)]
        target: Option<String>,
        #[arg(short, long)]
        environment: Option<String>,

        #[arg(long)]
        json_output: bool,
    },

    /// Switches the actively configured environment for the project.
    Activate {
        environment: String,

        #[arg(long)]
        json_output: bool,
    },

    /// Refreshes the project configuration based on the active environment.
    Refresh {
        #[arg(long)]
        json_output: bool,
    },

    /// Cleans the project by removing the `target/` and `.prio/` compilation directories.
    Clean {
        #[arg(long)]
        json_output: bool,
    },
}
