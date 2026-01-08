use geo::Rect;
use std::path::PathBuf;

/// Information about a tile from the GeoPackage index
#[derive(Debug, Clone)]
pub struct TileInfo {
    /// Path to the LAS/LAZ file
    pub file_path: PathBuf,

    /// 2D bounding box of the tile (from GeoPackage geometry)
    pub bounds: Rect<f64>,

    /// Optional metadata
    pub metadata: Option<TileMetadata>,
}

/// Optional metadata for a tile
#[derive(Debug, Clone)]
pub struct TileMetadata {
    /// Number of points in this tile (if available)
    pub point_count: Option<u64>,

    /// Spatial reference system (if available)
    pub srs: Option<String>,
}

impl TileInfo {
    /// Create a new TileInfo with just path and bounds
    pub fn new(file_path: PathBuf, bounds: Rect<f64>) -> Self {
        Self {
            file_path,
            bounds,
            metadata: None,
        }
    }

    /// Create a new TileInfo with metadata
    pub fn with_metadata(file_path: PathBuf, bounds: Rect<f64>, metadata: TileMetadata) -> Self {
        Self {
            file_path,
            bounds,
            metadata: Some(metadata),
        }
    }
}
