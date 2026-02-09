use clap::Parser;

mod ui;

fn main() {
    let cli = ui::Cli::parse();
}