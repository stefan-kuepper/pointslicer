//! Error types for the pointslicer library.
//!
//! This module defines the main error type [`TileIndexError`] and a convenience
//! [`Result`] type alias used throughout the library.

use thiserror::Error;

/// The main error type for tile index operations.
///
/// This enum represents all possible errors that can occur during point cloud
/// extraction operations, including GeoPackage I/O, LAS/LAZ file operations,
/// and spatial query errors.
#[derive(Error, Debug)]
pub enum TileIndexError {
    /// Failed to open a GeoPackage file.
    ///
    /// This error occurs when the GeoPackage file cannot be opened or is
    /// not a valid GeoPackage database.
    #[error("Failed to open GeoPackage: {0}")]
    GeoPackageOpen(String),

    /// Failed to query tiles from the GeoPackage.
    ///
    /// This error occurs when SQL queries against the tile index fail,
    /// typically due to database schema issues or invalid SQL.
    #[error("Failed to query tiles: {0}")]
    TileQuery(String),

    /// Invalid geometry found in the tile index.
    ///
    /// This error occurs when the geometry data in the GeoPackage cannot
    /// be parsed or is in an unsupported format.
    #[error("Invalid geometry in tile index: {0}")]
    InvalidGeometry(String),

    /// LAS/LAZ file I/O error.
    ///
    /// This error wraps errors from the `las` crate when reading or
    /// writing point cloud files.
    #[error("LAS/LAZ file error: {0}")]
    PointCloudIO(#[from] las::Error),

    /// No tiles found intersecting the extraction geometry.
    ///
    /// This error occurs when the spatial query returns no tiles that
    /// intersect with the specified extraction geometry.
    #[error("No tiles found intersecting the extraction geometry")]
    NoTilesFound,

    /// Output file creation error.
    ///
    /// This error occurs when the output LAS/LAZ file cannot be created
    /// or written to.
    #[error("Output file error: {0}")]
    OutputError(String),

    /// General I/O error.
    ///
    /// This error wraps standard I/O errors that occur during file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience result type alias for the library.
///
/// This type alias is used throughout the library to simplify error handling.
/// It represents operations that can fail with a [`TileIndexError`].
///
/// # Example
///
/// ```no_run
/// use pointslicer_core::Result;
///
/// fn process_tiles() -> Result<()> {
///     // Operations that can fail...
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, TileIndexError>;
