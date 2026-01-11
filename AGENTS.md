# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**pointslicer** is a Rust CLI tool for extracting points from LAS/LAZ point cloud files using a GeoPackage tile index created by `pdal tindex`. It uses parallel processing (Rayon) to efficiently filter large point cloud datasets based on geometric queries.

## Project Structure

This is a Cargo workspace with three crates:

```
pointslicer/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── pointslicer-core/   # Library crate with core application logic
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── geometry/   # Extensible geometry system
│   │       ├── index/      # GeoPackage tile index reading
│   │       ├── pipeline/   # Extraction pipeline orchestration
│   │       └── pointcloud/ # LAS/LAZ I/O
│   ├── pointslicer-cli/    # Binary crate with CLI
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   └── cli/        # Command-line parsing and execution
│   │   └── tests/          # Integration tests
│   └── pointslicer-python/ # Python bindings crate
│       ├── src/
│       │   └── lib.rs      # PyO3 bindings
│       ├── pyproject.toml  # Python packaging
│       ├── README.md       # Python documentation
│       └── example.py      # Python usage example
```

## Development Commands

### Build
```bash
cargo build --release
```
Binary is located at `./target/release/pointslicer`

### Testing
```bash
cargo test
```

### Linting
```bash
cargo clippy
```

### Running the Tool
```bash
# Using the release binary
./target/release/pointslicer --index tiles.gpkg --output extracted.laz cylinder --x 12345.0 --y 67890.0 --diameter 12.0

# With verbose logging
./target/release/pointslicer -v --index tiles.gpkg --output extracted.laz bbox --min-x 10000 --max-x 20000 --min-y 30000 --max-y 40000
```

### Python Usage
```python
import pointslicer

# Create geometries
bbox = pointslicer.BoundingBox(10000, 20000, 30000, 40000)
cylinder = pointslicer.Cylinder(12345, 67890, 6.0)

# Extract points
stats = pointslicer.extract(
    index_path="tiles.gpkg",
    output_path="output.laz",
    geometry=bbox,
    verbose=True
)

print(f"Extracted {stats.points_written} points")
```

Build Python module:
```bash
cd crates/pointslicer-python
maturin develop  # For development
maturin build    # For distribution
```

## Architecture

### Workspace Structure

- **pointslicer-core**: Library crate containing all core functionality
  - Can be used as a dependency by other Rust projects
  - Contains geometry, index reading, point cloud I/O, and pipeline logic

- **pointslicer-cli**: Binary crate containing the CLI
  - Depends on pointslicer-core
  - Contains command-line parsing and execution logic

- **pointslicer-python**: Python bindings crate using PyO3
  - Depends on pointslicer-core
  - Provides Python bindings for the core functionality
  - Built with maturin for Python packaging

### Trait-Based Geometry System
The core abstraction is the `ExtractGeometry` trait (`crates/pointslicer-core/src/geometry/traits.rs`), which defines the interface for any geometry that can extract points:
- `bounding_box()`: Returns 2D bounds for spatial indexing
- `contains_xy()` / `contains_xyz()`: Point-in-geometry tests
- `intersects_rect()`: Tile intersection test

All geometry implementations must be `Send + Sync` for parallel processing.

### Pipeline Flow
1. **Index Reading** (`crates/pointslicer-core/src/index/reader.rs`): Opens GeoPackage, queries tiles that intersect with the extraction geometry's bounding box
2. **Parallel Processing** (`crates/pointslicer-core/src/pipeline/executor.rs`): Uses Rayon to process tiles in parallel
3. **Point Filtering** (`crates/pointslicer-core/src/pointcloud/filter.rs`): Filters points using geometry's `contains_xyz()` method
4. **Output Writing** (`crates/pointslicer-core/src/pointcloud/writer.rs`): Writes filtered points to LAS/LAZ, preserving point format from source

### GeoPackage Tile Index Format
The tool reads GeoPackage tile indices created by `pdal tindex`. Key implementation details:
- Queries `gpkg_contents` to find feature tables
- Queries `gpkg_geometry_columns` to find geometry column name
- Parses GeoPackage Binary Format header to extract tile envelopes (min_x, max_x, min_y, max_y)
- Expects a `location` column with the file path to each LAS/LAZ tile

Envelope type handling in `crates/pointslicer-core/src/index/reader.rs:114-194`:
- Type 1: XY (32 bytes)
- Type 2: XYZ (48 bytes)
- Type 3: XYM (48 bytes)
- Type 4: XYZM (64 bytes)
- Type 0: No envelope (not yet supported, would require full WKB parsing)

### Point Format Preservation
The pipeline preserves the point format from the source tiles when writing output (`crates/pointslicer-core/src/pipeline/executor.rs:115-125`). It:
1. Opens the first tile to get its header
2. Uses `las::Builder::from()` to clone the header
3. Preserves the `point_format` field
4. Creates a new header with the preserved format

## Adding New Extraction Geometries

To add a new geometry type (e.g., sphere, frustum, polygon):

1. Create `crates/pointslicer-core/src/geometry/your_geometry.rs`
2. Define a struct with geometry parameters
3. Implement the `ExtractGeometry` trait with all required methods
4. Add a new variant to `Commands` enum in `crates/pointslicer-cli/src/cli/commands.rs`
5. Handle the new command in the match statement in `crates/pointslicer-cli/src/cli/mod.rs:21-41`
6. Export the new geometry from `crates/pointslicer-core/src/geometry/mod.rs`

**No changes needed** to the pipeline, index reader, or point cloud I/O modules due to the trait-based design.

## Module Responsibilities

### pointslicer-core (library)
- **error**: Error types using thiserror
- **geometry**: Extensible geometry system with trait and implementations
- **index**: GeoPackage tile index reading and spatial filtering
- **pointcloud**: LAS/LAZ I/O (reader, writer, filtering logic)
- **pipeline**: Orchestrates the extraction workflow with parallel processing

### pointslicer-cli (binary)
- **cli**: Command-line parsing (clap) and execution orchestration

## Key Dependencies

- **las** (v0.8): LAS/LAZ point cloud I/O with LAZ compression support
- **rusqlite** (v0.27): SQLite/GeoPackage database access (bundled)
- **geo/geo-types**: Spatial geometry types (`Rect`, `coord!`, etc.)
- **rayon** (v1.11): Data parallelism for tile processing
- **clap** (v4): CLI with derive macros
- **gpkg** (v0.1): GeoPackage utilities
- **wkb** (v0.7): Well-Known Binary format support
- **pyo3** (v0.27): Python bindings for Rust (for pointslicer-python crate)
