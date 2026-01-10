use super::traits::ExtractGeometry;
use geo::{Rect, coord};

/// A vertical cylinder defined by center coordinates and radius.
///
/// This struct represents a vertical (Z-aligned) cylinder that can be used
/// to extract points within a circular region. The cylinder extends infinitely
/// in the Z direction.
///
/// # Examples
///
/// ```
/// use pointslicer_core::geometry::{VerticalCylinder, ExtractGeometry};
///
/// // Create a cylinder from diameter
/// let cylinder = VerticalCylinder::from_diameter(12345.0, 67890.0, 12.0);
///
/// // Create a cylinder from radius
/// let cylinder2 = VerticalCylinder::from_radius(12345.0, 67890.0, 6.0);
///
/// // Check if points are inside
/// assert!(cylinder.contains_xy(12345.0, 67890.0)); // Center point
/// assert!(cylinder.contains_xy(12351.0, 67890.0)); // 6 units right
/// assert!(!cylinder.contains_xy(12357.0, 67890.0)); // 12 units right (outside)
/// ```
#[derive(Debug, Clone)]
pub struct VerticalCylinder {
    /// X coordinate of the cylinder center.
    pub center_x: f64,

    /// Y coordinate of the cylinder center.
    pub center_y: f64,

    /// Radius of the cylinder.
    pub radius: f64,
}

impl VerticalCylinder {
    /// Create a new vertical cylinder from center coordinates and diameter
    pub fn from_diameter(center_x: f64, center_y: f64, diameter: f64) -> Self {
        Self {
            center_x,
            center_y,
            radius: diameter / 2.0,
        }
    }

    /// Create a new vertical cylinder from center coordinates and radius
    pub fn from_radius(center_x: f64, center_y: f64, radius: f64) -> Self {
        Self {
            center_x,
            center_y,
            radius,
        }
    }

    /// Calculate the squared distance from a point to the cylinder's center
    fn distance_squared(&self, x: f64, y: f64) -> f64 {
        let dx = x - self.center_x;
        let dy = y - self.center_y;
        dx * dx + dy * dy
    }
}

impl ExtractGeometry for VerticalCylinder {
    fn bounding_box(&self) -> Rect<f64> {
        Rect::new(
            coord! { x: self.center_x - self.radius, y: self.center_y - self.radius },
            coord! { x: self.center_x + self.radius, y: self.center_y + self.radius },
        )
    }

    fn contains_xy(&self, x: f64, y: f64) -> bool {
        self.distance_squared(x, y) <= self.radius * self.radius
    }

    fn contains_xyz(&self, x: f64, y: f64, _z: f64) -> bool {
        // Vertical cylinder ignores Z coordinate
        self.contains_xy(x, y)
    }

    fn intersects_rect(&self, rect: &Rect<f64>) -> bool {
        // Find the closest point on the rectangle to the circle's center
        let closest_x = self.center_x.clamp(rect.min().x, rect.max().x);
        let closest_y = self.center_y.clamp(rect.min().y, rect.max().y);

        // Check if this closest point is within the circle
        self.contains_xy(closest_x, closest_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_diameter() {
        let cyl = VerticalCylinder::from_diameter(100.0, 200.0, 20.0);
        assert_eq!(cyl.center_x, 100.0);
        assert_eq!(cyl.center_y, 200.0);
        assert_eq!(cyl.radius, 10.0);
    }

    #[test]
    fn test_contains_xy_center() {
        let cyl = VerticalCylinder::from_radius(0.0, 0.0, 10.0);
        assert!(cyl.contains_xy(0.0, 0.0));
    }

    #[test]
    fn test_contains_xy_inside() {
        let cyl = VerticalCylinder::from_radius(0.0, 0.0, 10.0);
        assert!(cyl.contains_xy(5.0, 0.0));
        assert!(cyl.contains_xy(0.0, 5.0));
        assert!(cyl.contains_xy(3.0, 4.0)); // 3-4-5 triangle
    }

    #[test]
    fn test_contains_xy_boundary() {
        let cyl = VerticalCylinder::from_radius(0.0, 0.0, 10.0);
        assert!(cyl.contains_xy(10.0, 0.0));
        assert!(cyl.contains_xy(0.0, 10.0));
        assert!(cyl.contains_xy(-10.0, 0.0));
        assert!(cyl.contains_xy(0.0, -10.0));
    }

    #[test]
    fn test_contains_xy_outside() {
        let cyl = VerticalCylinder::from_radius(0.0, 0.0, 10.0);
        assert!(!cyl.contains_xy(11.0, 0.0));
        assert!(!cyl.contains_xy(0.0, 11.0));
        assert!(!cyl.contains_xy(8.0, 8.0));
    }

    #[test]
    fn test_contains_xyz_ignores_z() {
        let cyl = VerticalCylinder::from_radius(0.0, 0.0, 10.0);
        assert!(cyl.contains_xyz(5.0, 0.0, 100.0));
        assert!(cyl.contains_xyz(5.0, 0.0, -100.0));
        assert!(!cyl.contains_xyz(11.0, 0.0, 0.0));
    }

    #[test]
    fn test_bounding_box() {
        let cyl = VerticalCylinder::from_radius(100.0, 200.0, 10.0);
        let bbox = cyl.bounding_box();
        assert_eq!(bbox.min().x, 90.0);
        assert_eq!(bbox.min().y, 190.0);
        assert_eq!(bbox.max().x, 110.0);
        assert_eq!(bbox.max().y, 210.0);
    }

    #[test]
    fn test_intersects_rect_overlapping() {
        let cyl = VerticalCylinder::from_radius(0.0, 0.0, 10.0);
        let rect = Rect::new(coord! { x: -5.0, y: -5.0 }, coord! { x: 5.0, y: 5.0 });
        assert!(cyl.intersects_rect(&rect));
    }

    #[test]
    fn test_intersects_rect_disjoint() {
        let cyl = VerticalCylinder::from_radius(0.0, 0.0, 10.0);
        let rect = Rect::new(coord! { x: 20.0, y: 20.0 }, coord! { x: 30.0, y: 30.0 });
        assert!(!cyl.intersects_rect(&rect));
    }

    #[test]
    fn test_intersects_rect_touching() {
        let cyl = VerticalCylinder::from_radius(0.0, 0.0, 10.0);
        let rect = Rect::new(coord! { x: 10.0, y: -5.0 }, coord! { x: 20.0, y: 5.0 });
        assert!(cyl.intersects_rect(&rect));
    }
}
