use geo::Rect;
use std::path::PathBuf;

/// Information about a tile from the GeoPackage index.
///
/// This struct contains the file path to a LAS/LAZ tile and its spatial
/// bounds as extracted from the GeoPackage geometry.
///
/// # Fields
///
/// - `file_path`: Path to the LAS/LAZ file
/// - `bounds`: 2D bounding box of the tile (from GeoPackage geometry)
/// - `metadata`: Optional metadata about the tile
#[derive(Debug, Clone)]
pub struct TileInfo {
    /// Path to the LAS/LAZ file.
    pub file_path: PathBuf,

    /// 2D bounding box of the tile (from GeoPackage geometry).
    pub bounds: Rect<f64>,

    /// Optional metadata about the tile.
    pub metadata: Option<TileMetadata>,
}

/// Optional metadata for a tile.
///
/// This struct contains additional information about a tile that may be
/// available in some GeoPackage indices.
///
/// # Fields
///
/// - `point_count`: Number of points in this tile (if available)
/// - `srs`: Spatial reference system (if available)
#[derive(Debug, Clone)]
pub struct TileMetadata {
    /// Number of points in this tile (if available).
    pub point_count: Option<u64>,

    /// Spatial reference system (if available).
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

#[cfg(test)]
mod tests {
    use super::*;
    use geo::coord;

    #[test]
    fn test_tile_info_new() {
        let path = PathBuf::from("test.laz");
        let bounds = Rect::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 100.0, y: 100.0 });

        let tile = TileInfo::new(path.clone(), bounds);

        assert_eq!(tile.file_path, path);
        assert_eq!(tile.bounds, bounds);
        assert!(tile.metadata.is_none());
    }

    #[test]
    fn test_tile_info_with_metadata() {
        let path = PathBuf::from("test.laz");
        let bounds = Rect::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 100.0, y: 100.0 });
        let metadata = TileMetadata {
            point_count: Some(1234),
            srs: Some("EPSG:32632".to_string()),
        };

        let tile = TileInfo::with_metadata(path.clone(), bounds, metadata.clone());

        assert_eq!(tile.file_path, path);
        assert_eq!(tile.bounds, bounds);
        assert!(tile.metadata.is_some());

        let tile_metadata = tile.metadata.unwrap();
        assert_eq!(tile_metadata.point_count, Some(1234));
        assert_eq!(tile_metadata.srs, Some("EPSG:32632".to_string()));
    }
}
