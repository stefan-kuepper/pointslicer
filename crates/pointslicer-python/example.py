#!/usr/bin/env python3
"""
Example usage of the pointslicer Python module.

This script demonstrates how to use the pointslicer Python bindings
to extract points from LAS/LAZ files using GeoPackage tile indices.
"""

import pointslicer


def main():
    print("=== pointslicer Python Bindings Example ===\n")

    # 1. Create geometries
    print("1. Creating extraction geometries:")

    # Bounding box (2D)
    bbox = pointslicer.BoundingBox(
        min_x=10000.0, max_x=20000.0, min_y=30000.0, max_y=40000.0
    )
    print(f"   - 2D BoundingBox: {bbox}")
    print(f"     Contains (15000, 35000): {bbox.contains(15000, 35000)}")
    print(f"     Contains (5000, 35000): {bbox.contains(5000, 35000)}")

    # Bounding box (3D)
    bbox_3d = pointslicer.BoundingBox.new_3d(
        min_x=10000.0,
        max_x=20000.0,
        min_y=30000.0,
        max_y=40000.0,
        min_z=500.0,
        max_z=1000.0,
    )
    print(f"   - 3D BoundingBox: {bbox_3d}")
    print(f"     Contains (15000, 35000, 750): {bbox_3d.contains(15000, 35000, 750)}")
    print(f"     Contains (15000, 35000, 2000): {bbox_3d.contains(15000, 35000, 2000)}")

    # Cylinder
    cylinder = pointslicer.Cylinder(center_x=12345.0, center_y=67890.0, radius=12.5)
    print(f"   - Cylinder: {cylinder}")
    print(f"     Contains (12345, 67890): {cylinder.contains(12345, 67890)}")
    print(f"     Contains (13000, 67890): {cylinder.contains(13000, 67890)}")
    print(f"     Diameter: {cylinder.diameter}")

    # Cylinder from diameter
    cylinder_diameter = pointslicer.Cylinder.from_diameter(
        center_x=100.0, center_y=200.0, diameter=20.0
    )
    print(f"   - Cylinder from diameter: {cylinder_diameter}")
    print(f"     Radius: {cylinder_diameter.radius}")

    print("\n2. Example extraction (commented out - requires actual data):")
    print("""
    # Extract points using a bounding box
    # stats = pointslicer.extract(
    #     index_path="tiles.gpkg",
    #     output_path="output.laz",
    #     geometry=bbox,
    #     verbose=True
    # )
    # 
    # print(f"\\nExtraction Statistics:")
    # print(f"  Tiles processed: {stats.tiles_processed}")
    # print(f"  Points read: {stats.points_read}")
    # print(f"  Points written: {stats.points_written}")
    # print(f"  Elapsed time: {stats.elapsed_time:.2f} seconds")
    # print(f"  Summary: {stats}")
    """)

    print("\n3. Module information:")
    print(f"   Version: {pointslicer.__version__}")
    print(f"   Documentation: {pointslicer.__doc__}")

    print("\n=== Example completed successfully ===")


if __name__ == "__main__":
    main()
