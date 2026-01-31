//! Integration tests for schema validation.

use nova_schema::{article_schema, FrontmatterParser, Schema};
use serde_json::json;

/// Test schema validation with valid article metadata.
#[test]
fn validate_valid_article_metadata() {
    let schema = Schema::new("article", article_schema()).unwrap();

    let valid_metadata = json!({
        "title": "A Valid Article Title",
        "authors": [
            {
                "name": "John Doe",
                "email": "john@example.com",
                "affiliation": "Test University"
            }
        ],
        "abstract": "This is the abstract of the article.",
        "keywords": ["testing", "validation", "novatype"]
    });

    let result = schema.validate(&valid_metadata);
    assert!(result.is_valid(), "Expected valid, got errors: {:?}", result.errors);
}

/// Test schema validation with missing required field.
#[test]
fn validate_missing_title() {
    let schema = Schema::new("article", article_schema()).unwrap();

    let invalid_metadata = json!({
        "authors": [{"name": "John Doe"}]
    });

    let result = schema.validate(&invalid_metadata);
    assert!(!result.is_valid());
    assert!(!result.errors.is_empty());
}

/// Test schema validation with invalid type.
#[test]
fn validate_invalid_type() {
    let schema = Schema::new("article", article_schema()).unwrap();

    let invalid_metadata = json!({
        "title": 12345  // Should be string
    });

    let result = schema.validate(&invalid_metadata);
    assert!(!result.is_valid());
}

/// Test frontmatter parsing and validation together.
#[test]
fn parse_and_validate_frontmatter() {
    let content = r#"---
title: "Parsed and Validated"
authors:
  - name: Jane Smith
---
Content here
"#;

    // Parse frontmatter
    let parser = FrontmatterParser::new();
    let (metadata, _): (Option<serde_json::Value>, _) = parser.parse(content).unwrap();
    let metadata = metadata.expect("Should have metadata");

    // Validate
    let schema = Schema::new("article", article_schema()).unwrap();
    let result = schema.validate(&metadata);

    assert!(result.is_valid());
}

/// Test validation result conversion.
#[test]
fn validation_result_into_result() {
    let schema = Schema::new("article", article_schema()).unwrap();

    // Valid case
    let valid = json!({"title": "Test"});
    let result = schema.validate(&valid);
    assert!(result.into_result().is_ok());

    // Invalid case
    let invalid = json!({});
    let result = schema.validate(&invalid);
    assert!(result.into_result().is_err());
}

/// Test ORCID pattern validation.
#[test]
fn validate_orcid_pattern() {
    let schema = Schema::new("article", article_schema()).unwrap();

    // Valid ORCID
    let valid = json!({
        "title": "Test",
        "authors": [{
            "name": "Test Author",
            "orcid": "0000-0002-1825-0097"
        }]
    });
    assert!(schema.is_valid(&valid));

    // Invalid ORCID format
    let invalid = json!({
        "title": "Test",
        "authors": [{
            "name": "Test Author",
            "orcid": "not-an-orcid"
        }]
    });
    assert!(!schema.is_valid(&invalid));
}
