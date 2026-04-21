use clap::Subcommand;

// Subcommands for project command
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
