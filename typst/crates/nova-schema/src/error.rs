//! Error types for nova-schema.

use thiserror::Error;

/// Result type alias for schema operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during schema operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Schema file not found.
    #[error("schema not found: {name}")]
    SchemaNotFound {
        /// Name of the missing schema.
        name: String,
    },

    /// Schema is invalid JSON Schema.
    #[error("invalid schema definition: {message}")]
    InvalidSchema {
        /// Error message.
        message: String,
    },

    /// Validation failed.
    #[error("validation failed: {message}")]
    ValidationFailed {
        /// Error message.
        message: String,
        /// Path to the invalid field.
        path: Option<String>,
    },

    /// Frontmatter parsing error.
    #[error("frontmatter parse error: {message}")]
    FrontmatterParse {
        /// Error message.
        message: String,
        /// Line number if available.
        line: Option<usize>,
    },

    /// YAML parsing error.
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// TOML parsing error.
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_schema_not_found() {
        let err = Error::SchemaNotFound {
            name: "ieee-article".to_string(),
        };
        assert_eq!(err.to_string(), "schema not found: ieee-article");
    }

    #[test]
    fn error_display_validation_failed() {
        let err = Error::ValidationFailed {
            message: "missing required field".to_string(),
            path: Some("/title".to_string()),
        };
        assert_eq!(err.to_string(), "validation failed: missing required field");
    }
}
