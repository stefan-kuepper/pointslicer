mod cli;

use clap::Parser;
use cli::commands::Cli;
use cli::execute;

fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Execute the command
    if let Err(e) = execute(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
