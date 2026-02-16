//! Error types for nova-core.
//!
//! This module defines the error types used throughout the core crate,
//! following the principle of explicit error handling.

use thiserror::Error;

/// Result type alias using [`Error`] as the error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during document compilation.
#[derive(Debug, Error)]
pub enum Error {
    /// The source file could not be found.
    #[error("source file not found: {path}")]
    SourceNotFound {
        /// Path to the missing file.
        path: String,
    },

    /// The document metadata is invalid.
    #[error("invalid metadata: {message}")]
    InvalidMetadata {
        /// Description of the validation error.
        message: String,
    },

    /// Schema validation failed.
    #[error("schema validation failed: {0}")]
    SchemaValidation(#[from] nova_schema::Error),

    /// The Typst compilation failed.
    #[error("typst compilation error: {message}")]
    TypstCompilation {
        /// Error message from Typst.
        message: String,
        /// Optional source location.
        location: Option<SourceLocation>,
    },

    /// A citation could not be resolved.
    #[cfg(feature = "citations")]
    #[error("citation error: {0}")]
    Citation(#[from] nova_cite::Error),

    /// Data visualization failed.
    #[cfg(feature = "plots")]
    #[error("plot error: {0}")]
    Plot(#[from] nova_plot::Error),

    /// An I/O operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Represents a location in the source document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// Line number (1-indexed).
    pub line: usize,
    /// Column number (1-indexed).
    pub column: usize,
    /// Optional file path.
    pub file: Option<String>,
}

// ─── Rich diagnostic types ──────────────────────────────────────────

/// Severity level for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

/// A resolved source location with text context for rendering.
#[derive(Debug, Clone)]
pub struct DiagnosticLocation {
    /// File path (relative to project root).
    pub file: String,
    /// Full source text of the file (needed for snippet rendering).
    pub source_text: String,
    /// Byte offset of the error span start within source_text.
    pub span_offset: usize,
    /// Byte length of the error span.
    pub span_length: usize,
    /// Line number (1-indexed).
    pub line: usize,
    /// Column number (1-indexed).
    pub column: usize,
}

/// A fully resolved diagnostic with source context.
///
/// Unlike Typst's `SourceDiagnostic` which uses opaque `Span` types,
/// this carries all the resolved information needed for rendering.
#[derive(Debug, Clone)]
pub struct NovaDiagnostic {
    /// Error or warning.
    pub severity: DiagnosticSeverity,
    /// The diagnostic message.
    pub message: String,
    /// Source location (file, line, column, byte range, source text).
    /// None if the span was detached.
    pub location: Option<DiagnosticLocation>,
    /// Hints from Typst (e.g., "did you mean...").
    pub hints: Vec<String>,
}

impl std::fmt::Display for NovaDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref loc) = self.location {
            write!(
                f,
                "{}:{}:{}: {}",
                loc.file, loc.line, loc.column, self.message
            )
        } else {
            write!(f, "{}", self.message)
        }
    }
}

/// Result of a compilation, carrying both output and warnings.
#[derive(Debug)]
pub struct CompilationResult<T> {
    /// The compilation output (if successful) or structured errors.
    pub output: std::result::Result<T, Vec<NovaDiagnostic>>,
    /// Warnings produced during compilation (always present, even on success).
    pub warnings: Vec<NovaDiagnostic>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_source_not_found() {
        let err = Error::SourceNotFound {
            path: "test.typ".to_string(),
        };
        assert_eq!(err.to_string(), "source file not found: test.typ");
    }

    #[test]
    fn error_display_invalid_metadata() {
        let err = Error::InvalidMetadata {
            message: "missing title field".to_string(),
        };
        assert_eq!(err.to_string(), "invalid metadata: missing title field");
    }

    #[test]
    fn source_location_equality() {
        let loc1 = SourceLocation {
            line: 10,
            column: 5,
            file: Some("test.typ".to_string()),
        };
        let loc2 = SourceLocation {
            line: 10,
            column: 5,
            file: Some("test.typ".to_string()),
        };
        assert_eq!(loc1, loc2);
    }
}
