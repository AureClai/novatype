//! Error types for nova-template.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for nova-template operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in nova-template.
#[derive(Debug, Error)]
pub enum Error {
    /// Template not found.
    #[error("Template not found: {name}")]
    TemplateNotFound { name: String },

    /// Template already exists.
    #[error("Template already installed: {name}")]
    TemplateAlreadyExists { name: String },

    /// Invalid template format.
    #[error("Invalid template: {message}")]
    InvalidTemplate { message: String },

    /// Missing required file in template.
    #[error("Missing required file in template: {file}")]
    MissingFile { file: String },

    /// Cache error.
    #[error("Cache error: {message}")]
    Cache { message: String },

    /// Network error.
    #[error("Network error: {message}")]
    Network { message: String },

    /// Source error (GitHub, marketplace, etc.).
    #[error("Source error: {source_name} - {message}")]
    Source { source_name: String, message: String },

    /// Invalid source URL.
    #[error("Invalid source URL: {url}")]
    InvalidSourceUrl { url: String },

    /// IO error.
    #[error("IO error at {path:?}: {source}")]
    Io {
        path: Option<PathBuf>,
        #[source]
        source: std::io::Error,
    },

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// TOML parsing error.
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// ZIP extraction error.
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io {
            path: None,
            source: e,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Network {
            message: e.to_string(),
        }
    }
}
