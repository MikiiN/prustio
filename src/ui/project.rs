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
    Add {
        package: Option<String>,

        #[arg(long)]
        json_output: bool,
    },
    Remove {
        package: Option<String>,

        #[arg(long)]
        json_output: bool,
    },
    Tasks {
        #[arg(long)]
        json_output: bool,
    }
}
