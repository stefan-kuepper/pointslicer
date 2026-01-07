use geo::Rect;

/// Trait for geometries that can be used to extract points from tiles
pub trait ExtractGeometry: Send + Sync {
    /// Get the 2D bounding box of this geometry (for spatial indexing)
    fn bounding_box(&self) -> Rect<f64>;

    /// Check if a 2D point (x, y) is within this geometry
    fn contains_xy(&self, x: f64, y: f64) -> bool;

    /// Check if a 3D point (x, y, z) is within this geometry
    fn contains_xyz(&self, x: f64, y: f64, z: f64) -> bool;

    /// Check if this geometry intersects with a rectangle (tile bounds)
    fn intersects_rect(&self, rect: &Rect<f64>) -> bool;
}
