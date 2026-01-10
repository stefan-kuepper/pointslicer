mod common;

use common::*;

#[test]
fn test_cli_cylinder_extraction_basic() {
    let fixture = TestFixture::standard().expect("Failed to create test fixture");

    let output = run_pointslicer(&[
        "--index",
        fixture.index_path.to_str().unwrap(),
        "--output",
        fixture.output_path.to_str().unwrap(),
        "cylinder",
        "--x",
        "100",
        "--y",
        "100",
        "--diameter",
        "50",
    ])
    .expect("Failed to run pointslicer");

    assert_success(&output);
    assert_laz_exists(&fixture.output_path);

    // Should have extracted some points
    let point_count = read_laz_point_count(&fixture.output_path).expect("Failed to read points");
    assert!(point_count > 0, "Expected some points to be extracted");
}

#[test]
fn test_cli_bbox_2d_extraction() {
    let fixture = TestFixture::standard().expect("Failed to create test fixture");

    let output = run_pointslicer(&[
        "--index",
        fixture.index_path.to_str().unwrap(),
        "--output",
        fixture.output_path.to_str().unwrap(),
        "bbox",
        "--min-x",
        "50",
        "--max-x",
        "150",
        "--min-y",
        "50",
        "--max-y",
        "150",
    ])
    .expect("Failed to run pointslicer");

    assert_success(&output);
    assert_laz_exists(&fixture.output_path);

    // Verify all extracted points are within bounds
    let points = read_laz_points(&fixture.output_path).expect("Failed to read points");
    assert!(!points.is_empty(), "Expected some points to be extracted");

    for point in &points {
        assert!(
            point.x >= 50.0 && point.x <= 150.0,
            "Point x={} is outside bounds [50, 150]",
            point.x
        );
        assert!(
            point.y >= 50.0 && point.y <= 150.0,
            "Point y={} is outside bounds [50, 150]",
            point.y
        );
    }
}

#[test]
fn test_cli_bbox_3d_extraction() {
    let fixture = TestFixture::standard().expect("Failed to create test fixture");

    let output = run_pointslicer(&[
        "--index",
        fixture.index_path.to_str().unwrap(),
        "--output",
        fixture.output_path.to_str().unwrap(),
        "bbox",
        "--min-x",
        "50",
        "--max-x",
        "150",
        "--min-y",
        "50",
        "--max-y",
        "150",
        "--min-z",
        "20",
        "--max-z",
        "60",
    ])
    .expect("Failed to run pointslicer");

    assert_success(&output);
    assert_laz_exists(&fixture.output_path);

    // Verify all extracted points are within 3D bounds including Z
    let points = read_laz_points(&fixture.output_path).expect("Failed to read points");
    assert!(!points.is_empty(), "Expected some points to be extracted");

    for point in &points {
        assert!(
            point.x >= 50.0 && point.x <= 150.0,
            "Point x={} is outside bounds [50, 150]",
            point.x
        );
        assert!(
            point.y >= 50.0 && point.y <= 150.0,
            "Point y={} is outside bounds [50, 150]",
            point.y
        );
        assert!(
            point.z >= 20.0 && point.z <= 60.0,
            "Point z={} is outside bounds [20, 60]",
            point.z
        );
    }
}

#[test]
fn test_cli_verbose_flag() {
    let fixture = TestFixture::standard().expect("Failed to create test fixture");

    let output = run_pointslicer(&[
        "-v",
        "--index",
        fixture.index_path.to_str().unwrap(),
        "--output",
        fixture.output_path.to_str().unwrap(),
        "cylinder",
        "--x",
        "100",
        "--y",
        "100",
        "--diameter",
        "50",
    ])
    .expect("Failed to run pointslicer");

    assert_success(&output);

    // Verbose mode should log processing information
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Processing tile") || stderr.contains("Found") || stderr.contains("tiles"),
        "Expected verbose logging in stderr but got: {}",
        stderr
    );
}

#[test]
fn test_cli_missing_index_file() {
    let fixture = TestFixture::standard().expect("Failed to create test fixture");

    let output = run_pointslicer(&[
        "--index",
        "/nonexistent/path/to/index.gpkg",
        "--output",
        fixture.output_path.to_str().unwrap(),
        "cylinder",
        "--x",
        "100",
        "--y",
        "100",
        "--diameter",
        "50",
    ])
    .expect("Failed to run pointslicer");

    assert!(!output.status.success(), "Expected command to fail");
}

#[test]
fn test_cli_missing_required_args() {
    // Missing --index argument
    let output = run_pointslicer(&[
        "--output",
        "output.laz",
        "cylinder",
        "--x",
        "100",
        "--y",
        "100",
        "--diameter",
        "50",
    ])
    .expect("Failed to run pointslicer");

    assert!(
        !output.status.success(),
        "Expected command to fail without --index"
    );
}

#[test]
fn test_cli_help_displays() {
    let output = run_pointslicer(&["--help"]).expect("Failed to run pointslicer");

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cylinder") && stdout.contains("bbox"),
        "Expected help text to contain both 'cylinder' and 'bbox' commands"
    );
}

#[test]
fn test_cli_version_displays() {
    let output = run_pointslicer(&["--version"]).expect("Failed to run pointslicer");

    assert_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("pointslicer") || stdout.contains("0.1"),
        "Expected version output to contain 'pointslicer' or version number"
    );
}
