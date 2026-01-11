# pointslicer-core

Core library for extracting points from LAS/LAZ files using GeoPackage tile indices.

## Overview

`pointslicer-core` provides a trait-based architecture for spatial extraction of point cloud data. It reads tile indices created by `pdal tindex` and efficiently filters points using parallel processing with Rayon.

## Features

- **Trait-based geometry system**: Implement the `ExtractGeometry` trait to add custom extraction geometries
- **Parallel processing**: Uses Rayon for efficient tile processing
- **GeoPackage support**: Reads tile indices created by PDAL's `tindex` command
- **LAS/LAZ I/O**: Supports both compressed and uncompressed point cloud formats
- **Point format preservation**: Maintains original point format when writing output

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
pointslicer-core = { path = "../pointslicer-core" }
```

Basic example:

```rust
use pointslicer_core::geometry::{BoundingBox, ExtractGeometry};
use pointslicer_core::pipeline::ExtractionPipeline;

// Create a bounding box geometry
let bbox = BoundingBox::new_2d(10000.0, 20000.0, 30000.0, 40000.0);

// Create and run the extraction pipeline
let pipeline = ExtractionPipeline::new("tiles.gpkg", "output.laz", false);
let stats = pipeline.execute(&bbox).expect("Extraction failed");

println!("Extracted {} points from {} tiles", stats.points_written, stats.tiles_processed);
```

## Architecture

The library is organized into several modules:

- **error**: Error types and result aliases
- **geometry**: Trait-based geometry system for spatial queries
- **index**: GeoPackage tile index reading and spatial filtering
- **pipeline**: Orchestration of the extraction workflow
- **pointcloud**: LAS/LAZ file I/O and point filtering

## Adding Custom Geometries

To add a new extraction geometry, implement the `ExtractGeometry` trait:

```rust
use pointslicer_core::geometry::ExtractGeometry;
use geo::{Rect, coord};

struct MyGeometry {
    center_x: f64,
    center_y: f64,
    radius: f64,
}

impl ExtractGeometry for MyGeometry {
    fn bounding_box(&self) -> Rect<f64> {
        Rect::new(
            coord! { x: self.center_x - self.radius, y: self.center_y - self.radius },
            coord! { x: self.center_x + self.radius, y: self.center_y + self.radius },
        )
    }
    
    fn contains_xy(&self, x: f64, y: f64) -> bool {
        let dx = x - self.center_x;
        let dy = y - self.center_y;
        dx * dx + dy * dy <= self.radius * self.radius
    }
    
    fn contains_xyz(&self, x: f64, y: f64, _z: f64) -> bool {
        self.contains_xy(x, y)
    }
    
    fn intersects_rect(&self, rect: &Rect<f64>) -> bool {
        // Implement rectangle intersection test
        true // Simplified example
    }
}
```

## Dependencies

- **las**: LAS/LAZ point cloud I/O
- **geo**: Spatial geometry types
- **rusqlite**: SQLite/GeoPackage database access
- **rayon**: Parallel processing
- **anyhow/thiserror**: Error handling

## License

GPL-3.0