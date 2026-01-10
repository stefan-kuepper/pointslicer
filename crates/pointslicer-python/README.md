# pointslicer Python Bindings

Python bindings for the `pointslicer` library, allowing you to extract points from LAS/LAZ files using GeoPackage tile indices.

## Installation

```bash
pip install pointslicer
```

Or from source:

```bash
pip install maturin
maturin develop
```

## Usage

```python
import pointslicer

# Create a bounding box geometry
bbox = pointslicer.BoundingBox(10000, 20000, 30000, 40000)

# Or create a cylinder geometry
cylinder = pointslicer.Cylinder(12345, 67890, 6.0)

# Extract points from tile index
stats = pointslicer.extract(
    index_path="tiles.gpkg",
    output_path="output.laz",
    geometry=bbox,
    verbose=True
)

print(f"Extracted {stats.points_written} points from {stats.tiles_processed} tiles")
print(f"Time: {stats.elapsed_time:.2f} seconds")
```

## API Reference

### `BoundingBox`

A 2D or 3D axis-aligned bounding box for point extraction.

```python
# 2D bounding box
bbox = pointslicer.BoundingBox(min_x, max_x, min_y, max_y)

# 3D bounding box  
bbox = pointslicer.BoundingBox(min_x, max_x, min_y, max_y, min_z, max_z)

# Convenience methods
bbox = pointslicer.BoundingBox.new_2d(min_x, max_x, min_y, max_y)
bbox = pointslicer.BoundingBox.new_3d(min_x, max_x, min_y, max_y, min_z, max_z)

# Check if a point is inside
bbox.contains(x, y)           # 2D check
bbox.contains(x, y, z)        # 3D check (only for 3D boxes)
```

### `Cylinder`

A vertical cylinder defined by center coordinates and radius.

```python
# From radius
cylinder = pointslicer.Cylinder(center_x, center_y, radius)

# From diameter
cylinder = pointslicer.Cylinder.from_diameter(center_x, center_y, diameter)

# Check if a point is inside
cylinder.contains(x, y, z)    # z is ignored for vertical cylinders
```

### `extract()`

Main function for extracting points.

```python
stats = pointslicer.extract(
    index_path="tiles.gpkg",      # Path to GeoPackage tile index
    output_path="output.laz",     # Output LAS/LAZ file
    geometry=bbox_or_cylinder,    # Extraction geometry
    verbose=False                 # Enable verbose logging
)
```

### `ExtractionStats`

Statistics returned by the extraction process.

```python
stats.tiles_processed    # Number of tiles processed
stats.points_read        # Total points read
stats.points_written     # Points that passed geometry filter
stats.elapsed_time       # Extraction time in seconds
```

## Requirements

- Python 3.8+
- GeoPackage tile index created by `pdal tindex`
- LAS/LAZ point cloud files

## License

GPL-3.0