//! Integration tests for the Typst compilation pipeline.
//!
//! These tests use `NativeWorld` and `compile_pdf`/`compile_svg` directly
//! to verify that actual Typst compilation works end-to-end.

use novatype_core::{compile_pdf, compile_svg, NativeWorld};
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

// ─── PDF compilation ────────────────────────────────────────────────

#[test]
fn compile_pdf_hello_world() {
    let world = NativeWorld::from_source("Hello, World!", ".");
    let result = compile_pdf(&world);

    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    let pdf = result.unwrap();
    assert!(!pdf.is_empty());
    assert_eq!(&pdf[0..5], b"%PDF-");
}

#[test]
fn compile_pdf_with_headings() {
    let source = r#"
= Chapter 1

Introduction paragraph.

== Section 1.1

First section content.

== Section 1.2

Second section content.

= Chapter 2

Another chapter.
"#;

    let world = NativeWorld::from_source(source, ".");
    let result = compile_pdf(&world);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
}

#[test]
fn compile_pdf_with_math() {
    let source = r#"
The equation $E = m c^2$ is famous.

$ integral_0^infinity e^(-x^2) dif x = sqrt(pi) / 2 $
"#;

    let world = NativeWorld::from_source(source, ".");
    let result = compile_pdf(&world);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
}

#[test]
fn compile_pdf_with_table() {
    let source = r#"
#table(
  columns: (auto, auto, auto),
  [Name], [Age], [City],
  [Alice], [30], [Paris],
  [Bob], [25], [London],
)
"#;

    let world = NativeWorld::from_source(source, ".");
    let result = compile_pdf(&world);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
}

#[test]
fn compile_pdf_with_styling() {
    let source = r#"
#set text(size: 12pt)
#set par(justify: true)

= Styled Document

This document has *bold*, _italic_, and `code` styling.

#lorem(100)
"#;

    let world = NativeWorld::from_source(source, ".");
    let result = compile_pdf(&world);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
}

// ─── SVG compilation ────────────────────────────────────────────────

#[test]
fn compile_svg_hello_world() {
    let world = NativeWorld::from_source("Hello, SVG!", ".");
    let result = compile_svg(&world);

    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    let pages = result.unwrap();
    assert!(!pages.is_empty());
    assert!(pages[0].contains("<svg"));
}

#[test]
fn compile_svg_multipage() {
    let source = r#"
Page 1 content.
#pagebreak()
Page 2 content.
#pagebreak()
Page 3 content.
"#;

    let world = NativeWorld::from_source(source, ".");
    let result = compile_svg(&world);

    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    let pages = result.unwrap();
    assert_eq!(pages.len(), 3, "Should produce 3 SVG pages");
    for page in &pages {
        assert!(page.contains("<svg"));
    }
}

// ─── Fixture-based compilation ──────────────────────────────────────

#[test]
fn compile_fixture_minimal() {
    let path = fixture_path("minimal.typ");
    if !path.exists() {
        eprintln!("Skipping: fixture not found at {:?}", path);
        return;
    }

    let world = NativeWorld::new(&path).unwrap();
    let result = compile_pdf(&world);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
}

#[test]
fn compile_fixture_with_math() {
    let path = fixture_path("with-math.typ");
    if !path.exists() {
        eprintln!("Skipping: fixture not found at {:?}", path);
        return;
    }

    let world = NativeWorld::new(&path).unwrap();
    let result = compile_pdf(&world);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
}

#[test]
fn compile_fixture_with_headings() {
    let path = fixture_path("with-headings.typ");
    if !path.exists() {
        eprintln!("Skipping: fixture not found at {:?}", path);
        return;
    }

    let world = NativeWorld::new(&path).unwrap();
    let result = compile_pdf(&world);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
}

#[test]
fn compile_fixture_multipage() {
    let path = fixture_path("multipage.typ");
    if !path.exists() {
        eprintln!("Skipping: fixture not found at {:?}", path);
        return;
    }

    let world = NativeWorld::new(&path).unwrap();
    let result = compile_svg(&world);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());

    let pages = result.unwrap();
    assert!(pages.len() >= 3, "Should produce at least 3 pages");
}

// ─── Error cases ────────────────────────────────────────────────────

#[test]
fn compile_invalid_typst_returns_error() {
    let world = NativeWorld::from_source("#this-is-not-valid-typst((", ".");
    let result = compile_pdf(&world);

    assert!(result.is_err(), "Invalid Typst should produce an error");
    let errors = result.unwrap_err();
    assert!(!errors.is_empty(), "Error messages should not be empty");
}

#[test]
fn compile_empty_document_succeeds() {
    let world = NativeWorld::from_source("", ".");
    let result = compile_pdf(&world);
    // An empty document should still produce a valid (empty) PDF
    assert!(result.is_ok(), "Empty document failed: {:?}", result.err());
}
