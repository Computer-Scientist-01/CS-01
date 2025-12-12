use clap::{Parser, Subcommand};
use colored::*;
use cs_01::commands;

/// The main structure for our Command Line Interface (CLI).
/// It uses the `clap` library to parse command line arguments automatically.
#[derive(Parser)]
#[command(name = "CS01")]
#[command(about = "\n\nCS01 Version Control System", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// The available subcommands for our application.
/// Currently, we only support `init`.
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
    // 1. Parse the arguments provided by the user.
    let cli = Cli::parse();

    // 2. Match against the subcommand content to decide what to do.
    let result = match &cli.command {
        Commands::Init {
            bare,
            initial_branch,
            path,
        } => commands::init::init(*bare, initial_branch, path),
    };

    // 3. Handle any errors that occurred during execution.
    // If there was an error, print it in red and exit with a failure code.
    if let Err(e) = result {
        eprintln!("{}", format!("Error: {}", e).bright_red());
        std::process::exit(1);
    }
}
