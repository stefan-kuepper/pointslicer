use crate::error::Result;
use crate::geometry::ExtractGeometry;
use crate::index::TileIndexReader;
use crate::pointcloud::{PointCloudReader, PointCloudWriter, filter_points};
use las::Point;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Statistics collected during an extraction operation.
///
/// This struct contains metrics about the extraction process including
/// tile counts, point counts, and timing information.
///
/// # Fields
///
/// - `tiles_processed`: Number of tiles that were successfully processed
/// - `points_read`: Total number of points read from all tiles
/// - `points_written`: Number of points that passed the geometry filter
/// - `elapsed_time`: Total time taken for the extraction
#[derive(Debug, Clone)]
pub struct ExtractionStats {
    /// Number of tiles that were successfully processed.
    pub tiles_processed: usize,

    /// Total number of points read from all tiles.
    pub points_read: u64,

    /// Number of points that passed the geometry filter.
    pub points_written: u64,

    /// Total time taken for the extraction.
    pub elapsed_time: Duration,
}

/// Main pipeline for orchestrating point cloud extraction.
///
/// This struct manages the entire extraction workflow from reading tile indices
/// to writing output files. It uses parallel processing with Rayon for
/// efficient tile processing.
///
/// # Fields
///
/// - `index_path`: Path to the GeoPackage tile index file
/// - `output_path`: Path where the extracted points will be written
/// - `verbose`: Whether to log detailed progress information
pub struct ExtractionPipeline {
    /// Path to the GeoPackage tile index file.
    pub index_path: PathBuf,

    /// Path where the extracted points will be written.
    pub output_path: PathBuf,

    /// Whether to log detailed progress information.
    pub verbose: bool,
}

impl ExtractionPipeline {
    /// Create a new extraction pipeline
    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(
        index_path: P1,
        output_path: P2,
        verbose: bool,
    ) -> Self {
        Self {
            index_path: index_path.as_ref().to_path_buf(),
            output_path: output_path.as_ref().to_path_buf(),
            verbose,
        }
    }

    /// Execute the extraction pipeline
    pub fn execute<G: ExtractGeometry>(&self, geometry: &G) -> Result<ExtractionStats> {
        let start_time = Instant::now();

        if self.verbose {
            log::info!("Opening tile index: {:?}", self.index_path);
        }

        // 1. Read GeoPackage tile index
        let reader = TileIndexReader::open(&self.index_path)?;
        let tiles = reader.read_tiles(None, geometry)?;

        if self.verbose {
            log::info!("Found {} intersecting tiles", tiles.len());
        }

        // 2. Process tiles in parallel and filter points
        let tile_results: Vec<(u64, Vec<Point>)> = tiles
            .par_iter()
            .filter_map(|tile| {
                if self.verbose {
                    log::info!("Processing tile: {:?}", tile.file_path);
                }

                // Open the LAS/LAZ file
                let mut reader = match PointCloudReader::open(&tile.file_path) {
                    Ok(r) => r,
                    Err(e) => {
                        log::warn!("Failed to open {:?}: {}", tile.file_path, e);
                        return None;
                    }
                };

                // Read and filter points
                match reader.points() {
                    Ok(points) => {
                        let point_count = points.len() as u64;
                        let filtered = filter_points(&points, geometry);
                        if self.verbose {
                            log::info!(
                                "Tile {:?}: {} -> {} points",
                                tile.file_path,
                                point_count,
                                filtered.len()
                            );
                        }
                        Some((point_count, filtered))
                    }
                    Err(e) => {
                        log::warn!("Failed to read points from {:?}: {}", tile.file_path, e);
                        None
                    }
                }
            })
            .collect();

        // 3. Aggregate statistics
        let points_read: u64 = tile_results.iter().map(|(count, _)| count).sum();
        let all_filtered: Vec<Point> = tile_results
            .into_iter()
            .flat_map(|(_, points)| points)
            .collect();
        let points_written = all_filtered.len() as u64;

        if self.verbose {
            log::info!(
                "Total: read {} points, writing {} points",
                points_read,
                points_written
            );
        }

        // 4. Write output LAS/LAZ file
        if points_written > 0 {
            // Get header from the first tile to preserve point format and other settings
            let first_tile = &tiles[0];
            let first_reader = PointCloudReader::open(&first_tile.file_path)?;
            let source_header = first_reader.header();

            // Create a Builder from the source header to preserve format settings
            let mut builder = las::Builder::from(source_header.clone());

            // Update bounds to match the filtered points
            builder.point_format = *source_header.point_format();

            let output_header = builder.into_header()?;

            // Writer automatically handles LAZ compression based on file extension
            let mut writer = PointCloudWriter::create(&self.output_path, output_header)?;
            writer.write_points(&all_filtered)?;
            writer.close()?;

            if self.verbose {
                log::info!("Wrote output to: {:?}", self.output_path);
            }
        } else {
            log::warn!("No points to write!");
        }

        let elapsed_time = start_time.elapsed();

        Ok(ExtractionStats {
            tiles_processed: tiles.len(),
            points_read,
            points_written,
            elapsed_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_stats() {
        let stats = ExtractionStats {
            tiles_processed: 5,
            points_read: 1000,
            points_written: 500,
            elapsed_time: Duration::from_secs(2),
        };
        assert_eq!(stats.tiles_processed, 5);
        assert_eq!(stats.points_read, 1000);
        assert_eq!(stats.points_written, 500);
    }

    #[test]
    fn test_pipeline_construction() {
        let pipeline = ExtractionPipeline::new("index.gpkg", "output.laz", true);
        assert_eq!(pipeline.index_path, PathBuf::from("index.gpkg"));
        assert_eq!(pipeline.output_path, PathBuf::from("output.laz"));
        assert!(pipeline.verbose);
    }
}
