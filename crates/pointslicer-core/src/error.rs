use thiserror::Error;

#[derive(Error, Debug)]
pub enum TileIndexError {
    #[error("Failed to open GeoPackage: {0}")]
    GeoPackageOpen(String),

    #[error("Failed to query tiles: {0}")]
    TileQuery(String),

    #[error("Invalid geometry in tile index: {0}")]
    InvalidGeometry(String),

    #[error("LAS/LAZ file error: {0}")]
    PointCloudIO(#[from] las::Error),

    #[error("No tiles found intersecting the extraction geometry")]
    NoTilesFound,

    #[error("Output file error: {0}")]
    OutputError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, TileIndexError>;
