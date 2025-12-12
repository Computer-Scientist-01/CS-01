
use clap::{Parser, Subcommand};
use anyhow::Result;

use cs_01::commands;

#[derive(Parser)]
#[command(name = "CS01")]
#[command(about = "CS01 Version Control System", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new CS01 repository
    Init {
        /// Create a bare repository
        #[arg(long)]
        bare: bool,

        /// Specify the initial branch name
        #[arg(long, default_value = "main")]
        initial_branch: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { bare, initial_branch } => {
            commands::init::init(*bare, initial_branch)?;
        }
    }

    Ok(())
}
