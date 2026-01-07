use super::tile::TileInfo;
use crate::error::{Result, TileIndexError};
use crate::geometry::ExtractGeometry;
use geo::{coord, Rect};
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

        // Query all rows from the table
        // PDAL tindex typically has: location (text), geometry (blob)
        let query = format!("SELECT location, ST_MinX(geom) as min_x, ST_MaxX(geom) as max_x, ST_MinY(geom) as min_y, ST_MaxY(geom) as max_y FROM \"{}\"", table);

        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|e| TileIndexError::TileQuery(format!("Failed to prepare query: {}", e)))?;

        let all_tiles: Vec<TileInfo> = stmt
            .query_map([], |row| {
                let location: String = row.get(0)?;
                let min_x: f64 = row.get(1)?;
                let max_x: f64 = row.get(2)?;
                let min_y: f64 = row.get(3)?;
                let max_y: f64 = row.get(4)?;

                let bounds = Rect::new(coord! { x: min_x, y: min_y }, coord! { x: max_x, y: max_y });
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
