use crate::error::Result;
use las::{Point, Read};
use std::path::Path;

/// Wrapper for reading LAS/LAZ files.
///
/// This struct provides a convenient interface for reading point cloud data
/// from LAS or LAZ files, with automatic decompression for LAZ files.
///
/// # Examples
///
/// ```no_run
/// use pointslicer_core::pointcloud::PointCloudReader;
///
/// # fn main() -> Result<(), pointslicer_core::TileIndexError> {
/// // Open a LAS/LAZ file
/// let mut reader = PointCloudReader::open("input.laz")?;
///
/// // Get file header information
/// let header = reader.header();
/// println!("Point format: {:?}", header.point_format());
/// println!("Number of points: {}", header.number_of_points());
///
/// // Read all points
/// let points = reader.points()?;
/// println!("Read {} points", points.len());
/// # Ok(())
/// # }
/// ```
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
    use las::{Builder, Point, Write, Writer};
    use tempfile::TempDir;

    fn create_test_las_file(
        path: &std::path::Path,
        point_count: usize,
    ) -> crate::error::Result<()> {
        let mut builder = Builder::from((1, 4));
        builder.point_format = las::point::Format::new(0)?;
        let header = builder.into_header()?;

        let mut writer = Writer::from_path(path, header)?;
        for i in 0..point_count {
            let point = Point {
                x: i as f64,
                y: i as f64,
                z: i as f64,
                ..Default::default()
            };
            writer.write(point)?;
        }
        writer.close()?;
        Ok(())
    }

    #[test]
    fn test_read_las_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.las");

        // Create a test LAS file with 10 points
        create_test_las_file(&test_file, 10).unwrap();

        // Read the file
        let mut reader = PointCloudReader::open(&test_file).unwrap();

        // Verify header
        assert_eq!(reader.header().number_of_points(), 10);

        // Verify we can read points
        let points = reader.points().unwrap();
        assert_eq!(points.len(), 10);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = PointCloudReader::open("/nonexistent/file.las");
        assert!(result.is_err());
    }
}
