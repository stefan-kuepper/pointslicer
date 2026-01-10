use crate::error::{Result, TileIndexError};
use las::{Builder, Header, Point, Write};
use std::io::BufWriter;
use std::path::Path;

/// Wrapper for writing LAS/LAZ files with format preservation.
///
/// This struct provides a convenient interface for writing point cloud data
/// to LAS or LAZ files, with automatic compression for LAZ files and
/// preservation of the original point format.
///
/// # Examples
///
/// ```no_run
/// use pointslicer_core::pointcloud::{PointCloudReader, PointCloudWriter};
///
/// # fn main() -> Result<(), pointslicer_core::TileIndexError> {
/// // Read points from a source file
/// let mut reader = PointCloudReader::open("source.laz")?;
/// let points = reader.points()?;
///
/// // Create a new file with the same header (preserves point format)
/// let mut writer = PointCloudWriter::create("output.laz", reader.header().clone())?;
///
/// // Write points
/// writer.write_points(&points)?;
/// # Ok(())
/// # }
/// ```
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

        let header = builder
            .into_header()
            .map_err(|e| TileIndexError::OutputError(format!("Failed to create header: {}", e)))?;

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
    use las::{Builder, Point, Read, Reader};
    use tempfile::TempDir;

    #[test]
    fn test_write_and_read_back() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_output.las");

        // Create header
        let mut builder = Builder::from((1, 4));
        builder.point_format = las::point::Format::new(0).unwrap();
        let header = builder.into_header().unwrap();

        // Write some points
        let mut writer = PointCloudWriter::create(&test_file, header).unwrap();
        let mut test_points = Vec::new();
        for i in 0..5 {
            let point = Point {
                x: i as f64 * 10.0,
                y: i as f64 * 20.0,
                z: i as f64 * 30.0,
                ..Default::default()
            };
            test_points.push(point);
        }
        writer.write_points(&test_points).unwrap();
        writer.close().unwrap();

        // Read back and verify
        let mut reader = Reader::from_path(&test_file).unwrap();
        assert_eq!(reader.header().number_of_points(), 5);

        let read_points: Vec<_> = reader.points().map(|p| p.unwrap()).collect();
        assert_eq!(read_points.len(), 5);
        for (i, point) in read_points.iter().enumerate() {
            assert_eq!(point.x, i as f64 * 10.0);
            assert_eq!(point.y, i as f64 * 20.0);
            assert_eq!(point.z, i as f64 * 30.0);
        }
    }

    #[test]
    fn test_write_laz_compression() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_output.laz");

        // Create and write
        let mut builder = Builder::from((1, 4));
        builder.point_format = las::point::Format::new(0).unwrap();
        let header = builder.into_header().unwrap();

        let mut writer = PointCloudWriter::create(&test_file, header).unwrap();
        let point = Point {
            x: 100.0,
            y: 200.0,
            z: 300.0,
            ..Default::default()
        };
        writer.write_points(&[point]).unwrap();
        writer.close().unwrap();

        // Verify file was created and is readable
        assert!(test_file.exists());
        let reader = Reader::from_path(&test_file).unwrap();
        assert_eq!(reader.header().number_of_points(), 1);
    }

    #[test]
    fn test_create_with_default_las() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_output.las");

        // Create writer with default header
        let mut writer = PointCloudWriter::create_with_default(&test_file).unwrap();

        // Write some points
        let point = Point {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            ..Default::default()
        };
        writer.write_point(point).unwrap();
        writer.close().unwrap();

        // Verify file was created
        assert!(test_file.exists());
        let reader = Reader::from_path(&test_file).unwrap();
        assert_eq!(reader.header().number_of_points(), 1);
    }

    #[test]
    fn test_create_with_default_laz() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_output.laz");

        // Create writer with default header for LAZ file
        let mut writer = PointCloudWriter::create_with_default(&test_file).unwrap();

        // Write some points
        let point = Point {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            ..Default::default()
        };
        writer.write_point(point).unwrap();
        writer.close().unwrap();

        // Verify file was created
        assert!(test_file.exists());
        let reader = Reader::from_path(&test_file).unwrap();
        assert_eq!(reader.header().number_of_points(), 1);
    }

    #[test]
    fn test_write_empty_points() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_empty.las");

        let mut builder = Builder::from((1, 4));
        builder.point_format = las::point::Format::new(0).unwrap();
        let header = builder.into_header().unwrap();

        let mut writer = PointCloudWriter::create(&test_file, header).unwrap();

        // Write empty slice
        writer.write_points(&[]).unwrap();
        writer.close().unwrap();

        // Verify file was created with 0 points
        assert!(test_file.exists());
        let reader = Reader::from_path(&test_file).unwrap();
        assert_eq!(reader.header().number_of_points(), 0);
    }

    #[test]
    fn test_header_access() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_header.las");

        let mut builder = Builder::from((1, 4));
        builder.point_format = las::point::Format::new(0).unwrap();
        let header = builder.into_header().unwrap();

        let writer = PointCloudWriter::create(&test_file, header).unwrap();

        // Test header access
        let writer_header = writer.header();
        assert_eq!(writer_header.version().major, 1);
        assert_eq!(writer_header.version().minor, 4);

        writer.close().unwrap();
    }
}
