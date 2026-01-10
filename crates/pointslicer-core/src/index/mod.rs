//! GeoPackage tile index reading and spatial filtering.
//!
//! This module provides functionality for reading GeoPackage tile indices
//! created by `pdal tindex` and performing spatial queries to find tiles
//! that intersect with extraction geometries.
//!
//! ## Key Components
//!
//! - [`TileIndexReader`]: Main interface for reading GeoPackage tile indices
//! - [`TileInfo`]: Information about a single tile (file path and bounds)
//! - [`TileMetadata`]: Metadata about a tile's spatial extent
//!
//! ## Usage Example
//!
//! ```no_run
//! use pointslicer_core::index::TileIndexReader;
//! use pointslicer_core::geometry::BoundingBox;
//!
//! // Open a GeoPackage tile index
//! let reader = TileIndexReader::open("tiles.gpkg")?;
//!
//! // Create an extraction geometry
//! let bbox = BoundingBox::new_2d(10000.0, 20000.0, 30000.0, 40000.0);
//!
//! // Find tiles that intersect with the geometry
//! let tiles = reader.read_tiles(None, &bbox)?;
//!
//! println!("Found {} intersecting tiles", tiles.len());
//! # Ok::<(), pointslicer_core::TileIndexError>(())
//! ```
//!
//! ## GeoPackage Format Support
//!
//! The module supports GeoPackage tile indices created by PDAL's `tindex` command,
//! which typically contain:
//! - A `location` column with file paths to LAS/LAZ tiles
//! - A geometry column with tile bounds in GeoPackage Binary format
//! - Metadata in `gpkg_contents` and `gpkg_geometry_columns` tables

pub mod reader;
pub mod tile;

/// Reader for GeoPackage tile index files.
///
/// This struct provides methods to open GeoPackage files and query tiles
/// that intersect with extraction geometries.
pub use reader::TileIndexReader;

/// Information about a single tile in the index.
///
/// Contains the file path to the LAS/LAZ tile and its spatial bounds.
pub use tile::TileInfo;

/// Metadata about a tile's spatial extent.
///
/// Contains the minimum and maximum coordinates of a tile's bounding box.
pub use tile::TileMetadata;
