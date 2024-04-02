use std::{env, path::PathBuf};

use anyhow::Result;
use clap::{command, Parser, Subcommand};

mod cli;
mod common_ports;
mod dns;
mod error;
mod modules;
mod ports;
pub use error::Error;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Scan a target")]
    Scan {
        #[arg(short, long)]
        target: String,
    },
    Modules,
}

fn main() -> Result<(), anyhow::Error> {
    env::set_var("RUST_LOG", "info,trust_dns_proto=error");
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { target } => cli::scan(&target)?,
        Commands::Modules => {
            cli::modules();
        }
    }

    Ok(())
}
