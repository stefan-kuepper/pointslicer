use crate::geometry::ExtractGeometry;
use las::Point;

/// Filter points based on an extraction geometry.
///
/// This function takes a slice of points and returns only those points
/// that are inside the specified extraction geometry.
///
/// # Parameters
///
/// - `points`: Slice of points to filter
/// - `geometry`: Extraction geometry to test points against
///
/// # Returns
///
/// A vector containing only the points that are inside the geometry.
///
/// # Examples
///
/// ```
/// use pointslicer_core::pointcloud::filter_points;
/// use pointslicer_core::geometry::BoundingBox;
/// use las::Point;
///
/// let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 10.0);
///
/// let mut point1 = Point::default();
/// point1.x = 5.0;
/// point1.y = 5.0;
/// point1.z = 0.0;
///
/// let mut point2 = Point::default();
/// point2.x = 15.0;
/// point2.y = 15.0;
/// point2.z = 0.0;
///
/// let points = vec![point1, point2];
/// let filtered = filter_points(&points, &bbox);
///
/// assert_eq!(filtered.len(), 1);
/// ```
pub fn filter_points<G: ExtractGeometry>(points: &[Point], geometry: &G) -> Vec<Point> {
    points
        .iter()
        .filter(|point| geometry.contains_xyz(point.x, point.y, point.z))
        .cloned()
        .collect()
}

/// Filter points from an iterator based on an extraction geometry.
///
/// This function takes an iterator of points and returns only those points
/// that are inside the specified extraction geometry. This is more memory
/// efficient than `filter_points()` for large datasets.
///
/// # Parameters
///
/// - `points`: Iterator of points to filter
/// - `geometry`: Extraction geometry to test points against
///
/// # Returns
///
/// A vector containing only the points that are inside the geometry.
///
/// # Examples
///
/// ```
/// use pointslicer_core::pointcloud::filter_points_iter;
/// use pointslicer_core::geometry::BoundingBox;
/// use las::Point;
///
/// let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 10.0);
///
/// let points = vec![
///     Point { x: 5.0, y: 5.0, z: 0.0, ..Default::default() },
///     Point { x: 15.0, y: 15.0, z: 0.0, ..Default::default() },
/// ];
///
/// let filtered = filter_points_iter(points.into_iter(), &bbox);
/// assert_eq!(filtered.len(), 1);
/// ```
pub fn filter_points_iter<G: ExtractGeometry, I>(points: I, geometry: &G) -> Vec<Point>
where
    I: Iterator<Item = Point>,
{
    points
        .filter(|point| geometry.contains_xyz(point.x, point.y, point.z))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::BoundingBox;

    fn create_test_point(x: f64, y: f64, z: f64) -> Point {
        Point {
            x,
            y,
            z,
            ..Default::default()
        }
    }

    #[test]
    fn test_filter_points_bbox() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 10.0);

        let points = vec![
            create_test_point(5.0, 5.0, 0.0),   // inside
            create_test_point(15.0, 5.0, 0.0),  // outside
            create_test_point(0.0, 0.0, 0.0),   // on boundary (included)
            create_test_point(10.0, 10.0, 0.0), // on boundary (included)
        ];

        let filtered = filter_points(&points, &bbox);
        assert_eq!(filtered.len(), 3); // 2 inside + 2 on boundary
    }

    #[test]
    fn test_filter_points_bbox_3d() {
        let bbox = BoundingBox::new_3d(0.0, 10.0, 0.0, 10.0, 0.0, 5.0);

        let points = vec![
            create_test_point(5.0, 5.0, 2.0),  // inside
            create_test_point(5.0, 5.0, 10.0), // outside (z too high)
            create_test_point(5.0, 5.0, -1.0), // outside (z too low)
        ];

        let filtered = filter_points(&points, &bbox);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_points_iter() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 10.0);

        let points = vec![
            create_test_point(5.0, 5.0, 0.0),
            create_test_point(15.0, 5.0, 0.0),
        ];

        let filtered = filter_points_iter(points.into_iter(), &bbox);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_points_empty() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 10.0);
        let points: Vec<Point> = vec![];

        let filtered = filter_points(&points, &bbox);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_points_all_outside() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 10.0);
        let points = vec![
            create_test_point(15.0, 15.0, 0.0),
            create_test_point(20.0, 20.0, 0.0),
            create_test_point(-5.0, -5.0, 0.0),
        ];

        let filtered = filter_points(&points, &bbox);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_points_boundary_cases() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 10.0);
        let points = vec![
            create_test_point(0.0, 0.0, 0.0),   // exact corner
            create_test_point(10.0, 10.0, 0.0), // exact corner
            create_test_point(5.0, 0.0, 0.0),   // exact edge
            create_test_point(0.0, 5.0, 0.0),   // exact edge
        ];

        let filtered = filter_points(&points, &bbox);
        // All boundary points should be included
        assert_eq!(filtered.len(), 4);
    }

    #[test]
    fn test_filter_points_all_inside() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 10.0);
        let points = vec![
            create_test_point(1.0, 1.0, 0.0),
            create_test_point(5.0, 5.0, 0.0),
            create_test_point(9.0, 9.0, 0.0),
        ];

        let filtered = filter_points(&points, &bbox);
        assert_eq!(filtered.len(), 3);
    }
}
