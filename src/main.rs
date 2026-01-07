use clap::Parser;
use pointslicer::cli::commands::Cli;
use pointslicer::cli::execute;

fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Execute the command
    if let Err(e) = execute(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
