//! Integration tests for the compilation workflow.

use nova_core::document::Document;
use nova_schema::FrontmatterParser;
use std::path::PathBuf;

/// Test that we can parse a document with frontmatter.
#[test]
fn parse_document_with_frontmatter() {
    let content = r#"---
title: "Integration Test Document"
authors:
  - name: Test Author
    email: test@example.com
date: "2024-01-15"
---

= Introduction

This is a test document for integration testing.

= Methods

We use automated tests to verify functionality.

= Results

All tests pass successfully.
"#;

    let parser = FrontmatterParser::new();
    let result: Result<(Option<serde_json::Value>, &str), _> = parser.parse(content);

    assert!(result.is_ok());
    let (metadata, remaining) = result.unwrap();

    // Check metadata was parsed
    assert!(metadata.is_some());
    let meta = metadata.unwrap();
    assert_eq!(meta["title"], "Integration Test Document");

    // Check remaining content
    assert!(remaining.contains("Introduction"));
    assert!(remaining.contains("Methods"));
}

/// Test document creation from content.
#[test]
fn create_document_from_content() {
    let content = "Hello, NovaType!";
    let doc = Document::from_content(content);

    assert_eq!(doc.content, content);
    assert!(doc.source_path.is_none());
    assert!(!doc.has_metadata());
}

/// Test document title accessor.
#[test]
fn document_title_from_metadata() {
    let mut doc = Document::new();
    assert!(doc.title().is_none());

    doc.metadata.title = Some("My Test Document".to_string());
    assert_eq!(doc.title(), Some("My Test Document"));
}

/// Test loading fixtures.
#[test]
fn load_fixture_document() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("simple.typ");

    // Skip if fixture doesn't exist
    if !fixture_path.exists() {
        eprintln!("Skipping test: fixture not found at {:?}", fixture_path);
        return;
    }

    let result = Document::from_file(&fixture_path);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert!(!doc.content.is_empty());
    assert_eq!(doc.source_path, Some(fixture_path));
}
