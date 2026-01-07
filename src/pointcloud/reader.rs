use crate::error::Result;
use las::{Point, Read};
use std::path::Path;

/// Wrapper for reading LAS/LAZ files
pub struct PointCloudReader<'a> {
    reader: las::Reader<'a>,
}

impl<'a> PointCloudReader<'a> {
    /// Open a LAS or LAZ file for reading
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let reader = las::Reader::from_path(path)?;
        Ok(Self { reader })
    }

    /// Get the header from the LAS file
    pub fn header(&self) -> &las::Header {
        self.reader.header()
    }

    /// Read all points from the file
    pub fn points(&mut self) -> Result<Vec<Point>> {
        let points: std::result::Result<Vec<_>, _> = self.reader.points().collect();
        Ok(points?)
    }

    /// Iterate over points
    pub fn iter_points(&mut self) -> impl Iterator<Item = las::Result<Point>> + '_ {
        self.reader.points()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader_construction() {
        // This test just ensures the API compiles
        // Real testing would require a sample LAS file
    }
}
