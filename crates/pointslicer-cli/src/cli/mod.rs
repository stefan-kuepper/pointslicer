pub mod commands;

use commands::{Cli, Commands};
use pointslicer_core::geometry::{BoundingBox, VerticalCylinder};
use pointslicer_core::pipeline::ExtractionPipeline;
use pointslicer_core::Result;

/// Execute the CLI command
pub fn execute(cli: Cli) -> Result<()> {
    // Set up logging if verbose
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    // Create the pipeline
    let pipeline = ExtractionPipeline::new(&cli.index, &cli.output, cli.verbose);

    // Execute based on the command
    let stats = match cli.command {
        Commands::Cylinder { x, y, diameter } => {
            let geometry = VerticalCylinder::from_diameter(x, y, diameter);
            pipeline.execute(&geometry)?
        }
        Commands::BBox {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        } => {
            let geometry = if let (Some(min_z), Some(max_z)) = (min_z, max_z) {
                BoundingBox::new_3d(min_x, max_x, min_y, max_y, min_z, max_z)
            } else {
                BoundingBox::new_2d(min_x, max_x, min_y, max_y)
            };
            pipeline.execute(&geometry)?
        }
    };

    // Display results
    println!("\nExtraction complete!");
    println!("  Tiles processed: {}", stats.tiles_processed);
    println!("  Points read:     {}", stats.points_read);
    println!("  Points written:  {}", stats.points_written);
    println!(
        "  Elapsed time:    {:.2}s",
        stats.elapsed_time.as_secs_f64()
    );

    Ok(())
}
