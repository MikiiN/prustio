//! Command-line arguments for project management.
//!
//! This module defines the `clap` subcommands used when a user executes 
//! `prustio project`. It provides the configuration structures for initiating 
//! new PrustIO projects.

use clap::Subcommand;

/// Subcommands available under the `project` command.
#[derive(Subcommand)]
pub enum ProjectCommands {
    Init {
        name: Option<String>,

        #[arg(short, long)]
        board: Option<String>,

        #[arg(long)]
        hybrid: bool,

        #[arg(long)]
        json_output: bool,
    },
}
