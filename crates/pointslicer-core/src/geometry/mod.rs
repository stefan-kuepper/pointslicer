//! Geometry system for spatial point extraction.
//!
//! This module provides a trait-based system for defining extraction geometries
//! that can be used to filter points from LAS/LAZ files. The core abstraction
//! is the [`ExtractGeometry`] trait, which defines the interface for any
//! geometry that can extract points.
//!
//! ## Available Geometries
//!
//! - [`BoundingBox`]: A 2D or 3D axis-aligned bounding box
//! - [`VerticalCylinder`]: A vertical cylinder defined by center coordinates and radius
//!
//! ## Adding Custom Geometries
//!
//! To add a new geometry type, implement the [`ExtractGeometry`] trait:
//!
//! ```no_run
//! use pointslicer_core::geometry::ExtractGeometry;
//! use geo::{Rect, coord};
//!
//! struct MyGeometry {
//!     center_x: f64,
//!     center_y: f64,
//!     radius: f64,
//! }
//!
//! impl ExtractGeometry for MyGeometry {
//!     fn bounding_box(&self) -> Rect<f64> {
//!         // Return the bounding rectangle of your geometry
//!         Rect::new(
//!             coord! { x: self.center_x - self.radius, y: self.center_y - self.radius },
//!             coord! { x: self.center_x + self.radius, y: self.center_y + self.radius },
//!         )
//!     }
//!     
//!     fn contains_xy(&self, x: f64, y: f64) -> bool {
//!         // Implement point-in-geometry test
//!         let dx = x - self.center_x;
//!         let dy = y - self.center_y;
//!         dx * dx + dy * dy <= self.radius * self.radius
//!     }
//!     
//!     fn contains_xyz(&self, x: f64, y: f64, _z: f64) -> bool {
//!         self.contains_xy(x, y)
//!     }
//!     
//!     fn intersects_rect(&self, rect: &Rect<f64>) -> bool {
//!         // Implement rectangle intersection test
//!         true // Simplified example
//!     }
//! }
//! ```
//!
//! ## Usage Example
//!
//! ```no_run
//! use pointslicer_core::geometry::{BoundingBox, VerticalCylinder, ExtractGeometry};
//!
//! // Create a 2D bounding box
//! let bbox = BoundingBox::new_2d(10000.0, 20000.0, 30000.0, 40000.0);
//!
//! // Create a vertical cylinder from diameter
//! let cylinder = VerticalCylinder::from_diameter(12345.0, 67890.0, 12.0);
//!
//! // Check if points are inside the geometries
//! assert!(bbox.contains_xy(15000.0, 35000.0));
//! assert!(cylinder.contains_xy(12345.0, 67890.0));
//! ```

pub mod bbox;
pub mod cylinder;
pub mod traits;

/// A 2D or 3D axis-aligned bounding box for point extraction.
///
/// This struct represents a rectangular region in 2D or 3D space that can be
/// used to filter points. It supports both 2D (X,Y only) and 3D (X,Y,Z)
/// extraction.
///
/// # Fields
///
/// - `min_x`: Minimum X coordinate
/// - `max_x`: Maximum X coordinate  
/// - `min_y`: Minimum Y coordinate
/// - `max_y`: Maximum Y coordinate
/// - `min_z`: Optional minimum Z coordinate (for 3D extraction)
/// - `max_z`: Optional maximum Z coordinate (for 3D extraction)
pub use bbox::BoundingBox;

/// A vertical cylinder defined by center coordinates and radius.
///
/// This struct represents a vertical (Z-aligned) cylinder that can be used
/// to extract points within a circular region. The cylinder extends infinitely
/// in the Z direction.
///
/// # Fields
///
/// - `center_x`: X coordinate of the cylinder center
/// - `center_y`: Y coordinate of the cylinder center
/// - `radius`: Radius of the cylinder
pub use cylinder::VerticalCylinder;

/// The core trait for extraction geometries.
///
/// This trait defines the interface that all extraction geometries must
/// implement. It provides methods for spatial queries that are used by
/// the extraction pipeline to filter points.
pub use traits::ExtractGeometry;
