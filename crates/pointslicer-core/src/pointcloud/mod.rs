//! LAS/LAZ point cloud I/O and filtering.
//!
//! This module provides functionality for reading and writing LAS/LAZ files,
//! as well as filtering points based on extraction geometries.
//!
//! ## Key Components
//!
//! - [`PointCloudReader`]: Wrapper for reading LAS/LAZ files
//! - [`PointCloudWriter`]: Wrapper for writing LAS/LAZ files with format preservation
//! - [`filter_points()`]: Function for filtering points using extraction geometries
//! - [`filter_points_iter()`]: Iterator-based filtering for memory efficiency
//!
//! ## Usage Example
//!
//! ```no_run
//! use pointslicer_core::pointcloud::{PointCloudReader, PointCloudWriter, filter_points};
//! use pointslicer_core::geometry::BoundingBox;
//!
//! # fn main() -> Result<(), pointslicer_core::TileIndexError> {
//! // Read points from a LAS/LAZ file
//! let mut reader = PointCloudReader::open("input.laz")?;
//! let points = reader.points()?;
//!
//! // Create an extraction geometry
//! let bbox = BoundingBox::new_2d(10000.0, 20000.0, 30000.0, 40000.0);
//!
//! // Filter points
//! let filtered = filter_points(&points, &bbox);
//!
//! // Write filtered points to a new file
//! let mut writer = PointCloudWriter::create("output.laz", reader.header().clone())?;
//! writer.write_points(&filtered)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Point Format Preservation
//!
//! The [`PointCloudWriter`] preserves the point format from the source file
//! when creating output files, ensuring compatibility with downstream tools.

pub mod filter;
pub mod reader;
pub mod writer;

/// Functions for filtering points using extraction geometries.
pub use filter::{filter_points, filter_points_iter};

/// Wrapper for reading LAS/LAZ files.
pub use reader::PointCloudReader;

/// Wrapper for writing LAS/LAZ files with format preservation.
pub use writer::PointCloudWriter;
