//! JSON Schema validation for document metadata.
//!
//! Provides validation of document metadata against predefined schemas.

use crate::error::{Error, Result};
use serde_json::Value;
use std::path::Path;
use tracing::debug;

/// A JSON Schema for validating document metadata.
#[derive(Debug)]
pub struct Schema {
    /// Schema name/identifier.
    name: String,

    /// The JSON Schema definition.
    schema: Value,

    /// Compiled validator.
    compiled: jsonschema::JSONSchema,
}

impl Clone for Schema {
    fn clone(&self) -> Self {
        // Re-compile for clone since JSONSchema doesn't implement Clone
        Self::new(self.name.clone(), self.schema.clone())
            .expect("Schema was valid before, should be valid now")
    }
}

impl Schema {
    /// Create a new schema from a JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if the schema is invalid.
    pub fn new(name: impl Into<String>, schema: Value) -> Result<Self> {
        let name = name.into();
        debug!(%name, "Creating schema");

        let compiled =
            jsonschema::JSONSchema::compile(&schema).map_err(|e| Error::InvalidSchema {
                message: e.to_string(),
            })?;

        Ok(Self {
            name,
            schema,
            compiled,
        })
    }

    /// Load a schema from a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        let schema: Value = serde_json::from_str(&content)?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self::new(name, schema)
    }

    /// Get the schema name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Validate a JSON value against this schema.
    ///
    /// # Returns
    ///
    /// A [`ValidationResult`] containing any errors found.
    pub fn validate(&self, value: &Value) -> ValidationResult {
        let mut errors = Vec::new();

        if let Err(validation_errors) = self.compiled.validate(value) {
            for error in validation_errors {
                errors.push(ValidationError {
                    message: error.to_string(),
                    path: error.instance_path.to_string(),
                    keyword: format!("{:?}", error.kind),
                });
            }
        }

        ValidationResult { errors }
    }

    /// Check if a value is valid against this schema.
    #[must_use]
    pub fn is_valid(&self, value: &Value) -> bool {
        self.compiled.is_valid(value)
    }

    /// Get the raw schema definition.
    #[must_use]
    pub fn definition(&self) -> &Value {
        &self.schema
    }
}

/// Result of schema validation.
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// List of validation errors.
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    /// Check if validation passed (no errors).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the number of errors.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Convert to a Result, failing if there are errors.
    ///
    /// # Errors
    ///
    /// Returns an error if validation failed.
    pub fn into_result(self) -> Result<()> {
        if self.is_valid() {
            Ok(())
        } else {
            let message = self
                .errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            Err(Error::ValidationFailed {
                message,
                path: self.errors.first().map(|e| e.path.clone()),
            })
        }
    }
}

/// A single validation error.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Human-readable error message.
    pub message: String,

    /// JSON Pointer path to the invalid field.
    pub path: String,

    /// JSON Schema keyword that failed.
    pub keyword: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "{}: {}", self.path, self.message)
        }
    }
}

/// Create a basic article metadata schema.
#[must_use]
pub fn article_schema() -> Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "minLength": 1
            },
            "authors": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "email": { "type": "string", "format": "email" },
                        "affiliation": { "type": "string" },
                        "orcid": { "type": "string", "pattern": "^\\d{4}-\\d{4}-\\d{4}-\\d{3}[\\dX]$" }
                    },
                    "required": ["name"]
                }
            },
            "date": { "type": "string" },
            "abstract": { "type": "string" },
            "keywords": {
                "type": "array",
                "items": { "type": "string" }
            },
            "template": { "type": "string" },
            "citation_style": { "type": "string" },
            "bibliography": {
                "type": "array",
                "items": { "type": "string" }
            },
            "fonts": {
                "type": "object",
                "description": "Font configuration for the document",
                "properties": {
                    "main": { "$ref": "#/$defs/fontSpec" },
                    "heading": { "$ref": "#/$defs/fontSpec" },
                    "mono": { "$ref": "#/$defs/fontSpec" },
                    "math": { "$ref": "#/$defs/fontSpec" }
                }
            }
        },
        "$defs": {
            "fontSpec": {
                "oneOf": [
                    { "type": "string", "description": "Font family name only" },
                    {
                        "type": "object",
                        "description": "Full font specification",
                        "properties": {
                            "family": { "type": "string", "description": "Font family name" },
                            "size": { "type": "string", "description": "Font size (e.g., 11pt, 1.2em)" },
                            "weight": { "type": "string", "description": "Font weight (e.g., bold, 600)" },
                            "style": { "type": "string", "enum": ["normal", "italic", "oblique"] }
                        },
                        "required": ["family"]
                    }
                ]
            }
        },
        "required": ["title"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_validate_valid_document() {
        let schema = Schema::new("test", article_schema()).unwrap();

        let valid_doc = serde_json::json!({
            "title": "My Document",
            "authors": [{ "name": "John Doe" }]
        });

        let result = schema.validate(&valid_doc);
        assert!(result.is_valid());
    }

    #[test]
    fn schema_validate_missing_required() {
        let schema = Schema::new("test", article_schema()).unwrap();

        let invalid_doc = serde_json::json!({
            "authors": [{ "name": "John Doe" }]
        });

        let result = schema.validate(&invalid_doc);
        assert!(!result.is_valid());
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn schema_validate_invalid_type() {
        let schema = Schema::new("test", article_schema()).unwrap();

        let invalid_doc = serde_json::json!({
            "title": 123  // Should be string
        });

        let result = schema.validate(&invalid_doc);
        assert!(!result.is_valid());
    }

    #[test]
    fn schema_is_valid_shortcut() {
        let schema = Schema::new("test", article_schema()).unwrap();

        let valid = serde_json::json!({ "title": "Test" });
        let invalid = serde_json::json!({ "title": "" });

        assert!(schema.is_valid(&valid));
        assert!(!schema.is_valid(&invalid));
    }

    #[test]
    fn validation_result_into_result() {
        let valid_result = ValidationResult::default();
        assert!(valid_result.into_result().is_ok());

        let invalid_result = ValidationResult {
            errors: vec![ValidationError {
                message: "test error".to_string(),
                path: "/title".to_string(),
                keyword: "required".to_string(),
            }],
        };
        assert!(invalid_result.into_result().is_err());
    }

    #[test]
    fn validation_error_display() {
        let error = ValidationError {
            message: "is required".to_string(),
            path: "/title".to_string(),
            keyword: "required".to_string(),
        };
        assert_eq!(error.to_string(), "/title: is required");

        let error_no_path = ValidationError {
            message: "invalid type".to_string(),
            path: String::new(),
            keyword: "type".to_string(),
        };
        assert_eq!(error_no_path.to_string(), "invalid type");
    }

    #[test]
    fn schema_validate_fonts_simple() {
        let schema = Schema::new("test", article_schema()).unwrap();

        let doc_with_fonts = serde_json::json!({
            "title": "My Document",
            "fonts": {
                "main": "Inter",
                "mono": "JetBrains Mono",
                "heading": "Open Sans"
            }
        });

        let result = schema.validate(&doc_with_fonts);
        assert!(result.is_valid(), "Simple fonts should be valid: {:?}", result.errors);
    }

    #[test]
    fn schema_validate_fonts_full() {
        let schema = Schema::new("test", article_schema()).unwrap();

        let doc_with_fonts = serde_json::json!({
            "title": "My Document",
            "fonts": {
                "main": {
                    "family": "Inter",
                    "size": "11pt"
                },
                "heading": {
                    "family": "Open Sans",
                    "size": "14pt",
                    "weight": "bold"
                },
                "mono": {
                    "family": "JetBrains Mono",
                    "size": "9pt",
                    "style": "normal"
                }
            }
        });

        let result = schema.validate(&doc_with_fonts);
        assert!(result.is_valid(), "Full fonts should be valid: {:?}", result.errors);
    }

    #[test]
    fn schema_validate_fonts_invalid_type() {
        let schema = Schema::new("test", article_schema()).unwrap();

        let doc_with_invalid_fonts = serde_json::json!({
            "title": "My Document",
            "fonts": {
                "main": 123  // Should be string or object
            }
        });

        let result = schema.validate(&doc_with_invalid_fonts);
        assert!(!result.is_valid(), "Fonts with invalid type should fail");
    }
}
