# pointslicer

A Rust CLI tool for extracting points from LAS/LAZ files using a GeoPackage tile index created by `pdal tindex`.

## Features

- **GeoPackage tile index support**: Reads tile indices created by `pdal tindex`
- **LAS/LAZ I/O**: Supports both compressed (LAZ) and uncompressed (LAS) point cloud files
- **Multiple extraction geometries**:
  - Vertical cylinder (center X, Y + diameter)
  - Bounding box (2D or 3D)
- **Parallel processing**: Uses Rayon for efficient multi-threaded tile processing
- **Extensible architecture**: Easy to add new extraction geometries

## Installation

```bash
cargo build --release
```

The binary will be available at `./target/release/pointslicer`.

## Usage

### Extract points within a vertical cylinder

```bash
pointslicer \
  --index tiles.gpkg \
  --output extracted.laz \
  cylinder --x 12345.0 --y 67890.0 --diameter 12.0
```

### Extract points within a bounding box

```bash
# 2D bounding box
pointslicer \
  --index tiles.gpkg \
  --output extracted.laz \
  bbox --min-x 10000 --max-x 20000 --min-y 30000 --max-y 40000

# 3D bounding box with Z constraints
pointslicer \
  --index tiles.gpkg \
  --output extracted.laz \
  bbox \
    --min-x 10000 --max-x 20000 \
    --min-y 30000 --max-y 40000 \
    --min-z 100 --max-z 200
```

### Verbose output

Add the `-v` flag for detailed logging:

```bash
pointslicer -v \
  --index tiles.gpkg \
  --output extracted.laz \
  cylinder --x 12345 --y 67890 --diameter 12
```

## Architecture

The tool is organized into several modules:

- **geometry**: Trait-based geometry system for extensible shape support
- **index**: GeoPackage tile index reader
- **pointcloud**: LAS/LAZ file I/O and point filtering
- **pipeline**: Orchestrates the extraction workflow with parallel processing
- **cli**: Command-line interface

## Adding New Extraction Geometries

To add a new extraction geometry (e.g., frustum, sphere):

1. Create a new file in `src/geometry/` (e.g., `frustum.rs`)
2. Define a struct with the geometry parameters
3. Implement the `ExtractGeometry` trait
4. Add a new CLI subcommand in `src/cli/commands.rs`
5. Add command handling in `src/cli/mod.rs`

No changes needed to the pipeline or I/O modules!

## Dependencies

- **clap**: CLI argument parsing
- **las**: LAS/LAZ point cloud I/O
- **geo**: Spatial geometry types
- **rusqlite**: SQLite/GeoPackage database access
- **rayon**: Parallel processing
- **anyhow/thiserror**: Error handling

## License

This project is open source. See LICENSE for details.
