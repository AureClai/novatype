//! Error types for nova-cite.

use thiserror::Error;

/// Result type alias for citation operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during citation operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Citation not found in any source.
    #[error("citation not found: {key}")]
    NotFound {
        /// The citation key that was not found.
        key: String,
    },

    /// Invalid citation key format.
    #[error("invalid citation key: {key}")]
    InvalidKey {
        /// The invalid key.
        key: String,
    },

    /// BibTeX parsing error.
    #[error("BibTeX parse error: {message}")]
    BibtexParse {
        /// Error message.
        message: String,
        /// Line number if available.
        line: Option<usize>,
    },

    /// Citation style not supported.
    #[error("unsupported citation style: {style}")]
    UnsupportedStyle {
        /// The unsupported style name.
        style: String,
    },

    /// API request failed.
    #[error("API error: {message}")]
    Api {
        /// Error message.
        message: String,
        /// HTTP status code if available.
        status_code: Option<u16>,
    },

    /// Rate limit exceeded.
    #[error("rate limit exceeded, retry after {retry_after_secs} seconds")]
    RateLimited {
        /// Seconds to wait before retry.
        retry_after_secs: u64,
    },

    /// Network error.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_not_found() {
        let err = Error::NotFound {
            key: "smith2023".to_string(),
        };
        assert_eq!(err.to_string(), "citation not found: smith2023");
    }

    #[test]
    fn error_display_rate_limited() {
        let err = Error::RateLimited {
            retry_after_secs: 60,
        };
        assert_eq!(
            err.to_string(),
            "rate limit exceeded, retry after 60 seconds"
        );
    }

    #[test]
    fn error_display_api() {
        let err = Error::Api {
            message: "server error".to_string(),
            status_code: Some(500),
        };
        assert_eq!(err.to_string(), "API error: server error");
    }
}
