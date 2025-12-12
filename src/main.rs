use clap::{Parser, Subcommand};
use colored::*;
use cs_01::commands;
#[derive(Parser)]
#[command(name = "CS01")]
#[command(about = "\n\nCS01 Version Control System", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new CS01 repository
    Init {
        /// Create a bare repository (one without a working tree, just web-like storage)
        #[arg(long)]
        bare: bool,

        /// Specify the initial branch name (defaults to "main")
        #[arg(long, default_value = "main")]
        initial_branch: String,

        /// Specify the directory to initialize (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Init {
            bare,
            initial_branch,
            path,
        } => commands::init::init(*bare, initial_branch, path),
    };

    if let Err(e) = result {
        eprintln!("{}", format!("Error: {}", e).bright_red());
        std::process::exit(1);
    }
}
