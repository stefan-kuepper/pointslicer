use crate::geometry::ExtractGeometry;
use las::Point;

/// Filter points based on an extraction geometry
pub fn filter_points<G: ExtractGeometry>(
    points: &[Point],
    geometry: &G,
) -> Vec<Point> {
    points
        .iter()
        .filter(|point| geometry.contains_xyz(point.x, point.y, point.z))
        .cloned()
        .collect()
}

/// Filter points from an iterator based on an extraction geometry
pub fn filter_points_iter<G: ExtractGeometry, I>(
    points: I,
    geometry: &G,
) -> Vec<Point>
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
        let mut point = Point::default();
        point.x = x;
        point.y = y;
        point.z = z;
        point
    }

    #[test]
    fn test_filter_points_bbox() {
        let bbox = BoundingBox::new_2d(0.0, 10.0, 0.0, 10.0);

        let points = vec![
            create_test_point(5.0, 5.0, 0.0),  // inside
            create_test_point(15.0, 5.0, 0.0), // outside
            create_test_point(0.0, 0.0, 0.0),  // on boundary (included)
            create_test_point(10.0, 10.0, 0.0), // on boundary (included)
        ];

        let filtered = filter_points(&points, &bbox);
        assert_eq!(filtered.len(), 3); // 2 inside + 2 on boundary
    }

    #[test]
    fn test_filter_points_bbox_3d() {
        let bbox = BoundingBox::new_3d(0.0, 10.0, 0.0, 10.0, 0.0, 5.0);

        let points = vec![
            create_test_point(5.0, 5.0, 2.0),   // inside
            create_test_point(5.0, 5.0, 10.0),  // outside (z too high)
            create_test_point(5.0, 5.0, -1.0),  // outside (z too low)
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
}
