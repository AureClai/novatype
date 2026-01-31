//! Error types for nova-plot.

use thiserror::Error;

/// Result type alias for plot operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during plot operations.
#[derive(Debug, Error)]
pub enum Error {
    /// No data provided.
    #[error("no data provided for chart")]
    NoData,

    /// Invalid data format.
    #[error("invalid data format: {message}")]
    InvalidData {
        /// Error message.
        message: String,
    },

    /// Missing required column.
    #[error("missing column: {column}")]
    MissingColumn {
        /// Column name.
        column: String,
    },

    /// Invalid configuration.
    #[error("invalid configuration: {message}")]
    InvalidConfig {
        /// Error message.
        message: String,
    },

    /// Rendering error.
    #[error("rendering failed: {message}")]
    RenderError {
        /// Error message.
        message: String,
    },

    /// CSV parsing error.
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    /// JSON parsing error.
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
    fn error_display() {
        let err = Error::MissingColumn {
            column: "value".to_string(),
        };
        assert_eq!(err.to_string(), "missing column: value");
    }

    #[test]
    fn error_no_data() {
        let err = Error::NoData;
        assert_eq!(err.to_string(), "no data provided for chart");
    }
}
