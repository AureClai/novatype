//! Error types for nova-python.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for nova-python operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during Python figure generation.
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration file not found.
    #[error("nova.toml not found in {path}")]
    ConfigNotFound { path: PathBuf },

    /// Invalid configuration.
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    /// Python executable not found.
    #[error("Python executable not found: {path}")]
    PythonNotFound { path: String },

    /// Nova Python package not installed.
    #[error("Nova Python package not installed. Run: pip install nova-typst")]
    NovaPackageNotInstalled,

    /// Figure not found.
    #[error("Figure '{name}' not found in Python sources")]
    FigureNotFound { name: String },

    /// Python execution failed.
    #[error("Python execution failed: {message}")]
    ExecutionFailed { message: String },

    /// Python script error.
    #[error("Python script error in {file}: {error}")]
    ScriptError { file: PathBuf, error: String },

    /// Figure generation failed.
    #[error("Failed to generate figure '{name}': {reason}")]
    GenerationFailed { name: String, reason: String },

    /// Cache error.
    #[error("Cache error: {message}")]
    CacheError { message: String },

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error.
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Regex error.
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}
