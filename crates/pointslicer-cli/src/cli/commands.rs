use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Extract point cloud data using a tile index
#[derive(Parser, Debug)]
#[command(name = "pointslicer")]
#[command(about = "Extract point cloud data from LAS/LAZ files using a tile index")]
#[command(version)]
pub struct Cli {
    /// Path to GeoPackage tile index file
    #[arg(short, long)]
    pub index: PathBuf,

    /// Output LAS/LAZ file path
    #[arg(short, long)]
    pub output: PathBuf,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Extraction geometry subcommand
    #[command(subcommand)]
    pub command: Commands,
}

/// Extraction geometry commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Extract points within a vertical cylinder
    Cylinder {
        /// Center X coordinate
        #[arg(short, long)]
        x: f64,

        /// Center Y coordinate
        #[arg(short, long)]
        y: f64,

        /// Cylinder diameter
        #[arg(short, long)]
        diameter: f64,
    },

    /// Extract points within a bounding box
    #[command(name = "bbox")]
    BBox {
        /// Minimum X coordinate
        #[arg(long)]
        min_x: f64,

        /// Maximum X coordinate
        #[arg(long)]
        max_x: f64,

        /// Minimum Y coordinate
        #[arg(long)]
        min_y: f64,

        /// Maximum Y coordinate
        #[arg(long)]
        max_y: f64,

        /// Minimum Z coordinate (optional)
        #[arg(long)]
        min_z: Option<f64>,

        /// Maximum Z coordinate (optional)
        #[arg(long)]
        max_z: Option<f64>,
    },
}
