use pointslicer_core::geometry::{BoundingBox, ExtractGeometry, VerticalCylinder};
use pointslicer_core::pipeline::{ExtractionPipeline, ExtractionStats};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Python wrapper for extraction statistics.
#[pyclass(name = "ExtractionStats")]
#[derive(Debug, Clone)]
pub struct PyExtractionStats {
    /// Number of tiles that were successfully processed.
    #[pyo3(get)]
    pub tiles_processed: usize,

    /// Total number of points read from all tiles.
    #[pyo3(get)]
    pub points_read: u64,

    /// Number of points that passed the geometry filter.
    #[pyo3(get)]
    pub points_written: u64,

    /// Total time taken for the extraction in seconds.
    #[pyo3(get)]
    pub elapsed_time: f64,
}

impl From<ExtractionStats> for PyExtractionStats {
    fn from(stats: ExtractionStats) -> Self {
        Self {
            tiles_processed: stats.tiles_processed,
            points_read: stats.points_read,
            points_written: stats.points_written,
            elapsed_time: stats.elapsed_time.as_secs_f64(),
        }
    }
}

#[pymethods]
impl PyExtractionStats {
    fn __repr__(&self) -> String {
        format!(
            "ExtractionStats(tiles_processed={}, points_read={}, points_written={}, elapsed_time={:.2}s)",
            self.tiles_processed, self.points_read, self.points_written, self.elapsed_time
        )
    }

    fn __str__(&self) -> String {
        format!(
            "Extracted {} points from {} tiles in {:.2} seconds",
            self.points_written, self.tiles_processed, self.elapsed_time
        )
    }
}

/// Python wrapper for a 2D/3D bounding box geometry.
#[pyclass(name = "BoundingBox")]
#[derive(Debug, Clone)]
pub struct PyBoundingBox {
    inner: BoundingBox,
}

#[pymethods]
impl PyBoundingBox {
    /// Create a new 2D bounding box.
    ///
    /// Args:
    ///     min_x: Minimum X coordinate
    ///     max_x: Maximum X coordinate
    ///     min_y: Minimum Y coordinate
    ///     max_y: Maximum Y coordinate
    ///     min_z: Optional minimum Z coordinate (for 3D extraction)
    ///     max_z: Optional maximum Z coordinate (for 3D extraction)
    ///
    /// Returns:
    ///     BoundingBox: A new bounding box geometry
    #[new]
    #[pyo3(signature = (min_x, max_x, min_y, max_y, min_z = None, max_z = None))]
    fn new(
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
        min_z: Option<f64>,
        max_z: Option<f64>,
    ) -> PyResult<Self> {
        if min_x >= max_x {
            return Err(PyValueError::new_err("min_x must be less than max_x"));
        }
        if min_y >= max_y {
            return Err(PyValueError::new_err("min_y must be less than max_y"));
        }
        if let (Some(min_z_val), Some(max_z_val)) = (min_z, max_z)
            && min_z_val >= max_z_val {
                return Err(PyValueError::new_err("min_z must be less than max_z"));
            }

        let inner = BoundingBox {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        };

        Ok(Self { inner })
    }

    /// Create a new 2D bounding box (convenience method).
    ///
    /// Args:
    ///     min_x: Minimum X coordinate
    ///     max_x: Maximum X coordinate
    ///     min_y: Minimum Y coordinate
    ///     max_y: Maximum Y coordinate
    ///
    /// Returns:
    ///     BoundingBox: A new 2D bounding box geometry
    #[staticmethod]
    fn new_2d(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> PyResult<Self> {
        Self::new(min_x, max_x, min_y, max_y, None, None)
    }

    /// Create a new 3D bounding box (convenience method).
    ///
    /// Args:
    ///     min_x: Minimum X coordinate
    ///     max_x: Maximum X coordinate
    ///     min_y: Minimum Y coordinate
    ///     max_y: Maximum Y coordinate
    ///     min_z: Minimum Z coordinate
    ///     max_z: Maximum Z coordinate
    ///
    /// Returns:
    ///     BoundingBox: A new 3D bounding box geometry
    #[staticmethod]
    fn new_3d(
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
        min_z: f64,
        max_z: f64,
    ) -> PyResult<Self> {
        Self::new(min_x, max_x, min_y, max_y, Some(min_z), Some(max_z))
    }

    fn __repr__(&self) -> String {
        match (self.inner.min_z, self.inner.max_z) {
            (Some(min_z), Some(max_z)) => format!(
                "BoundingBox(min_x={}, max_x={}, min_y={}, max_y={}, min_z={}, max_z={})",
                self.inner.min_x,
                self.inner.max_x,
                self.inner.min_y,
                self.inner.max_y,
                min_z,
                max_z
            ),
            _ => format!(
                "BoundingBox(min_x={}, max_x={}, min_y={}, max_y={})",
                self.inner.min_x, self.inner.max_x, self.inner.min_y, self.inner.max_y
            ),
        }
    }

    /// Check if a point is inside the bounding box.
    ///
    /// Args:
    ///     x: X coordinate
    ///     y: Y coordinate
    ///     z: Optional Z coordinate (only checked for 3D boxes)
    ///
    /// Returns:
    ///     bool: True if the point is inside the bounding box
    #[pyo3(signature = (x, y, z = None))]
    fn contains(&self, x: f64, y: f64, z: Option<f64>) -> bool {
        if let Some(z_val) = z {
            ExtractGeometry::contains_xyz(&self.inner, x, y, z_val)
        } else {
            ExtractGeometry::contains_xy(&self.inner, x, y)
        }
    }

    #[getter]
    fn min_x(&self) -> f64 {
        self.inner.min_x
    }

    #[getter]
    fn max_x(&self) -> f64 {
        self.inner.max_x
    }

    #[getter]
    fn min_y(&self) -> f64 {
        self.inner.min_y
    }

    #[getter]
    fn max_y(&self) -> f64 {
        self.inner.max_y
    }

    #[getter]
    fn min_z(&self) -> Option<f64> {
        self.inner.min_z
    }

    #[getter]
    fn max_z(&self) -> Option<f64> {
        self.inner.max_z
    }
}

impl From<PyBoundingBox> for BoundingBox {
    fn from(py_bbox: PyBoundingBox) -> Self {
        py_bbox.inner
    }
}

/// Python wrapper for a vertical cylinder geometry.
#[pyclass(name = "Cylinder")]
#[derive(Debug, Clone)]
pub struct PyCylinder {
    inner: VerticalCylinder,
}

#[pymethods]
impl PyCylinder {
    /// Create a new vertical cylinder from center coordinates and radius.
    ///
    /// Args:
    ///     center_x: X coordinate of the cylinder center
    ///     center_y: Y coordinate of the cylinder center
    ///     radius: Radius of the cylinder
    ///
    /// Returns:
    ///     Cylinder: A new vertical cylinder geometry
    #[new]
    fn new(center_x: f64, center_y: f64, radius: f64) -> PyResult<Self> {
        if radius <= 0.0 {
            return Err(PyValueError::new_err("radius must be positive"));
        }

        let inner = VerticalCylinder {
            center_x,
            center_y,
            radius,
        };

        Ok(Self { inner })
    }

    /// Create a new vertical cylinder from center coordinates and diameter.
    ///
    /// Args:
    ///     center_x: X coordinate of the cylinder center
    ///     center_y: Y coordinate of the cylinder center
    ///     diameter: Diameter of the cylinder
    ///
    /// Returns:
    ///     Cylinder: A new vertical cylinder geometry
    #[staticmethod]
    fn from_diameter(center_x: f64, center_y: f64, diameter: f64) -> PyResult<Self> {
        if diameter <= 0.0 {
            return Err(PyValueError::new_err("diameter must be positive"));
        }

        let inner = VerticalCylinder::from_diameter(center_x, center_y, diameter);
        Ok(Self { inner })
    }

    fn __repr__(&self) -> String {
        format!(
            "Cylinder(center_x={}, center_y={}, radius={})",
            self.inner.center_x, self.inner.center_y, self.inner.radius
        )
    }

    /// Check if a point is inside the cylinder.
    ///
    /// Args:
    ///     x: X coordinate
    ///     y: Y coordinate
    ///     z: Z coordinate (ignored for vertical cylinders)
    ///
    /// Returns:
    ///     bool: True if the point is inside the cylinder
    #[pyo3(signature = (x, y, z = 0.0))]
    fn contains(&self, x: f64, y: f64, z: f64) -> bool {
        ExtractGeometry::contains_xyz(&self.inner, x, y, z)
    }

    #[getter]
    fn center_x(&self) -> f64 {
        self.inner.center_x
    }

    #[getter]
    fn center_y(&self) -> f64 {
        self.inner.center_y
    }

    #[getter]
    fn radius(&self) -> f64 {
        self.inner.radius
    }

    #[getter]
    fn diameter(&self) -> f64 {
        self.inner.radius * 2.0
    }
}

impl From<PyCylinder> for VerticalCylinder {
    fn from(py_cylinder: PyCylinder) -> Self {
        py_cylinder.inner
    }
}

/// Extract points from LAS/LAZ files using a GeoPackage tile index.
///
/// Args:
///     index_path: Path to the GeoPackage tile index file
///     output_path: Path where the extracted points will be written
///     geometry: Extraction geometry (BoundingBox or Cylinder)
///     verbose: Whether to log detailed progress information (default: False)
///
/// Returns:
///     ExtractionStats: Statistics about the extraction process
///
/// Raises:
///     ValueError: If the geometry is not valid
///     RuntimeError: If the extraction fails
///
/// Example:
///     ```python
///     import pointslicer
///
///     # Create a bounding box
///     bbox = pointslicer.BoundingBox(10000, 20000, 30000, 40000)
///
///     # Extract points
///     stats = pointslicer.extract("tiles.gpkg", "output.laz", bbox, verbose=True)
///     print(f"Extracted {stats.points_written} points")
///     ```
#[pyfunction]
#[pyo3(signature = (index_path, output_path, geometry, verbose = false))]
fn extract(
    index_path: String,
    output_path: String,
    geometry: Bound<'_, PyAny>,
    verbose: bool,
) -> PyResult<PyExtractionStats> {
    // Check if it's a BoundingBox
    if let Ok(py_bbox) = geometry.extract::<PyBoundingBox>() {
        let bbox: BoundingBox = py_bbox.into();
        let pipeline = ExtractionPipeline::new(&index_path, &output_path, verbose);
        match pipeline.execute(&bbox) {
            Ok(stats) => Ok(stats.into()),
            Err(e) => Err(PyValueError::new_err(format!("Extraction failed: {}", e))),
        }
    }
    // Check if it's a Cylinder
    else if let Ok(py_cylinder) = geometry.extract::<PyCylinder>() {
        let cylinder: VerticalCylinder = py_cylinder.into();
        let pipeline = ExtractionPipeline::new(&index_path, &output_path, verbose);
        match pipeline.execute(&cylinder) {
            Ok(stats) => Ok(stats.into()),
            Err(e) => Err(PyValueError::new_err(format!("Extraction failed: {}", e))),
        }
    }
    // Unknown geometry type
    else {
        Err(PyValueError::new_err(
            "geometry must be either BoundingBox or Cylinder",
        ))
    }
}

/// The pointslicer Python module.
///
/// This module provides Python bindings for the pointslicer library,
/// allowing you to extract points from LAS/LAZ files using GeoPackage
/// tile indices.
#[pymodule]
fn pointslicer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyExtractionStats>()?;
    m.add_class::<PyBoundingBox>()?;
    m.add_class::<PyCylinder>()?;
    m.add_function(wrap_pyfunction!(extract, m)?)?;

    // Add module documentation
    m.add("__doc__", "Python bindings for pointslicer - extract points from LAS/LAZ files using GeoPackage tile indices")?;

    // Add version
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
