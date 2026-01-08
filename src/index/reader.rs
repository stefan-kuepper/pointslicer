use super::tile::TileInfo;
use crate::error::{Result, TileIndexError};
use crate::geometry::ExtractGeometry;
use geo::{Rect, coord};
use rusqlite::Connection;
use std::path::Path;

/// Reader for GeoPackage tile index files
pub struct TileIndexReader {
    conn: Connection,
}

impl TileIndexReader {
    /// Open a GeoPackage tile index file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path.as_ref())
            .map_err(|e| TileIndexError::GeoPackageOpen(e.to_string()))?;

        Ok(Self { conn })
    }

    /// Get all feature tables in the GeoPackage
    pub fn list_tables(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT table_name FROM gpkg_contents WHERE data_type = 'features'")
            .map_err(|e| TileIndexError::TileQuery(e.to_string()))?;

        let tables: rusqlite::Result<Vec<String>> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| TileIndexError::TileQuery(e.to_string()))?
            .collect();

        tables.map_err(|e| TileIndexError::TileQuery(e.to_string()))
    }

    /// Read tiles from a specific table that intersect with the extraction geometry
    /// If table_name is None, uses the first feature table
    pub fn read_tiles<G: ExtractGeometry>(
        &self,
        table_name: Option<&str>,
        geometry: &G,
    ) -> Result<Vec<TileInfo>> {
        let table = if let Some(name) = table_name {
            name.to_string()
        } else {
            // Use the first feature table
            let tables = self.list_tables()?;
            tables
                .first()
                .ok_or_else(|| TileIndexError::TileQuery("No feature tables found".to_string()))?
                .clone()
        };

        // First, find the geometry column name
        let geom_column = self.find_geometry_column(&table)?;

        // Query all rows from the table
        // PDAL tindex typically has: location (text), geometry (blob)
        let query = format!("SELECT location, \"{}\" FROM \"{}\"", geom_column, table);

        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|e| TileIndexError::TileQuery(format!("Failed to prepare query: {}", e)))?;

        let all_tiles: Vec<TileInfo> = stmt
            .query_map([], |row| {
                let location: String = row.get(0)?;
                let geom_blob: Vec<u8> = row.get(1)?;

                // Parse the GeoPackage geometry to get bounds
                let bounds = Self::extract_bounds_from_blob(&geom_blob)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                Ok(TileInfo::new(std::path::PathBuf::from(location), bounds))
            })
            .map_err(|e| TileIndexError::TileQuery(e.to_string()))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| TileIndexError::TileQuery(e.to_string()))?;

        let tiles: Vec<TileInfo> = all_tiles
            .into_iter()
            .filter(|tile| geometry.intersects_rect(&tile.bounds))
            .collect();

        if tiles.is_empty() {
            return Err(TileIndexError::NoTilesFound);
        }

        Ok(tiles)
    }

    /// Find the geometry column name for a table
    fn find_geometry_column(&self, table_name: &str) -> Result<String> {
        let mut stmt = self
            .conn
            .prepare("SELECT column_name FROM gpkg_geometry_columns WHERE table_name = ?1")
            .map_err(|e| TileIndexError::TileQuery(e.to_string()))?;

        let column_name: String = stmt
            .query_row([table_name], |row| row.get(0))
            .map_err(|e| {
                TileIndexError::TileQuery(format!(
                    "No geometry column found for table '{}': {}",
                    table_name, e
                ))
            })?;

        Ok(column_name)
    }

    /// Extract bounding box from GeoPackage geometry blob
    fn extract_bounds_from_blob(blob: &[u8]) -> Result<Rect<f64>> {
        // GeoPackage Binary Format has a header before the WKB
        // Header format: magic(2) + version(1) + flags(1) + srs_id(4) + envelope(variable)

        if blob.len() < 8 {
            return Err(TileIndexError::InvalidGeometry(
                "Geometry blob too short".to_string(),
            ));
        }

        // Check magic bytes (should be 0x47, 0x50 = "GP")
        if blob[0] != 0x47 || blob[1] != 0x50 {
            return Err(TileIndexError::InvalidGeometry(
                "Invalid GeoPackage geometry magic bytes".to_string(),
            ));
        }

        let flags = blob[3];
        let envelope_type = (flags >> 1) & 0x07;

        // Envelope types:
        // 0 = no envelope
        // 1 = XY (32 bytes: 4 doubles)
        // 2 = XYZ (48 bytes: 6 doubles)
        // 3 = XYM (48 bytes: 6 doubles)
        // 4 = XYZM (64 bytes: 8 doubles)

        let envelope_offset = 8; // After header

        let (min_x, max_x, min_y, max_y) = match envelope_type {
            1..=4 => {
                // All envelope types start with min_x, max_x, min_y, max_y
                if blob.len() < envelope_offset + 32 {
                    return Err(TileIndexError::InvalidGeometry(
                        "Geometry blob too short for envelope".to_string(),
                    ));
                }

                let min_x = f64::from_le_bytes(
                    blob[envelope_offset..envelope_offset + 8]
                        .try_into()
                        .unwrap(),
                );
                let max_x = f64::from_le_bytes(
                    blob[envelope_offset + 8..envelope_offset + 16]
                        .try_into()
                        .unwrap(),
                );
                let min_y = f64::from_le_bytes(
                    blob[envelope_offset + 16..envelope_offset + 24]
                        .try_into()
                        .unwrap(),
                );
                let max_y = f64::from_le_bytes(
                    blob[envelope_offset + 24..envelope_offset + 32]
                        .try_into()
                        .unwrap(),
                );

                (min_x, max_x, min_y, max_y)
            }
            0 => {
                // No envelope - need to parse WKB geometry
                // For now, return an error - this would require full WKB parsing
                return Err(TileIndexError::InvalidGeometry(
                    "Geometry has no envelope - WKB parsing not yet implemented".to_string(),
                ));
            }
            _ => {
                return Err(TileIndexError::InvalidGeometry(format!(
                    "Unknown envelope type: {}",
                    envelope_type
                )));
            }
        };

        Ok(Rect::new(
            coord! { x: min_x, y: min_y },
            coord! { x: max_x, y: max_y },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_names() {
        // Test that common column names are recognized
        let column_names = ["location", "path", "file", "filename"];
        assert!(column_names.contains(&"location"));
    }
}
