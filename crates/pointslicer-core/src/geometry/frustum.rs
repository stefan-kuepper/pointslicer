use super::traits::ExtractGeometry;
use geo::{coord, Rect};

/// A frustum (truncated pyramid) defined by origin, angular bounds, and distance bounds.
///
/// The frustum is defined in a geographic coordinate system where:
/// - phi (azimuth) is measured clockwise from north (0-360 degrees)
/// - theta (elevation) is measured above/below horizontal (-90 to +90 degrees)
///
/// The two (phi, theta) pairs define opposite corners of the angular extent,
/// creating a rectangular field-of-view in spherical coordinates.
///
/// # Examples
///
/// ```
/// use pointslicer_core::geometry::{Frustum, ExtractGeometry};
///
/// // Create a frustum looking northeast (0-90 degrees), horizontal to 45 degrees up
/// let frustum = Frustum::new(0.0, 0.0, 0.0, 0.0, 0.0, 90.0, 45.0, 10.0, 100.0);
///
/// // Check if points are inside
/// assert!(frustum.contains_xyz(50.0, 50.0, 0.0)); // Northeast, horizontal
/// assert!(!frustum.contains_xyz(-50.0, -50.0, 0.0)); // Southwest (outside)
/// ```
#[derive(Debug, Clone)]
pub struct Frustum {
    /// X coordinate of the frustum origin (apex).
    pub origin_x: f64,

    /// Y coordinate of the frustum origin (apex).
    pub origin_y: f64,

    /// Z coordinate of the frustum origin (apex).
    pub origin_z: f64,

    /// Minimum azimuth angle in radians (clockwise from north).
    phi_min: f64,

    /// Maximum azimuth angle in radians (clockwise from north).
    phi_max: f64,

    /// Minimum elevation angle in radians (above horizontal).
    theta_min: f64,

    /// Maximum elevation angle in radians (above horizontal).
    theta_max: f64,

    /// Minimum distance from origin (near plane).
    pub min_distance: f64,

    /// Maximum distance from origin (far plane).
    pub max_distance: f64,

    /// Whether the frustum crosses the north axis (phi wraps around 0/360).
    crosses_north: bool,
}

impl Frustum {
    /// Create a new frustum from two corner angles (in degrees) and distance bounds.
    ///
    /// # Parameters
    /// - `origin_x`, `origin_y`, `origin_z`: The apex/origin point
    /// - `phi1`, `theta1`: First corner (azimuth in degrees 0-360, elevation in degrees -90 to +90)
    /// - `phi2`, `theta2`: Opposite corner
    /// - `min_distance`: Near plane distance from origin
    /// - `max_distance`: Far plane distance from origin
    ///
    /// # Panics
    /// Panics if min_distance > max_distance or if min_distance is negative.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        origin_x: f64,
        origin_y: f64,
        origin_z: f64,
        phi1: f64,
        theta1: f64,
        phi2: f64,
        theta2: f64,
        min_distance: f64,
        max_distance: f64,
    ) -> Self {
        assert!(
            min_distance >= 0.0,
            "min_distance must be non-negative, got {}",
            min_distance
        );
        assert!(
            max_distance >= min_distance,
            "max_distance ({}) must be >= min_distance ({})",
            max_distance,
            min_distance
        );

        // Convert degrees to radians
        let phi1_rad = phi1.to_radians();
        let phi2_rad = phi2.to_radians();
        let theta1_rad = theta1.to_radians();
        let theta2_rad = theta2.to_radians();

        // Check if this is a full 360-degree azimuth (or close to it)
        let phi_diff = (phi2 - phi1).abs();
        let is_full_circle = phi_diff >= 359.9 || phi_diff <= 0.1;

        // Normalize phi to [0, 2*PI)
        let phi1_norm = Self::normalize_phi(phi1_rad);
        let phi2_norm = Self::normalize_phi(phi2_rad);

        // Determine min/max for theta (straightforward)
        let (theta_min, theta_max) = if theta1_rad <= theta2_rad {
            (theta1_rad, theta2_rad)
        } else {
            (theta2_rad, theta1_rad)
        };

        // Determine min/max for phi and whether we cross north
        let (phi_min, phi_max, crosses_north) = if is_full_circle {
            // Full 360-degree coverage
            (0.0, std::f64::consts::TAU, false)
        } else {
            Self::compute_phi_bounds(phi1_norm, phi2_norm)
        };

        Self {
            origin_x,
            origin_y,
            origin_z,
            phi_min,
            phi_max,
            theta_min,
            theta_max,
            min_distance,
            max_distance,
            crosses_north,
        }
    }

    /// Normalize phi angle to [0, 2*PI) range.
    fn normalize_phi(phi: f64) -> f64 {
        let two_pi = std::f64::consts::TAU;
        let mut normalized = phi % two_pi;
        if normalized < 0.0 {
            normalized += two_pi;
        }
        normalized
    }

    /// Compute phi bounds and determine if the frustum crosses north (0/360 boundary).
    ///
    /// This chooses the shorter arc between the two phi values.
    fn compute_phi_bounds(phi1: f64, phi2: f64) -> (f64, f64, bool) {
        let two_pi = std::f64::consts::TAU;

        // Calculate the two possible arc lengths
        let direct_diff = (phi2 - phi1).abs();
        let wrap_diff = two_pi - direct_diff;

        if direct_diff <= wrap_diff {
            // Use direct arc (doesn't cross north)
            if phi1 <= phi2 {
                (phi1, phi2, false)
            } else {
                (phi2, phi1, false)
            }
        } else {
            // Use wrapped arc (crosses north)
            // The frustum includes angles >= phi_min OR <= phi_max
            if phi1 <= phi2 {
                // phi1 is smaller, phi2 is larger
                // Wrapped arc goes from phi2 around through 0 to phi1
                (phi2, phi1, true)
            } else {
                // phi2 is smaller, phi1 is larger
                // Wrapped arc goes from phi1 around through 0 to phi2
                (phi1, phi2, true)
            }
        }
    }

    /// Convert Cartesian offset to spherical coordinates (distance, phi, theta).
    ///
    /// Returns (distance, phi_radians, theta_radians) where:
    /// - phi is azimuth clockwise from north [0, 2*PI)
    /// - theta is elevation above horizontal [-PI/2, PI/2]
    fn cartesian_to_spherical(dx: f64, dy: f64, dz: f64) -> (f64, f64, f64) {
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();

        if distance < 1e-10 {
            // Point is at origin - return arbitrary angles
            return (0.0, 0.0, 0.0);
        }

        // Horizontal distance
        let h_dist = (dx * dx + dy * dy).sqrt();

        // Elevation angle (theta)
        let theta = (dz / distance).asin();

        // Azimuth angle (phi) - clockwise from north
        // North = +Y, East = +X
        // atan2(dx, dy) gives angle clockwise from +Y
        let phi = if h_dist < 1e-10 {
            // Point is directly above/below origin
            0.0
        } else {
            let phi_raw = dx.atan2(dy); // atan2(x, y) for clockwise from +Y
            Self::normalize_phi(phi_raw)
        };

        (distance, phi, theta)
    }

    /// Check if phi angle is within bounds, accounting for north crossing.
    fn phi_in_bounds(&self, phi: f64) -> bool {
        // Full circle case (phi_max == TAU)
        if self.phi_max >= std::f64::consts::TAU - 1e-9 {
            return true;
        }

        if self.crosses_north {
            // Frustum wraps around north: includes phi >= phi_min OR phi <= phi_max
            phi >= self.phi_min || phi <= self.phi_max
        } else {
            // Normal case: includes phi_min <= phi <= phi_max
            phi >= self.phi_min && phi <= self.phi_max
        }
    }

    /// Convert spherical coordinates back to Cartesian (absolute coordinates).
    fn spherical_to_cartesian(&self, dist: f64, phi: f64, theta: f64) -> (f64, f64, f64) {
        // phi is clockwise from north, theta is elevation
        let h_dist = dist * theta.cos(); // horizontal distance
        let z = dist * theta.sin() + self.origin_z;

        // phi clockwise from north: x = h_dist * sin(phi), y = h_dist * cos(phi)
        let x = h_dist * phi.sin() + self.origin_x;
        let y = h_dist * phi.cos() + self.origin_y;

        (x, y, z)
    }

    /// Calculate the 2D bounding box of the frustum's XY projection.
    fn compute_bounding_box(&self) -> Rect<f64> {
        let mut min_x = self.origin_x;
        let mut max_x = self.origin_x;
        let mut min_y = self.origin_y;
        let mut max_y = self.origin_y;

        // Sample the corners of the frustum at both near and far planes
        let phi_values: Vec<f64> = if self.crosses_north {
            vec![self.phi_min, self.phi_max, 0.0] // Include north when crossing
        } else {
            vec![self.phi_min, self.phi_max]
        };
        let theta_values = vec![self.theta_min, self.theta_max];
        let dist_values = vec![self.min_distance, self.max_distance];

        for &phi in &phi_values {
            for &theta in &theta_values {
                for &dist in &dist_values {
                    let (x, y, _z) = self.spherical_to_cartesian(dist, phi, theta);
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
            }
        }

        // Also sample cardinal directions if they fall within the frustum
        // This catches cases where the extremes are along axes
        let cardinal_phis = [
            0.0,                              // North
            std::f64::consts::FRAC_PI_2,      // East
            std::f64::consts::PI,             // South
            3.0 * std::f64::consts::FRAC_PI_2, // West
        ];

        for &phi in &cardinal_phis {
            if self.phi_in_bounds(phi) {
                for &theta in &theta_values {
                    for &dist in &dist_values {
                        let (x, y, _z) = self.spherical_to_cartesian(dist, phi, theta);
                        min_x = min_x.min(x);
                        max_x = max_x.max(x);
                        min_y = min_y.min(y);
                        max_y = max_y.max(y);
                    }
                }
            }
        }

        Rect::new(coord! { x: min_x, y: min_y }, coord! { x: max_x, y: max_y })
    }
}

impl ExtractGeometry for Frustum {
    fn bounding_box(&self) -> Rect<f64> {
        self.compute_bounding_box()
    }

    fn contains_xy(&self, x: f64, y: f64) -> bool {
        // For 2D containment, check if the XY projection could be inside.
        // We check if there exists ANY z value that would put (x, y, z) inside.

        let dx = x - self.origin_x;
        let dy = y - self.origin_y;
        let h_dist = (dx * dx + dy * dy).sqrt();

        if h_dist < 1e-10 {
            // Point is directly above/below origin
            // Check if vertical ray intersects frustum (any elevation in range)
            return self.theta_max >= -std::f64::consts::FRAC_PI_2
                && self.theta_min <= std::f64::consts::FRAC_PI_2;
        }

        // Calculate phi for this XY position
        let phi = Self::normalize_phi(dx.atan2(dy));

        if !self.phi_in_bounds(phi) {
            return false;
        }

        // Check if any valid distance/theta combination could place a point here
        // The horizontal distance at elevation theta and 3D distance d is: h = d * cos(theta)
        // So d = h / cos(theta)
        // For a point to be in the frustum at this h_dist:
        // min_distance <= h_dist / cos(theta) <= max_distance
        // Which means: h_dist / max_distance <= cos(theta) <= h_dist / min_distance

        // Maximum possible horizontal distance is at theta closest to 0
        let theta_for_max_h = if self.theta_min <= 0.0 && self.theta_max >= 0.0 {
            0.0 // Horizontal is in range
        } else if self.theta_min.abs() < self.theta_max.abs() {
            self.theta_min
        } else {
            self.theta_max
        };

        let max_h_dist = self.max_distance * theta_for_max_h.cos();

        // Minimum horizontal distance depends on elevation limits
        let min_h_dist = if self.min_distance == 0.0 {
            0.0
        } else {
            // Minimum h occurs at extreme elevations
            let h_at_theta_min = self.min_distance * self.theta_min.cos();
            let h_at_theta_max = self.min_distance * self.theta_max.cos();
            h_at_theta_min.min(h_at_theta_max)
        };

        h_dist >= min_h_dist && h_dist <= max_h_dist
    }

    fn contains_xyz(&self, x: f64, y: f64, z: f64) -> bool {
        // Calculate offset from origin
        let dx = x - self.origin_x;
        let dy = y - self.origin_y;
        let dz = z - self.origin_z;

        // Convert to spherical coordinates
        let (distance, phi, theta) = Self::cartesian_to_spherical(dx, dy, dz);

        // Check distance bounds
        if distance < self.min_distance || distance > self.max_distance {
            return false;
        }

        // Check theta (elevation) bounds
        if theta < self.theta_min || theta > self.theta_max {
            return false;
        }

        // Check phi (azimuth) bounds
        self.phi_in_bounds(phi)
    }

    fn intersects_rect(&self, rect: &Rect<f64>) -> bool {
        // Conservative approximation using bounding box intersection
        let bbox = self.compute_bounding_box();
        bbox.min().x <= rect.max().x
            && bbox.max().x >= rect.min().x
            && bbox.min().y <= rect.max().y
            && bbox.max().y >= rect.min().y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

    const EPSILON: f64 = 1e-9;

    #[test]
    fn test_new_basic() {
        let f = Frustum::new(0.0, 0.0, 0.0, 0.0, 0.0, 90.0, 45.0, 10.0, 100.0);
        assert_eq!(f.origin_x, 0.0);
        assert_eq!(f.min_distance, 10.0);
        assert_eq!(f.max_distance, 100.0);
    }

    #[test]
    #[should_panic(expected = "min_distance must be non-negative")]
    fn test_new_negative_min_distance() {
        Frustum::new(0.0, 0.0, 0.0, 0.0, 0.0, 90.0, 45.0, -10.0, 100.0);
    }

    #[test]
    #[should_panic(expected = "max_distance")]
    fn test_new_invalid_distances() {
        Frustum::new(0.0, 0.0, 0.0, 0.0, 0.0, 90.0, 45.0, 100.0, 10.0);
    }

    #[test]
    fn test_contains_xyz_north_direction() {
        // Frustum looking north (phi -20 to 20 deg = 340 to 20), horizontal
        let f = Frustum::new(0.0, 0.0, 0.0, 340.0, -10.0, 20.0, 10.0, 10.0, 100.0);

        // Point directly north at distance 50
        assert!(f.contains_xyz(0.0, 50.0, 0.0));

        // Point directly south - should not be contained
        assert!(!f.contains_xyz(0.0, -50.0, 0.0));
    }

    #[test]
    fn test_contains_xyz_east_direction() {
        // Frustum looking east (phi 80-100 deg), horizontal
        let f = Frustum::new(0.0, 0.0, 0.0, 80.0, -10.0, 100.0, 10.0, 10.0, 100.0);

        // Point directly east at distance 50
        assert!(f.contains_xyz(50.0, 0.0, 0.0));

        // Point directly west - should not be contained
        assert!(!f.contains_xyz(-50.0, 0.0, 0.0));
    }

    #[test]
    fn test_contains_xyz_south_direction() {
        // Frustum looking south (phi 170-190 deg), horizontal
        let f = Frustum::new(0.0, 0.0, 0.0, 170.0, -10.0, 190.0, 10.0, 10.0, 100.0);

        // Point directly south at distance 50
        assert!(f.contains_xyz(0.0, -50.0, 0.0));

        // Point directly north - should not be contained
        assert!(!f.contains_xyz(0.0, 50.0, 0.0));
    }

    #[test]
    fn test_contains_xyz_west_direction() {
        // Frustum looking west (phi 260-280 deg), horizontal
        let f = Frustum::new(0.0, 0.0, 0.0, 260.0, -10.0, 280.0, 10.0, 10.0, 100.0);

        // Point directly west at distance 50
        assert!(f.contains_xyz(-50.0, 0.0, 0.0));

        // Point directly east - should not be contained
        assert!(!f.contains_xyz(50.0, 0.0, 0.0));
    }

    #[test]
    fn test_contains_xyz_with_elevation_up() {
        // Frustum looking up (theta 30-60 deg), north
        let f = Frustum::new(0.0, 0.0, 0.0, 350.0, 30.0, 10.0, 60.0, 10.0, 100.0);

        // Point above at 45 degrees north
        let dist = 50.0;
        let theta = FRAC_PI_4; // 45 degrees
        let y = dist * theta.cos();
        let z = dist * theta.sin();
        assert!(f.contains_xyz(0.0, y, z));

        // Point at horizontal level - should not be contained
        assert!(!f.contains_xyz(0.0, 50.0, 0.0));
    }

    #[test]
    fn test_contains_xyz_with_elevation_down() {
        // Frustum looking down (theta -60 to -30 deg), north
        let f = Frustum::new(0.0, 0.0, 0.0, 350.0, -60.0, 10.0, -30.0, 10.0, 100.0);

        // Point below at -45 degrees north
        let dist = 50.0;
        let theta = -FRAC_PI_4; // -45 degrees
        let y = dist * theta.cos();
        let z = dist * theta.sin();
        assert!(f.contains_xyz(0.0, y, z));

        // Point at horizontal level - should not be contained
        assert!(!f.contains_xyz(0.0, 50.0, 0.0));
    }

    #[test]
    fn test_distance_bounds_too_close() {
        let f = Frustum::new(0.0, 0.0, 0.0, 0.0, -45.0, 90.0, 45.0, 20.0, 80.0);

        // Point at distance 10 (too close)
        assert!(!f.contains_xyz(0.0, 10.0, 0.0));
    }

    #[test]
    fn test_distance_bounds_in_range() {
        let f = Frustum::new(0.0, 0.0, 0.0, 0.0, -45.0, 90.0, 45.0, 20.0, 80.0);

        // Point at distance 50 (in range)
        assert!(f.contains_xyz(0.0, 50.0, 0.0));
    }

    #[test]
    fn test_distance_bounds_too_far() {
        let f = Frustum::new(0.0, 0.0, 0.0, 0.0, -45.0, 90.0, 45.0, 20.0, 80.0);

        // Point at distance 100 (too far)
        assert!(!f.contains_xyz(0.0, 100.0, 0.0));
    }

    #[test]
    fn test_distance_bounds_at_boundary() {
        let f = Frustum::new(0.0, 0.0, 0.0, 0.0, -45.0, 90.0, 45.0, 20.0, 80.0);

        // Points exactly at boundaries should be included
        assert!(f.contains_xyz(0.0, 20.0, 0.0)); // At min distance
        assert!(f.contains_xyz(0.0, 80.0, 0.0)); // At max distance
    }

    #[test]
    fn test_phi_crossing_north() {
        // Frustum spanning from 350 to 10 degrees (crossing north)
        let f = Frustum::new(0.0, 0.0, 0.0, 350.0, -10.0, 10.0, 10.0, 10.0, 100.0);

        // Point at phi=0 (due north) should be inside
        assert!(f.contains_xyz(0.0, 50.0, 0.0));

        // Point at phi=5 degrees should be inside
        let phi = 5.0_f64.to_radians();
        let dist = 50.0;
        assert!(f.contains_xyz(dist * phi.sin(), dist * phi.cos(), 0.0));

        // Point at phi=355 degrees should be inside
        let phi = 355.0_f64.to_radians();
        assert!(f.contains_xyz(dist * phi.sin(), dist * phi.cos(), 0.0));

        // Point at phi=180 (due south) should be outside
        assert!(!f.contains_xyz(0.0, -50.0, 0.0));
    }

    #[test]
    fn test_bounding_box_contains_origin() {
        let f = Frustum::new(100.0, 200.0, 50.0, 0.0, 0.0, 90.0, 45.0, 10.0, 100.0);
        let bbox = f.bounding_box();

        // Bounding box should contain origin
        assert!(bbox.min().x <= 100.0 && bbox.max().x >= 100.0);
        assert!(bbox.min().y <= 200.0 && bbox.max().y >= 200.0);
    }

    #[test]
    fn test_bounding_box_reasonable_size() {
        let f = Frustum::new(0.0, 0.0, 0.0, 0.0, -10.0, 90.0, 10.0, 10.0, 100.0);
        let bbox = f.bounding_box();

        // Bounding box should extend to at least max_distance in relevant directions
        assert!(bbox.max().x >= 90.0); // East direction
        assert!(bbox.max().y >= 90.0); // North direction
    }

    #[test]
    fn test_intersects_rect_overlapping() {
        let f = Frustum::new(0.0, 0.0, 0.0, 0.0, -10.0, 90.0, 10.0, 10.0, 100.0);

        // Rectangle in the frustum's direction
        let rect = Rect::new(coord! { x: 10.0, y: 10.0 }, coord! { x: 50.0, y: 50.0 });
        assert!(f.intersects_rect(&rect));
    }

    #[test]
    fn test_intersects_rect_disjoint() {
        let f = Frustum::new(0.0, 0.0, 0.0, 0.0, -10.0, 45.0, 10.0, 10.0, 50.0);

        // Rectangle far behind the frustum (southwest)
        let rect = Rect::new(
            coord! { x: -200.0, y: -200.0 },
            coord! { x: -150.0, y: -150.0 },
        );
        assert!(!f.intersects_rect(&rect));
    }

    #[test]
    fn test_contains_xy_in_frustum_direction() {
        let f = Frustum::new(0.0, 0.0, 0.0, 0.0, -45.0, 90.0, 45.0, 10.0, 100.0);

        // Point in northeast (in phi range)
        assert!(f.contains_xy(30.0, 30.0));

        // Point in southwest (outside phi range)
        assert!(!f.contains_xy(-50.0, -50.0));
    }

    #[test]
    fn test_zero_min_distance() {
        let f = Frustum::new(0.0, 0.0, 0.0, 0.0, -45.0, 90.0, 45.0, 0.0, 100.0);

        // Point very close to origin should be inside
        assert!(f.contains_xyz(0.0, 0.1, 0.0));

        // Point at origin - distance is 0, which equals min_distance
        assert!(f.contains_xyz(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_full_hemisphere_up() {
        // Frustum covering full 360 degrees, looking up
        let f = Frustum::new(0.0, 0.0, 0.0, 0.0, 0.0, 360.0, 90.0, 10.0, 100.0);

        // Point directly up
        assert!(f.contains_xyz(0.0, 0.0, 50.0));

        // Point horizontal north
        assert!(f.contains_xyz(0.0, 50.0, 0.0));

        // Point horizontal east
        assert!(f.contains_xyz(50.0, 0.0, 0.0));

        // Point below horizontal should not be contained
        assert!(!f.contains_xyz(0.0, 50.0, -10.0));
    }

    #[test]
    fn test_spherical_cartesian_conversion() {
        // Test that cartesian_to_spherical works correctly
        // Point due north at distance 100
        let (dist, phi, theta) = Frustum::cartesian_to_spherical(0.0, 100.0, 0.0);
        assert!((dist - 100.0).abs() < EPSILON);
        assert!(phi.abs() < EPSILON); // phi should be 0 (north)
        assert!(theta.abs() < EPSILON); // theta should be 0 (horizontal)

        // Point due east at distance 100
        let (dist, phi, theta) = Frustum::cartesian_to_spherical(100.0, 0.0, 0.0);
        assert!((dist - 100.0).abs() < EPSILON);
        assert!((phi - FRAC_PI_2).abs() < EPSILON); // phi should be 90 degrees (east)
        assert!(theta.abs() < EPSILON);

        // Point directly up at distance 100
        let (dist, phi, theta) = Frustum::cartesian_to_spherical(0.0, 0.0, 100.0);
        assert!((dist - 100.0).abs() < EPSILON);
        assert!((theta - FRAC_PI_2).abs() < EPSILON); // theta should be 90 degrees
    }

    #[test]
    fn test_offset_origin() {
        // Test frustum with non-zero origin
        let f = Frustum::new(100.0, 200.0, 50.0, 350.0, -10.0, 10.0, 10.0, 10.0, 100.0);

        // Point north of origin at distance 50
        assert!(f.contains_xyz(100.0, 250.0, 50.0));

        // Point south of origin should not be contained
        assert!(!f.contains_xyz(100.0, 150.0, 50.0));
    }
}
