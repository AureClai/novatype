//! Integration tests for data injection from nova.toml [data] section.
//!
//! These tests verify the full pipeline: data files → Typst `#let` bindings → compilation.

use novatype_core::{compile_pdf, load_data_files, NativeWorld};
use std::collections::HashMap;
use std::path::PathBuf;

// ─── JSON injection ─────────────────────────────────────────────────

#[test]
fn json_data_compiles_in_typst() {
    let dir = tempfile::TempDir::new().unwrap();
    let json_path = dir.path().join("budget.json");
    std::fs::write(
        &json_path,
        r#"{"total": 15000000, "label": "Budget annuel"}"#,
    )
    .unwrap();

    let mut data = HashMap::new();
    data.insert("budget".to_string(), json_path);

    let data_code = load_data_files(&data, dir.path()).unwrap();

    // Build Typst source that uses the injected variable
    let source = format!(
        r#"{}
Le budget total est de #budget.total €.

Label: #budget.label
"#,
        data_code
    );

    let world = NativeWorld::from_source(&source, ".");
    let result = compile_pdf(&world);
    assert!(
        result.is_ok(),
        "JSON data injection failed: {:?}",
        result.err()
    );
}

// ─── CSV injection ──────────────────────────────────────────────────

#[test]
fn csv_data_compiles_with_for_loop() {
    let dir = tempfile::TempDir::new().unwrap();
    let csv_path = dir.path().join("people.csv");
    std::fs::write(&csv_path, "name,age,score\nAlice,30,95.5\nBob,25,88.0\n").unwrap();

    let mut data = HashMap::new();
    data.insert("people".to_string(), csv_path);

    let data_code = load_data_files(&data, dir.path()).unwrap();

    // Use the array-of-dicts format in a for loop
    let source = format!(
        r#"{}
#for person in people [
  #person.name is #person.age years old with score #person.score.

]
"#,
        data_code
    );

    let world = NativeWorld::from_source(&source, ".");
    let result = compile_pdf(&world);
    assert!(
        result.is_ok(),
        "CSV data injection with for loop failed: {:?}",
        result.err()
    );
}

// ─── TOML injection ─────────────────────────────────────────────────

#[test]
fn toml_data_compiles_in_typst() {
    let dir = tempfile::TempDir::new().unwrap();
    let toml_path = dir.path().join("meta.toml");
    std::fs::write(
        &toml_path,
        r#"
version = "2.0"
year = 2025
active = true
"#,
    )
    .unwrap();

    let mut data = HashMap::new();
    data.insert("meta".to_string(), toml_path);

    let data_code = load_data_files(&data, dir.path()).unwrap();

    let source = format!(
        r#"{}
Version: #meta.version

Year: #meta.year
"#,
        data_code
    );

    let world = NativeWorld::from_source(&source, ".");
    let result = compile_pdf(&world);
    assert!(
        result.is_ok(),
        "TOML data injection failed: {:?}",
        result.err()
    );
}

// ─── Multiple data files ────────────────────────────────────────────

#[test]
fn multiple_data_files_all_accessible() {
    let dir = tempfile::TempDir::new().unwrap();

    let json_path = dir.path().join("config.json");
    std::fs::write(&json_path, r#"{"title": "Report"}"#).unwrap();

    let csv_path = dir.path().join("rows.csv");
    std::fs::write(&csv_path, "x,y\n1,2\n3,4\n").unwrap();

    let mut data = HashMap::new();
    data.insert("config".to_string(), json_path);
    data.insert("rows".to_string(), csv_path);

    let data_code = load_data_files(&data, dir.path()).unwrap();

    let source = format!(
        r#"{}
Title: #config.title

#for row in rows [
  Point: #row.x, #row.y

]
"#,
        data_code
    );

    let world = NativeWorld::from_source(&source, ".");
    let result = compile_pdf(&world);
    assert!(
        result.is_ok(),
        "Multiple data files failed: {:?}",
        result.err()
    );
}

// ─── Error cases ────────────────────────────────────────────────────

#[test]
fn missing_data_file_returns_error() {
    let mut data = HashMap::new();
    data.insert(
        "missing".to_string(),
        PathBuf::from("/nonexistent/data.json"),
    );

    let result = load_data_files(&data, std::path::Path::new("."));
    assert!(result.is_err(), "Missing data file should produce an error");
}

#[test]
fn unsupported_format_returns_error() {
    let dir = tempfile::TempDir::new().unwrap();
    let txt_path = dir.path().join("data.txt");
    std::fs::write(&txt_path, "hello world").unwrap();

    let mut data = HashMap::new();
    data.insert("stuff".to_string(), txt_path);

    let result = load_data_files(&data, dir.path());
    assert!(
        result.is_err(),
        "Unsupported format should produce an error"
    );
}
