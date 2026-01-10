use super::traits::ExtractGeometry;
use geo::{Rect, coord};

/// A 2D or 3D axis-aligned bounding box for point extraction.
///
/// This struct represents a rectangular region in 2D or 3D space that can be
/// used to filter points. It supports both 2D (X,Y only) and 3D (X,Y,Z)
/// extraction.
///
/// # Examples
///
/// ```
/// use pointslicer_core::geometry::{BoundingBox, ExtractGeometry};
///
/// // Create a 2D bounding box
/// let bbox_2d = BoundingBox::new_2d(10000.0, 20000.0, 30000.0, 40000.0);
///
/// // Create a 3D bounding box with Z constraints
/// let bbox_3d = BoundingBox::new_3d(10000.0, 20000.0, 30000.0, 40000.0, 0.0, 100.0);
///
/// // Check if a point is inside
/// assert!(bbox_2d.contains_xy(15000.0, 35000.0));
/// assert!(bbox_3d.contains_xyz(15000.0, 35000.0, 50.0));
/// ```
#[derive(Debug, Clone)]
pub struct BoundingBox {
    /// Minimum X coordinate of the bounding box.
    pub min_x: f64,

    /// Maximum X coordinate of the bounding box.
    pub max_x: f64,

    /// Minimum Y coordinate of the bounding box.
    pub min_y: f64,

    /// Maximum Y coordinate of the bounding box.
    pub max_y: f64,

    /// Optional minimum Z coordinate for 3D extraction.
    ///
    /// If `None`, no lower Z constraint is applied.
    pub min_z: Option<f64>,

    /// Optional maximum Z coordinate for 3D extraction.
    ///
    /// If `None`, no upper Z constraint is applied.
    pub max_z: Option<f64>,
}

impl BoundingBox {
    /// Create a new 2D bounding box
    pub fn new_2d(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z: None,
            max_z: None,
        }
    }

    /// Create a new 3D bounding box
    pub fn new_3d(min_x: f64, max_x: f64, min_y: f64, max_y: f64, min_z: f64, max_z: f64) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z: Some(min_z),
            max_z: Some(max_z),
        }
    }

    /// Check if this bounding box has Z constraints
    pub fn has_z_constraint(&self) -> bool {
        self.min_z.is_some() || self.max_z.is_some()
    }
}

impl ExtractGeometry for BoundingBox {
    fn bounding_box(&self) -> Rect<f64> {
        Rect::new(
            coord! { x: self.min_x, y: self.min_y },
            coord! { x: self.max_x, y: self.max_y },
        )
    }

    fn contains_xy(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    fn contains_xyz(&self, x: f64, y: f64, z: f64) -> bool {
        // Check X and Y first
        if !self.contains_xy(x, y) {
            return false;
        }

        // Check Z constraints if they exist
        if let Some(min_z) = self.min_z
            && z < min_z
        {
            return false;
        }
        if let Some(max_z) = self.max_z
            && z > max_z
        {
            return false;
        }

        true
    }

    fn intersects_rect(&self, rect: &Rect<f64>) -> bool {
        // Two rectangles intersect if they overlap in both X and Y
        let x_overlap = self.min_x <= rect.max().x && self.max_x >= rect.min().x;
        let y_overlap = self.min_y <= rect.max().y && self.max_y >= rect.min().y;
        x_overlap && y_overlap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_2d() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 20.0);
        assert_eq!(bbox.min_x, 0.0);
        assert_eq!(bbox.max_x, 10.0);
        assert_eq!(bbox.min_y, 0.0);
        assert_eq!(bbox.max_y, 20.0);
        assert!(bbox.min_z.is_none());
        assert!(bbox.max_z.is_none());
        assert!(!bbox.has_z_constraint());
    }

    #[test]
    fn test_new_3d() {
        let bbox = BoundingBox::new_3d(0.0, 10.0, 0.0, 20.0, 0.0, 30.0);
        assert_eq!(bbox.min_z, Some(0.0));
        assert_eq!(bbox.max_z, Some(30.0));
        assert!(bbox.has_z_constraint());
    }

    #[test]
    fn test_contains_xy_inside() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 20.0);
        assert!(bbox.contains_xy(5.0, 10.0));
    }

    #[test]
    fn test_contains_xy_boundary() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 20.0);
        assert!(bbox.contains_xy(0.0, 0.0));
        assert!(bbox.contains_xy(10.0, 20.0));
        assert!(bbox.contains_xy(0.0, 20.0));
        assert!(bbox.contains_xy(10.0, 0.0));
    }

    #[test]
    fn test_contains_xy_outside() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 20.0);
        assert!(!bbox.contains_xy(-1.0, 10.0));
        assert!(!bbox.contains_xy(11.0, 10.0));
        assert!(!bbox.contains_xy(5.0, -1.0));
        assert!(!bbox.contains_xy(5.0, 21.0));
    }

    #[test]
    fn test_contains_xyz_2d_ignores_z() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 20.0);
        assert!(bbox.contains_xyz(5.0, 10.0, 1000.0));
        assert!(bbox.contains_xyz(5.0, 10.0, -1000.0));
    }

    #[test]
    fn test_contains_xyz_3d_with_z() {
        let bbox = BoundingBox::new_3d(0.0, 10.0, 0.0, 20.0, 0.0, 30.0);
        assert!(bbox.contains_xyz(5.0, 10.0, 15.0));
        assert!(bbox.contains_xyz(5.0, 10.0, 0.0));
        assert!(bbox.contains_xyz(5.0, 10.0, 30.0));
        assert!(!bbox.contains_xyz(5.0, 10.0, -1.0));
        assert!(!bbox.contains_xyz(5.0, 10.0, 31.0));
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new_2d(10.0, 20.0, 30.0, 40.0);
        let rect = bbox.bounding_box();
        assert_eq!(rect.min().x, 10.0);
        assert_eq!(rect.min().y, 30.0);
        assert_eq!(rect.max().x, 20.0);
        assert_eq!(rect.max().y, 40.0);
    }

    #[test]
    fn test_intersects_rect_overlapping() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 10.0);
        let rect = Rect::new(coord! { x: 5.0, y: 5.0 }, coord! { x: 15.0, y: 15.0 });
        assert!(bbox.intersects_rect(&rect));
    }

    #[test]
    fn test_intersects_rect_contained() {
        let bbox = BoundingBox::new_2d(0.0, 20.0, 0.0, 20.0);
        let rect = Rect::new(coord! { x: 5.0, y: 5.0 }, coord! { x: 15.0, y: 15.0 });
        assert!(bbox.intersects_rect(&rect));
    }

    #[test]
    fn test_intersects_rect_disjoint() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 10.0);
        let rect = Rect::new(coord! { x: 20.0, y: 20.0 }, coord! { x: 30.0, y: 30.0 });
        assert!(!bbox.intersects_rect(&rect));
    }

    #[test]
    fn test_intersects_rect_touching() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 10.0);
        let rect = Rect::new(coord! { x: 10.0, y: 0.0 }, coord! { x: 20.0, y: 10.0 });
        assert!(bbox.intersects_rect(&rect));
    }
}
