use clap::{Subcommand, ValueEnum};

// Subcommands for device command
#[derive(Subcommand)]
pub enum DeviceCommands {
    List {
        #[arg(long)]
        json_output: bool,
    },

    Monitor {
        #[arg(long, short)]
        port: Option<String>,

        #[arg(long, short)]
        baud: Option<u32>,

        #[arg(long, default_value = "N", ignore_case = true, value_name = "PARITY")]
        parity: Option<Parity>,

        #[arg(long)]
        rtscts: bool,

        #[arg(long)]
        xonxoff: bool, 

        #[arg(long, value_parser = clap::value_parser!(u8).range(0..=1))]
        rts: Option<u8>,

        #[arg(long, value_parser = clap::value_parser!(u8).range(0..=1))]
        dtr: Option<u8>,

        #[arg(long)]
        echo: bool,

        #[arg(long, default_value = "UTF-8")]
        encoding: Option<String>,

        #[arg(short, long)]
        filter: Option<String>,

        #[arg(long, default_value = "CRLF", ignore_case = true, value_name = "EOL")]
        eol: Option<EOL>,

        #[arg(long)]
        raw: bool,

        #[arg(long, default_value = "3")]
        exit_char: Option<u8>,

        #[arg(long, default_value = "20")]
        menu_char: Option<u8>,

        #[arg(long)]
        quiet: bool,

        #[arg(long)]
        no_reconnect: bool,
    }
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum Parity {
    #[value(name = "N")]
    N,
    #[value(name = "E")]
    E,
    #[value(name = "O")]
    O,
    #[value(name = "S")]
    S,
    #[value(name = "M")]
    M,
}

impl Parity {
    pub fn to_string(&self) -> String {
        match self {
            Parity::N => "N".to_string(),
            Parity::E => "E".to_string(),
            Parity::O => "O".to_string(),
            Parity::S => "S".to_string(),
            Parity::M => "M".to_string(),
        }
    }
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum EOL {
    #[value(name = "CR")]
    CR,
    #[value(name = "LF")]
    LF,
    #[value(name = "CRLF")]
    CRLF
}

impl EOL {
    pub fn to_string(&self) -> String {
        match self {
            EOL::CR => "CR".to_string(),
            EOL::LF => "LF".to_string(),
            EOL::CRLF => "CRLF".to_string(),
        }
    }
}