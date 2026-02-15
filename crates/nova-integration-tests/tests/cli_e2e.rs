//! End-to-end CLI tests for the `nova` binary.
//!
//! These tests run the actual `nova` binary via `assert_cmd` and verify
//! exit codes, stdout/stderr, and output files.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::sync::Once;
use tempfile::TempDir;

/// Ensure the `nova` binary is built before tests run.
static BUILD_NOVA: Once = Once::new();

fn ensure_nova_built() {
    BUILD_NOVA.call_once(|| {
        let status = std::process::Command::new("cargo")
            .args(["build", "-p", "novatype-cli"])
            .status()
            .expect("failed to run cargo build");
        assert!(status.success(), "cargo build -p novatype-cli failed");
    });
}

/// Helper to get a `nova` command.
fn nova() -> Command {
    ensure_nova_built();
    #[allow(deprecated)]
    Command::cargo_bin("nova").expect("nova binary not found")
}

/// Copy a fixture file into a temp directory.
fn copy_fixture(temp_dir: &TempDir, fixture_name: &str) -> std::path::PathBuf {
    let fixture_src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(fixture_name);
    let dest = temp_dir.path().join(fixture_name);
    fs::copy(&fixture_src, &dest).unwrap_or_else(|e| {
        panic!(
            "Failed to copy fixture {:?} to {:?}: {}",
            fixture_src, dest, e
        )
    });
    dest
}

// ─── Compile command ────────────────────────────────────────────────

#[test]
fn cli_compile_simple_to_pdf() {
    let temp_dir = TempDir::new().unwrap();
    let input = copy_fixture(&temp_dir, "minimal.typ");
    let output = temp_dir.path().join("output.pdf");

    nova()
        .args([
            "compile",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--no-python",
        ])
        .assert()
        .success();

    assert!(output.exists(), "PDF output file should exist");
    let content = fs::read(&output).unwrap();
    assert_eq!(&content[0..5], b"%PDF-", "Output should be a valid PDF");
}

#[test]
fn cli_compile_to_svg() {
    let temp_dir = TempDir::new().unwrap();
    let input = copy_fixture(&temp_dir, "minimal.typ");
    let output = temp_dir.path().join("output.svg");

    nova()
        .args([
            "compile",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-f",
            "svg",
            "--no-python",
        ])
        .assert()
        .success();

    assert!(output.exists(), "SVG output file should exist");
    let content = fs::read_to_string(&output).unwrap();
    assert!(content.contains("<svg"), "Output should be valid SVG");
}

#[test]
fn cli_compile_with_custom_output_path() {
    let temp_dir = TempDir::new().unwrap();
    let input = copy_fixture(&temp_dir, "minimal.typ");
    let custom_dir = temp_dir.path().join("custom");
    fs::create_dir_all(&custom_dir).unwrap();
    let output = custom_dir.join("my-document.pdf");

    nova()
        .args([
            "compile",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--no-python",
        ])
        .assert()
        .success();

    assert!(output.exists(), "Custom output path should have the PDF");
}

#[test]
fn cli_compile_nonexistent_file_fails() {
    nova()
        .args(["compile", "/nonexistent/file.typ", "--no-python"])
        .assert()
        .failure();
}

#[test]
fn cli_compile_invalid_typst_fails() {
    let temp_dir = TempDir::new().unwrap();
    let input = copy_fixture(&temp_dir, "invalid-syntax.typ");
    let output = temp_dir.path().join("output.pdf");

    nova()
        .args([
            "compile",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--no-python",
        ])
        .assert()
        .failure();
}

#[test]
fn cli_compile_with_headings() {
    let temp_dir = TempDir::new().unwrap();
    let input = copy_fixture(&temp_dir, "with-headings.typ");
    let output = temp_dir.path().join("output.pdf");

    nova()
        .args([
            "compile",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--no-python",
        ])
        .assert()
        .success();

    assert!(output.exists());
}

#[test]
fn cli_compile_with_math() {
    let temp_dir = TempDir::new().unwrap();
    let input = copy_fixture(&temp_dir, "with-math.typ");
    let output = temp_dir.path().join("output.pdf");

    nova()
        .args([
            "compile",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--no-python",
        ])
        .assert()
        .success();

    assert!(output.exists());
}

// ─── Init command ───────────────────────────────────────────────────

#[test]
fn cli_init_creates_project() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("my-project");

    nova()
        .args(["init", project_dir.to_str().unwrap(), "--no-git"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created project"));

    assert!(project_dir.join("main.typ").exists());
    assert!(project_dir.join("nova.toml").exists());
    assert!(project_dir.join("references.bib").exists());
    assert!(project_dir.join(".gitignore").exists());
}

#[test]
fn cli_init_with_python() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("python-project");

    nova()
        .args([
            "init",
            project_dir.to_str().unwrap(),
            "--no-git",
            "--python",
        ])
        .assert()
        .success();

    assert!(project_dir.join("figures/__init__.py").exists());
    assert!(project_dir.join("figures/example.py").exists());

    // nova.toml should contain [python] section
    let config = fs::read_to_string(project_dir.join("nova.toml")).unwrap();
    assert!(config.contains("[python]"));
}

#[test]
fn cli_init_existing_dir_fails() {
    let temp_dir = TempDir::new().unwrap();
    // temp_dir already exists, so init should fail
    nova()
        .args(["init", temp_dir.path().to_str().unwrap(), "--no-git"])
        .assert()
        .failure();
}

// ─── Help ───────────────────────────────────────────────────────────

#[test]
fn cli_help_shows_commands() {
    nova()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("compile"))
        .stdout(predicate::str::contains("init"));
}

#[test]
fn cli_compile_help_shows_options() {
    nova()
        .args(["compile", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--output"));
}
