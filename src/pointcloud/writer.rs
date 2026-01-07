use crate::error::{Result, TileIndexError};
use las::{Builder, Header, Point, Write};
use std::io::BufWriter;
use std::path::Path;

/// Wrapper for writing LAS/LAZ files
pub struct PointCloudWriter<W: 'static + std::io::Write + std::io::Seek + std::fmt::Debug + Send> {
    writer: las::Writer<W>,
}

impl PointCloudWriter<BufWriter<std::fs::File>> {
    /// Create a new LAS/LAZ file for writing
    pub fn create<P: AsRef<Path>>(path: P, header: Header) -> Result<Self> {
        let writer = las::Writer::from_path(path, header)
            .map_err(|e| TileIndexError::OutputError(e.to_string()))?;

        Ok(Self { writer })
    }

    /// Create a new LAS/LAZ file with a default header
    /// The header will be updated based on the points written
    pub fn create_with_default<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut builder = Builder::default();

        // Determine compression from file extension
        let path_ref = path.as_ref();
        if path_ref.extension().and_then(|s| s.to_str()) == Some("laz") {
            builder.point_format.is_compressed = true;
        }

        let header = builder.into_header().map_err(|e| {
            TileIndexError::OutputError(format!("Failed to create header: {}", e))
        })?;

        Self::create(path, header)
    }
}

impl<W: 'static + std::io::Write + std::io::Seek + std::fmt::Debug + Send> PointCloudWriter<W> {
    /// Write a single point to the file
    pub fn write_point(&mut self, point: Point) -> Result<()> {
        self.writer
            .write(point)
            .map_err(|e| TileIndexError::OutputError(e.to_string()))
    }

    /// Write multiple points to the file
    pub fn write_points(&mut self, points: &[Point]) -> Result<()> {
        for point in points {
            self.write_point(point.clone())?;
        }
        Ok(())
    }

    /// Close the writer (finalizes the file)
    pub fn close(mut self) -> Result<()> {
        self.writer
            .close()
            .map_err(|e| TileIndexError::OutputError(e.to_string()))
    }

    /// Get a reference to the header
    pub fn header(&self) -> &Header {
        self.writer.header()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer_construction() {
        // This test just ensures the API compiles
        // Real testing would require writing to a temp file
    }
}
