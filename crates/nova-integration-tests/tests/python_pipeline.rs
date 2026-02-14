//! Integration tests for the Python figure generation pipeline.
//!
//! These tests require Python + matplotlib + nova to be installed.
//! They skip gracefully if the environment is not available.

use nova_python::{FigureDiscovery, NovaPython, PythonConfig};
use std::path::PathBuf;
use tempfile::TempDir;

fn python_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/python")
}

fn nova_python_available() -> bool {
    nova_test_helpers::nova_python_available()
}

/// Copy the python fixture into a temp dir (to avoid polluting fixtures with cache).
fn setup_python_project(temp_dir: &TempDir) -> PathBuf {
    let src = python_fixture_path();
    let dest = temp_dir.path();

    // Copy nova.toml
    std::fs::copy(src.join("nova.toml"), dest.join("nova.toml")).unwrap();

    // Copy figures directory
    let fig_dest = dest.join("figures");
    std::fs::create_dir_all(&fig_dest).unwrap();
    std::fs::copy(
        src.join("figures/__init__.py"),
        fig_dest.join("__init__.py"),
    )
    .unwrap();
    std::fs::copy(src.join("figures/minimal.py"), fig_dest.join("minimal.py")).unwrap();

    dest.to_path_buf()
}

// ─── Discovery tests (no Python runtime needed) ────────────────────

#[test]
fn figure_discovery_finds_decorators() {
    let fixture = python_fixture_path();
    let figures_dir = fixture.join("figures");

    if !figures_dir.exists() {
        eprintln!("Skipping: fixture not found at {:?}", figures_dir);
        return;
    }

    let figures = FigureDiscovery::scan(&figures_dir).unwrap();

    assert!(!figures.is_empty(), "Should discover at least one figure");

    let names: Vec<&str> = figures.iter().map(|f| f.name.as_str()).collect();
    assert!(
        names.contains(&"test-circle"),
        "Should find the test-circle figure, found: {:?}",
        names
    );
}

#[test]
fn figure_discovery_returns_empty_for_no_decorators() {
    let temp_dir = TempDir::new().unwrap();
    let figures_dir = temp_dir.path().join("figures");
    std::fs::create_dir_all(&figures_dir).unwrap();

    // Create a Python file without @nova.figure
    std::fs::write(
        figures_dir.join("no_figures.py"),
        "def regular_function():\n    pass\n",
    )
    .unwrap();

    let figures = FigureDiscovery::scan(&figures_dir).unwrap();
    assert!(figures.is_empty(), "Should find no figures");
}

#[test]
fn figure_discovery_on_nonexistent_dir() {
    let result = FigureDiscovery::scan("/nonexistent/dir");
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

// ─── Full pipeline tests (require Python + matplotlib + nova) ──────

#[tokio::test]
async fn python_full_pipeline_generates_svg() {
    if !nova_python_available() {
        eprintln!("Skipping: Python + nova + matplotlib not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let project_root = setup_python_project(&temp_dir);

    let config = PythonConfig::from_project(&project_root).unwrap();
    let nova = NovaPython::new(config).unwrap();
    let registry = nova.generate_figures().await.unwrap();

    assert!(
        !registry.is_empty(),
        "Should have generated at least one figure"
    );
    assert!(
        registry.contains("test-circle"),
        "Registry should contain 'test-circle'"
    );

    // Verify the SVG file exists and is valid
    let svg_path = registry.get("test-circle").unwrap();
    assert!(svg_path.exists(), "SVG file should exist at {:?}", svg_path);

    let svg_content = std::fs::read_to_string(svg_path).unwrap();
    assert!(
        svg_content.contains("<svg") || svg_content.contains("<?xml"),
        "Output should be valid SVG"
    );
}

#[tokio::test]
async fn python_cache_avoids_regeneration() {
    if !nova_python_available() {
        eprintln!("Skipping: Python + nova + matplotlib not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let project_root = setup_python_project(&temp_dir);

    // First generation
    let config = PythonConfig::from_project(&project_root).unwrap();
    let nova = NovaPython::new(config).unwrap();
    let _ = nova.generate_figures().await.unwrap();

    // Get the SVG modification time
    let cache_dir = project_root.join(".nova/cache");
    let svg_path = cache_dir.join("test-circle.svg");
    assert!(svg_path.exists());
    let first_mtime = std::fs::metadata(&svg_path).unwrap().modified().unwrap();

    // Small delay to ensure mtime difference would be detectable
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Second generation (should use cache)
    let config2 = PythonConfig::from_project(&project_root).unwrap();
    let nova2 = NovaPython::new(config2).unwrap();
    let _ = nova2.generate_figures().await.unwrap();

    let second_mtime = std::fs::metadata(&svg_path).unwrap().modified().unwrap();
    assert_eq!(
        first_mtime, second_mtime,
        "SVG should not have been regenerated (cache hit)"
    );
}

#[tokio::test]
async fn python_config_from_project_works() {
    let fixture = python_fixture_path();
    if !fixture.join("nova.toml").exists() {
        eprintln!("Skipping: fixture not found");
        return;
    }

    let config = PythonConfig::from_project(&fixture);
    assert!(config.is_ok(), "Should parse config: {:?}", config.err());

    let config = config.unwrap();
    assert_eq!(config.figures_dir, fixture.join("figures"));
    assert_eq!(config.timeout, 30);
}

#[test]
fn python_config_missing_toml_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let result = PythonConfig::from_project(temp_dir.path());
    assert!(result.is_err(), "Should fail without nova.toml");
}
