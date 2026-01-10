use geo::{Rect, coord};
use las::{Builder, Point, Write, Writer};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Simple test point for fixture generation
#[derive(Debug, Clone, Copy)]
pub struct TestPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Specification for a test tile
#[derive(Debug, Clone)]
pub struct TileSpec {
    pub file_name: String,
    pub bounds: Rect<f64>,
    pub points: Vec<TestPoint>,
}

/// Test fixture with GeoPackage index and LAZ files
pub struct TestFixture {
    #[allow(dead_code)]
    pub temp_dir: TempDir, // Kept alive to prevent temp directory cleanup
    pub index_path: PathBuf,
    #[allow(dead_code)]
    pub tile_paths: Vec<PathBuf>, // Available for tests that need direct tile access
    pub output_path: PathBuf,
}

impl TestFixture {
    /// Create a test fixture with specified tiles
    pub fn new(tiles: Vec<TileSpec>) -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let index_path = temp_dir.path().join("test_index.gpkg");
        let output_path = temp_dir.path().join("output.laz");

        // Create LAZ files for each tile
        let mut tile_paths = Vec::new();
        for tile in &tiles {
            let tile_path = temp_dir.path().join(&tile.file_name);
            create_test_laz_file(&tile_path, &tile.points, 0)?;
            tile_paths.push(tile_path);
        }

        // Create GeoPackage index referencing the tiles
        create_test_geopackage(&index_path, &tiles, temp_dir.path())?;

        Ok(Self {
            temp_dir,
            index_path,
            tile_paths,
            output_path,
        })
    }

    /// Create a standard fixture with 2 overlapping tiles
    pub fn standard() -> anyhow::Result<Self> {
        let tile1 = TileSpec {
            file_name: "tile1.laz".to_string(),
            bounds: Rect::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 100.0, y: 100.0 }),
            points: generate_grid_points(0.0, 0.0, 100.0, 100.0, 10, 0.0, 50.0),
        };

        let tile2 = TileSpec {
            file_name: "tile2.laz".to_string(),
            bounds: Rect::new(coord! { x: 90.0, y: 90.0 }, coord! { x: 200.0, y: 200.0 }),
            points: generate_grid_points(90.0, 90.0, 200.0, 200.0, 10, 40.0, 100.0),
        };

        Self::new(vec![tile1, tile2])
    }

    /// Create a fixture with many tiles for parallel processing tests
    #[allow(dead_code)]
    pub fn multi_tile(count: usize) -> anyhow::Result<Self> {
        let mut tiles = Vec::new();
        let tile_size = 100.0;

        for i in 0..count {
            let x_offset = (i as f64) * tile_size * 0.8; // 20% overlap
            let y_offset = 0.0;

            let tile = TileSpec {
                file_name: format!("tile{}.laz", i + 1),
                bounds: Rect::new(
                    coord! { x: x_offset, y: y_offset },
                    coord! { x: x_offset + tile_size, y: y_offset + tile_size },
                ),
                points: generate_grid_points(
                    x_offset,
                    y_offset,
                    x_offset + tile_size,
                    y_offset + tile_size,
                    5, // Smaller grid for performance
                    0.0,
                    50.0,
                ),
            };
            tiles.push(tile);
        }

        Self::new(tiles)
    }
}

/// Create a minimal GeoPackage with tile index
fn create_test_geopackage(path: &Path, tiles: &[TileSpec], temp_dir: &Path) -> anyhow::Result<()> {
    let conn = Connection::open(path)?;

    // Create required GeoPackage metadata tables
    conn.execute(
        "CREATE TABLE gpkg_contents (
            table_name TEXT PRIMARY KEY,
            data_type TEXT,
            identifier TEXT,
            description TEXT,
            last_change TEXT,
            min_x REAL,
            min_y REAL,
            max_x REAL,
            max_y REAL,
            srs_id INTEGER
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE gpkg_geometry_columns (
            table_name TEXT,
            column_name TEXT,
            geometry_type_name TEXT,
            srs_id INTEGER,
            z INTEGER,
            m INTEGER
        )",
        [],
    )?;

    // Create feature table for tiles
    conn.execute(
        "CREATE TABLE tiles (
            id INTEGER PRIMARY KEY,
            location TEXT,
            geom BLOB
        )",
        [],
    )?;

    // Register in gpkg_contents
    conn.execute(
        "INSERT INTO gpkg_contents VALUES (
            'tiles', 'features', 'tiles', 'Tile index', datetime('now'),
            0, 0, 0, 0, 0
        )",
        [],
    )?;

    // Register geometry column
    conn.execute(
        "INSERT INTO gpkg_geometry_columns VALUES (
            'tiles', 'geom', 'POLYGON', 0, 0, 0
        )",
        [],
    )?;

    // Insert tiles
    for (i, tile) in tiles.iter().enumerate() {
        let location = temp_dir.join(&tile.file_name);
        let location_str = location.to_str().unwrap();

        // Create GeoPackage Binary Format blob for the tile bounds
        let geom_blob = create_geopackage_geometry_blob(&tile.bounds);

        conn.execute(
            "INSERT INTO tiles (id, location, geom) VALUES (?1, ?2, ?3)",
            rusqlite::params![i + 1, location_str, geom_blob],
        )?;
    }

    Ok(())
}

/// Create a GeoPackage Binary Format geometry blob with envelope type 1 (XY)
fn create_geopackage_geometry_blob(bounds: &Rect<f64>) -> Vec<u8> {
    let mut blob = Vec::new();

    // Header: magic bytes "GP"
    blob.push(0x47); // 'G'
    blob.push(0x50); // 'P'

    // Version (0)
    blob.push(0x00);

    // Flags: envelope type 1 (XY), binary type 0 (standard WKB)
    // Envelope type is in bits 1-3, so type 1 = 0b00000010 = 0x02
    blob.push(0x02);

    // SRS ID (4 bytes, little-endian, 0 = undefined)
    blob.extend_from_slice(&0u32.to_le_bytes());

    // Envelope: min_x, max_x, min_y, max_y (4 doubles, little-endian)
    blob.extend_from_slice(&bounds.min().x.to_le_bytes());
    blob.extend_from_slice(&bounds.max().x.to_le_bytes());
    blob.extend_from_slice(&bounds.min().y.to_le_bytes());
    blob.extend_from_slice(&bounds.max().y.to_le_bytes());

    // We don't need actual WKB geometry for the tile index reader,
    // as it only uses the envelope. But include minimal WKB for completeness.
    // WKB for a simple point at bounds center (for validity)
    blob.push(0x01); // Little endian
    blob.extend_from_slice(&1u32.to_le_bytes()); // Point type
    let center_x = (bounds.min().x + bounds.max().x) / 2.0;
    let center_y = (bounds.min().y + bounds.max().y) / 2.0;
    blob.extend_from_slice(&center_x.to_le_bytes());
    blob.extend_from_slice(&center_y.to_le_bytes());

    blob
}

/// Create a minimal LAS/LAZ file with test points
pub fn create_test_laz_file(
    path: &Path,
    points: &[TestPoint],
    point_format: u8,
) -> anyhow::Result<()> {
    if points.is_empty() {
        return Err(anyhow::anyhow!("Cannot create LAZ file with no points"));
    }

    // Calculate bounds from points
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    let mut min_z = f64::MAX;
    let mut max_z = f64::MIN;

    for p in points {
        min_x = min_x.min(p.x);
        max_x = max_x.max(p.x);
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
        min_z = min_z.min(p.z);
        max_z = max_z.max(p.z);
    }

    // Create LAS header
    let mut builder = Builder::from((1, 4));
    builder.point_format = las::point::Format::new(point_format)?;

    let header = builder.into_header()?;

    // Create writer (automatically handles .laz compression)
    let mut writer = Writer::from_path(path, header)?;

    // Write points
    for test_point in points {
        let mut point = Point::default();
        point.x = test_point.x;
        point.y = test_point.y;
        point.z = test_point.z;
        writer.write(point)?;
    }

    writer.close()?;

    Ok(())
}

/// Generate a grid of points within bounds
pub fn generate_grid_points(
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
    grid_size: usize,
    min_z: f64,
    max_z: f64,
) -> Vec<TestPoint> {
    let mut points = Vec::new();
    let step_x = (max_x - min_x) / (grid_size as f64);
    let step_y = (max_y - min_y) / (grid_size as f64);
    let step_z = (max_z - min_z) / (grid_size as f64);

    for i in 0..grid_size {
        for j in 0..grid_size {
            let x = min_x + (i as f64) * step_x + step_x / 2.0;
            let y = min_y + (j as f64) * step_y + step_y / 2.0;
            let z = min_z + ((i + j) % grid_size) as f64 * step_z;

            points.push(TestPoint { x, y, z });
        }
    }

    points
}
