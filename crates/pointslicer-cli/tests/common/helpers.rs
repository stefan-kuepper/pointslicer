use las::{Read, Reader};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Get the path to the compiled pointslicer binary
pub fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up from pointslicer-cli
    path.pop(); // Go up from crates
    path.push("target");
    path.push(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });
    path.push("pointslicer");
    path
}

/// Run pointslicer with given arguments
pub fn run_pointslicer(args: &[&str]) -> anyhow::Result<Output> {
    let binary = get_binary_path();
    let output = Command::new(binary).args(args).output()?;
    Ok(output)
}

/// Assert that command succeeded (exit code 0)
pub fn assert_success(output: &Output) {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "Command failed with exit code {:?}\nStdout: {}\nStderr: {}",
            output.status.code(),
            stdout,
            stderr
        );
    }
}

/// Assert that command failed with expected error message
pub fn assert_failure(output: &Output, expected_msg: &str) {
    assert!(
        !output.status.success(),
        "Expected command to fail but it succeeded"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected_msg),
        "Expected stderr to contain '{}' but got: {}",
        expected_msg,
        stderr
    );
}

/// Assert that LAZ file exists
pub fn assert_laz_exists(path: &Path) {
    assert!(
        path.exists(),
        "Expected LAZ file to exist at {:?} but it doesn't",
        path
    );
}

/// Assert LAZ file has expected point count
pub fn assert_laz_point_count(path: &Path, expected: u64) -> anyhow::Result<()> {
    let actual = read_laz_point_count(path)?;
    assert_eq!(
        actual, expected,
        "Expected {} points in {:?} but found {}",
        expected, path, actual
    );
    Ok(())
}

/// Read the point count from a LAZ file
pub fn read_laz_point_count(path: &Path) -> anyhow::Result<u64> {
    let reader = Reader::from_path(path)?;
    Ok(reader.header().number_of_points())
}

/// Read all points from a LAZ file
pub fn read_laz_points(path: &Path) -> anyhow::Result<Vec<las::Point>> {
    let mut reader = Reader::from_path(path)?;
    let points: Result<Vec<_>, _> = reader.points().collect();
    Ok(points?)
}

/// Get the point format from a LAZ file
pub fn get_laz_point_format(path: &Path) -> anyhow::Result<u8> {
    let reader = Reader::from_path(path)?;
    Ok(reader.header().point_format().to_u8()?)
}

/// Assert that stderr contains a specific message
pub fn assert_stderr_contains(output: &Output, text: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(text),
        "Expected stderr to contain '{}' but got: {}",
        text,
        stderr
    );
}

/// Assert that stdout contains a specific message
pub fn assert_stdout_contains(output: &Output, text: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(text),
        "Expected stdout to contain '{}' but got: {}",
        text,
        stdout
    );
}
