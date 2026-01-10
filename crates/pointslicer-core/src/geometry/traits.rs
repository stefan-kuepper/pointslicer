use geo::Rect;

/// Trait for geometries that can be used to extract points from tiles.
///
/// This trait defines the interface that all extraction geometries must implement.
/// It provides methods for spatial queries that are used by the extraction
/// pipeline to filter points efficiently.
///
/// # Requirements
///
/// Implementations must be `Send + Sync` to support parallel processing
/// with Rayon.
///
/// # Examples
///
/// ```
/// use pointslicer_core::geometry::ExtractGeometry;
/// use geo::{Rect, coord};
///
/// struct MyGeometry {
///     center_x: f64,
///     center_y: f64,
///     radius: f64,
/// }
///
/// impl ExtractGeometry for MyGeometry {
///     fn bounding_box(&self) -> Rect<f64> {
///         Rect::new(
///             coord! { x: self.center_x - self.radius, y: self.center_y - self.radius },
///             coord! { x: self.center_x + self.radius, y: self.center_y + self.radius },
///         )
///     }
///     
///     fn contains_xy(&self, x: f64, y: f64) -> bool {
///         let dx = x - self.center_x;
///         let dy = y - self.center_y;
///         dx * dx + dy * dy <= self.radius * self.radius
///     }
///     
///     fn contains_xyz(&self, x: f64, y: f64, _z: f64) -> bool {
///         self.contains_xy(x, y)
///     }
///     
///     fn intersects_rect(&self, rect: &Rect<f64>) -> bool {
///         // Find the closest point on the rectangle to the circle's center
///         let closest_x = self.center_x.clamp(rect.min().x, rect.max().x);
///         let closest_y = self.center_y.clamp(rect.min().y, rect.max().y);
///         
///         // Check if this closest point is within the circle
///         self.contains_xy(closest_x, closest_y)
///     }
/// }
/// ```
pub trait ExtractGeometry: Send + Sync {
    /// Get the 2D bounding box of this geometry.
    ///
    /// This method returns the axis-aligned bounding rectangle that completely
    /// contains the geometry. It is used for spatial indexing to quickly
    /// identify which tiles might intersect with the geometry.
    ///
    /// # Returns
    ///
    /// A [`Rect<f64>`] representing the geometry's bounding box.
    fn bounding_box(&self) -> Rect<f64>;

    /// Check if a 2D point (x, y) is within this geometry.
    ///
    /// This method tests whether a point with the given X and Y coordinates
    /// lies inside the geometry, ignoring the Z coordinate.
    ///
    /// # Parameters
    ///
    /// - `x`: X coordinate of the point
    /// - `y`: Y coordinate of the point
    ///
    /// # Returns
    ///
    /// `true` if the point is inside the geometry, `false` otherwise.
    fn contains_xy(&self, x: f64, y: f64) -> bool;

    /// Check if a 3D point (x, y, z) is within this geometry.
    ///
    /// This method tests whether a point with the given X, Y, and Z coordinates
    /// lies inside the geometry. For 2D geometries, this typically ignores
    /// the Z coordinate.
    ///
    /// # Parameters
    ///
    /// - `x`: X coordinate of the point
    /// - `y`: Y coordinate of the point
    /// - `z`: Z coordinate of the point
    ///
    /// # Returns
    ///
    /// `true` if the point is inside the geometry, `false` otherwise.
    fn contains_xyz(&self, x: f64, y: f64, z: f64) -> bool;

    /// Check if this geometry intersects with a rectangle (tile bounds).
    ///
    /// This method tests whether the geometry intersects with the given
    /// rectangle. It is used to determine which tiles need to be processed
    /// for point extraction.
    ///
    /// # Parameters
    ///
    /// - `rect`: The rectangle to test for intersection
    ///
    /// # Returns
    ///
    /// `true` if the geometry intersects the rectangle, `false` otherwise.
    fn intersects_rect(&self, rect: &Rect<f64>) -> bool;
}
