"""Python bindings for pointslicer - extract points from LAS/LAZ files using GeoPackage tile indices."""

from __future__ import annotations

from typing import Union

__version__: str

class ExtractionStats:
    """Statistics about an extraction operation."""

    tiles_processed: int
    points_read: int
    points_written: int
    elapsed_time: float

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

class BoundingBox:
    """A 2D or 3D axis-aligned bounding box geometry for point extraction."""

    min_x: float
    max_x: float
    min_y: float
    max_y: float
    min_z: float | None
    max_z: float | None

    def __init__(
        self,
        min_x: float,
        max_x: float,
        min_y: float,
        max_y: float,
        min_z: float | None = ...,
        max_z: float | None = ...,
    ) -> None: ...
    @staticmethod
    def new_2d(
        min_x: float, max_x: float, min_y: float, max_y: float
    ) -> BoundingBox: ...
    @staticmethod
    def new_3d(
        min_x: float,
        max_x: float,
        min_y: float,
        max_y: float,
        min_z: float,
        max_z: float,
    ) -> BoundingBox: ...
    def __repr__(self) -> str: ...
    def contains(self, x: float, y: float, z: float | None = ...) -> bool: ...

class Cylinder:
    """A vertical cylinder geometry for point extraction."""

    center_x: float
    center_y: float
    radius: float
    diameter: float

    def __init__(self, center_x: float, center_y: float, radius: float) -> None: ...
    @staticmethod
    def from_diameter(
        center_x: float, center_y: float, diameter: float
    ) -> Cylinder: ...
    def __repr__(self) -> str: ...
    def contains(self, x: float, y: float, z: float = ...) -> bool: ...

class Frustum:
    """A frustum (truncated pyramid) geometry for point extraction.

    A frustum is defined by an origin point, angular bounds (two corner angles),
    and distance bounds (near and far planes).

    Uses geographic coordinates:
    - phi (azimuth): measured clockwise from north (0-360 degrees)
    - theta (elevation): measured above/below horizontal (-90 to +90 degrees)
    """

    origin_x: float
    origin_y: float
    origin_z: float
    min_distance: float
    max_distance: float

    def __init__(
        self,
        origin_x: float,
        origin_y: float,
        origin_z: float,
        phi1: float,
        theta1: float,
        phi2: float,
        theta2: float,
        min_distance: float,
        max_distance: float,
    ) -> None: ...
    def __repr__(self) -> str: ...
    def contains(self, x: float, y: float, z: float) -> bool: ...

Geometry = Union[BoundingBox, Cylinder, Frustum]

def extract(
    index_path: str,
    output_path: str,
    geometry: Geometry,
    verbose: bool = ...,
) -> ExtractionStats:
    """Extract points from LAS/LAZ files using a GeoPackage tile index.

    Args:
        index_path: Path to the GeoPackage tile index file.
        output_path: Path where the extracted points will be written.
        geometry: Extraction geometry (BoundingBox, Cylinder, or Frustum).
        verbose: Whether to log detailed progress information.

    Returns:
        Statistics about the extraction process.

    Raises:
        ValueError: If the geometry is not valid.
        RuntimeError: If the extraction fails.
    """
    ...
