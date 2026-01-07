use clap::Parser;
use tile_index_tool::cli::commands::Cli;
use tile_index_tool::cli::execute;

fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Execute the command
    if let Err(e) = execute(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
