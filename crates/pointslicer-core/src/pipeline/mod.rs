//! Pipeline orchestration for point cloud extraction.
//!
//! This module provides the high-level pipeline that orchestrates the
//! extraction workflow, including reading tile indices, processing tiles
//! in parallel, filtering points, and writing output files.
//!
//! ## Key Components
//!
//! - [`ExtractionPipeline`]: Main pipeline struct that orchestrates the extraction process
//! - [`ExtractionStats`]: Statistics collected during extraction
//!
//! ## Usage Example
//!
//! ```no_run
//! use pointslicer_core::pipeline::ExtractionPipeline;
//! use pointslicer_core::geometry::BoundingBox;
//!
//! # fn main() -> Result<(), pointslicer_core::TileIndexError> {
//! // Create an extraction geometry
//! let bbox = BoundingBox::new_2d(10000.0, 20000.0, 30000.0, 40000.0);
//!
//! // Create and run the pipeline
//! let pipeline = ExtractionPipeline::new("tiles.gpkg", "output.laz", false);
//! let stats = pipeline.execute(&bbox)?;
//!
//! println!("Extracted {} points from {} tiles in {:?}",
//!     stats.points_written, stats.tiles_processed, stats.elapsed_time);
//! # Ok(())
//! # }
//! ```
//!
//! ## Pipeline Flow
//!
//! 1. **Index Reading**: Opens GeoPackage, queries tiles intersecting the geometry
//! 2. **Parallel Processing**: Uses Rayon to process tiles in parallel
//! 3. **Point Filtering**: Filters points using geometry's `contains_xyz()` method
//! 4. **Output Writing**: Writes filtered points to LAS/LAZ, preserving point format

pub mod executor;

/// Main pipeline for orchestrating point cloud extraction.
///
/// This struct manages the entire extraction workflow from reading tile indices
/// to writing output files.
pub use executor::ExtractionPipeline;

/// Statistics collected during an extraction operation.
///
/// Contains metrics about the extraction process including tile counts,
/// point counts, and timing information.
pub use executor::ExtractionStats;
