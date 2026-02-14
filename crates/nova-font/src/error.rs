//! Error types for nova-font.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for nova-font operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in nova-font.
#[derive(Debug, Error)]
pub enum Error {
    /// Font not found.
    #[error("font not found: {name}")]
    FontNotFound { name: String },

    /// Font already installed.
    #[error("font already installed: {name}")]
    AlreadyInstalled { name: String },

    /// API error.
    #[error("API error: {message}")]
    Api {
        message: String,
        #[source]
        source: Option<reqwest::Error>,
    },

    /// Rate limited by API.
    #[error("rate limited, retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },

    /// Network error.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Cache error.
    #[error("cache error: {message}")]
    Cache { message: String },

    /// Invalid font file.
    #[error("invalid font file: {path}")]
    InvalidFontFile { path: PathBuf },

    /// Download failed.
    #[error("download failed for {font_name}: {reason}")]
    DownloadFailed { font_name: String, reason: String },

    /// Zip extraction error.
    #[error("zip extraction error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// JSON parsing error.
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// No API key configured.
    #[error("Google Fonts API key not configured. Set GOOGLE_FONTS_API_KEY environment variable.")]
    NoApiKey,

    /// Invalid configuration.
    #[error("invalid configuration: {message}")]
    InvalidConfig { message: String },
}
