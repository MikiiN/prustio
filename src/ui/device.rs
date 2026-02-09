use clap::Subcommand;

// Subcommands for device command
#[derive(Subcommand)]
pub enum DeviceCommands {
    List {
        #[arg(long)]
        json_output: bool,
    }
}