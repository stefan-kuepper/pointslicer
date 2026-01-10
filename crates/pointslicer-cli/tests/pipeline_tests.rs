mod common;

use common::*;
use geo::coord;
use pointslicer_core::geometry::{BoundingBox, VerticalCylinder};
use pointslicer_core::pipeline::ExtractionPipeline;

#[test]
fn test_pipeline_execute_cylinder() {
    let fixture = TestFixture::standard().expect("Failed to create test fixture");

    // Create a cylinder that intersects both tiles
    let geometry = VerticalCylinder::from_diameter(100.0, 100.0, 50.0);

    // Execute pipeline
    let pipeline = ExtractionPipeline::new(&fixture.index_path, &fixture.output_path, false);
    let stats = pipeline
        .execute(&geometry)
        .expect("Pipeline execution failed");

    // Verify statistics
    assert_eq!(stats.tiles_processed, 2, "Expected 2 tiles to be processed");
    assert!(stats.points_read > 0, "Expected some points to be read");
    assert!(
        stats.points_written > 0,
        "Expected some points to be written"
    );
    assert!(
        stats.points_written <= stats.points_read,
        "Written points should not exceed read points"
    );

    // Verify output file exists and has points
    assert_laz_exists(&fixture.output_path);
    let point_count = read_laz_point_count(&fixture.output_path).expect("Failed to read points");
    assert_eq!(
        point_count, stats.points_written,
        "Point count in file should match stats"
    );
}

#[test]
fn test_pipeline_execute_bbox() {
    let fixture = TestFixture::standard().expect("Failed to create test fixture");

    // Create a 2D bounding box
    let geometry = BoundingBox::new_2d(50.0, 150.0, 50.0, 150.0);

    // Execute pipeline
    let pipeline = ExtractionPipeline::new(&fixture.index_path, &fixture.output_path, false);
    let stats = pipeline
        .execute(&geometry)
        .expect("Pipeline execution failed");

    // Verify statistics
    assert!(stats.tiles_processed > 0, "Expected tiles to be processed");
    assert!(
        stats.points_written > 0,
        "Expected some points to be written"
    );

    // Verify all points are within bounds
    let points = read_laz_points(&fixture.output_path).expect("Failed to read points");
    for point in &points {
        assert!(
            point.x >= 50.0 && point.x <= 150.0 && point.y >= 50.0 && point.y <= 150.0,
            "Point ({}, {}) is outside bbox bounds",
            point.x,
            point.y
        );
    }
}

#[test]
fn test_pipeline_no_matching_points() {
    let fixture = TestFixture::standard().expect("Failed to create test fixture");

    // Create a very small cylinder at the edge of the tile bounds
    // Tile 2 goes to (200, 200), but points are distributed with some spacing
    // Use a tiny cylinder just outside the actual point distribution
    let geometry = VerticalCylinder::from_diameter(200.0, 200.0, 0.1);

    // Execute pipeline - should succeed but likely write 0 or very few points
    let pipeline = ExtractionPipeline::new(&fixture.index_path, &fixture.output_path, false);
    let stats = pipeline
        .execute(&geometry)
        .expect("Pipeline execution failed");

    // Very small cylinder should extract 0 or very few points
    assert!(
        stats.points_written <= 2,
        "Expected very few points to be written, got {}",
        stats.points_written
    );
}

#[test]
fn test_pipeline_preserves_point_format() {
    // Create fixture with point format 2
    use geo::Rect;
    let bounds = Rect::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 100.0, y: 100.0 });
    let points = generate_grid_points(0.0, 0.0, 100.0, 100.0, 5, 0.0, 50.0);

    let tile_spec = TileSpec {
        file_name: "tile_fmt2.laz".to_string(),
        bounds,
        points,
    };

    let fixture = TestFixture::new(vec![tile_spec]).expect("Failed to create fixture");

    // Create geometry to extract all points
    let geometry = BoundingBox::new_2d(0.0, 100.0, 0.0, 100.0);

    // Execute pipeline
    let pipeline = ExtractionPipeline::new(&fixture.index_path, &fixture.output_path, false);
    pipeline
        .execute(&geometry)
        .expect("Pipeline execution failed");

    // Verify output has same point format as source
    let source_format =
        get_laz_point_format(&fixture.tile_paths[0]).expect("Failed to read source format");
    let output_format =
        get_laz_point_format(&fixture.output_path).expect("Failed to read output format");

    assert_eq!(
        source_format, output_format,
        "Output point format should match source"
    );
}

#[test]
fn test_pipeline_no_intersecting_tiles() {
    let fixture = TestFixture::standard().expect("Failed to create test fixture");

    // Create a bbox that doesn't intersect any tiles (tiles are roughly 0-200)
    let geometry = BoundingBox::new_2d(500.0, 600.0, 500.0, 600.0);

    // Execute pipeline - should fail with NoTilesFound error
    let pipeline = ExtractionPipeline::new(&fixture.index_path, &fixture.output_path, false);
    let result = pipeline.execute(&geometry);

    assert!(
        result.is_err(),
        "Expected pipeline to fail when no tiles intersect"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("No tiles") || error_msg.contains("tiles"),
        "Expected error message about no tiles, got: {}",
        error_msg
    );
}

#[test]
fn test_pipeline_parallel_processing() {
    // Create fixture with 4 tiles for parallel processing
    let fixture = TestFixture::multi_tile(4).expect("Failed to create fixture");

    // Create geometry that intersects all tiles
    let geometry = BoundingBox::new_2d(0.0, 400.0, 0.0, 100.0);

    // Execute pipeline
    let pipeline = ExtractionPipeline::new(&fixture.index_path, &fixture.output_path, false);
    let stats = pipeline
        .execute(&geometry)
        .expect("Pipeline execution failed");

    // Verify all tiles were processed
    assert_eq!(
        stats.tiles_processed, 4,
        "Expected all 4 tiles to be processed"
    );

    // Verify total point count is correct
    assert!(stats.points_read > 0, "Expected points to be read");
    assert!(stats.points_written > 0, "Expected points to be written");

    // Verify output file has the correct number of points
    let point_count = read_laz_point_count(&fixture.output_path).expect("Failed to read points");
    assert_eq!(
        point_count, stats.points_written,
        "Point count should match stats"
    );
}

#[test]
fn test_pipeline_with_verbose_logging() {
    let fixture = TestFixture::standard().expect("Failed to create test fixture");
    let geometry = VerticalCylinder::from_diameter(100.0, 100.0, 50.0);

    // Execute with verbose = true
    let pipeline = ExtractionPipeline::new(&fixture.index_path, &fixture.output_path, true);

    // This should execute successfully with logging enabled
    // Note: We can't easily test log output in unit tests without capturing,
    // but we verify it doesn't crash and produces correct results
    let stats = pipeline
        .execute(&geometry)
        .expect("Pipeline execution failed");

    assert!(stats.tiles_processed > 0, "Expected tiles to be processed");
    assert_laz_exists(&fixture.output_path);
}
