# pointslicer-cli

Command-line interface for the pointslicer point cloud extraction tool.

## Overview

`pointslicer-cli` is the command-line interface for extracting points from LAS/LAZ files using GeoPackage tile indices. It provides a user-friendly CLI built on top of the `pointslicer-core` library.

## Installation

Build from source:

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

### Help

View available commands and options:

```bash
pointslicer --help
pointslicer cylinder --help
pointslicer bbox --help
```

## Commands

### `cylinder`

Extract points within a vertical cylinder.

**Arguments:**
- `--x`, `--y`: Center coordinates of the cylinder
- `--diameter`: Diameter of the cylinder (or use `--radius`)

### `bbox`

Extract points within a bounding box.

**Arguments:**
- `--min-x`, `--max-x`: X-axis bounds
- `--min-y`, `--max-y`: Y-axis bounds
- `--min-z`, `--max-z`: Optional Z-axis bounds (for 3D extraction)

## Global Options

- `-i, --index <INDEX>`: Path to GeoPackage tile index (required)
- `-o, --output <OUTPUT>`: Path to output LAS/LAZ file (required)
- `-v, --verbose`: Enable verbose logging

## Architecture

The CLI is built using:
- **clap**: Command-line argument parsing with derive macros
- **pointslicer-core**: Core extraction library

Command structure is defined in `src/cli/commands.rs` and executed in `src/cli/mod.rs`.

## Development

To add a new command:

1. Add a new variant to the `Commands` enum in `src/cli/commands.rs`
2. Add command-line arguments using `#[command()]` attributes
3. Handle the new command in the match statement in `src/cli/mod.rs`

## License

GPL-3.0