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
    ScriptError {
        file: PathBuf,
        error: String,
        traceback: Option<Box<PythonTraceback>>,
    },

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

// ─── Python traceback parsing ───────────────────────────────────────

/// A single frame from a Python traceback.
#[derive(Debug, Clone)]
pub struct PythonFrame {
    /// Source file path.
    pub file: String,
    /// Line number (1-indexed).
    pub line: usize,
    /// Function name.
    pub function: String,
    /// Source code line (if available).
    pub code: Option<String>,
}

/// A parsed Python traceback.
#[derive(Debug, Clone)]
pub struct PythonTraceback {
    /// Stack frames (outermost first).
    pub frames: Vec<PythonFrame>,
    /// Exception type (e.g., "NameError").
    pub exception_type: String,
    /// Exception message (e.g., "name 'GRIS' is not defined").
    pub exception_message: String,
}

impl PythonTraceback {
    /// Try to parse a Python traceback from stderr output.
    ///
    /// Returns `None` if the text does not look like a traceback.
    pub fn parse(stderr: &str) -> Option<Self> {
        if !stderr.contains("Traceback (most recent call last):") {
            return None;
        }

        let mut frames = Vec::new();
        let mut lines = stderr.lines().peekable();

        // Skip to the "Traceback" line
        for line in lines.by_ref() {
            if line
                .trim_start()
                .starts_with("Traceback (most recent call last):")
            {
                break;
            }
        }

        // Parse frames: pairs of "  File "...", line N, in func" + optional code line
        while let Some(line) = lines.peek() {
            let trimmed = line.trim();
            if trimmed.starts_with("File ") {
                let frame_line = lines.next().unwrap().trim();
                if let Some(frame) = Self::parse_frame_line(frame_line) {
                    // Next line might be the code line (indented, not a File line)
                    let code = if let Some(next) = lines.peek() {
                        let next_trimmed = next.trim();
                        if !next_trimmed.starts_with("File ") && !next_trimmed.is_empty() {
                            Some(lines.next().unwrap().trim().to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    frames.push(PythonFrame {
                        file: frame.0,
                        line: frame.1,
                        function: frame.2,
                        code,
                    });
                } else {
                    lines.next();
                }
            } else {
                break;
            }
        }

        // Remaining lines: "ExceptionType: message" (possibly multi-line)
        let remaining: Vec<&str> = lines.collect();
        let error_line = remaining.first().unwrap_or(&"").trim();

        let (exception_type, exception_message) =
            if let Some((etype, emsg)) = error_line.split_once(": ") {
                (etype.to_string(), emsg.to_string())
            } else {
                (error_line.to_string(), String::new())
            };

        Some(PythonTraceback {
            frames,
            exception_type,
            exception_message,
        })
    }

    /// Parse a frame line like: File "figures/plots.py", line 15, in plot_data
    fn parse_frame_line(line: &str) -> Option<(String, usize, String)> {
        // File "path", line N, in func
        let rest = line.strip_prefix("File ")?;

        // Extract quoted path
        let (path, rest) = if let Some(after_quote) = rest.strip_prefix('"') {
            let end = after_quote.find('"')?;
            (&after_quote[..end], &after_quote[end + 1..])
        } else {
            return None;
        };

        // ", line N, in func"
        let rest = rest.strip_prefix(", line ")?;
        let (line_str, rest) = rest.split_once(", in ")?;
        let line_num: usize = line_str.parse().ok()?;

        Some((path.to_string(), line_num, rest.to_string()))
    }
}

impl std::fmt::Display for PythonTraceback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.exception_type)?;
        if !self.exception_message.is_empty() {
            write!(f, ": {}", self.exception_message)?;
        }
        if let Some(frame) = self.frames.last() {
            write!(f, " ({}:{})", frame.file, frame.line)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_traceback() {
        let stderr = r#"Traceback (most recent call last):
  File "figures/plots.py", line 15, in plot_data
    result = compute(data)
  File "figures/plots.py", line 8, in compute
    return data / 0
ZeroDivisionError: division by zero
"#;
        let tb = PythonTraceback::parse(stderr).unwrap();
        assert_eq!(tb.exception_type, "ZeroDivisionError");
        assert_eq!(tb.exception_message, "division by zero");
        assert_eq!(tb.frames.len(), 2);
        assert_eq!(tb.frames[0].file, "figures/plots.py");
        assert_eq!(tb.frames[0].line, 15);
        assert_eq!(tb.frames[0].function, "plot_data");
        assert_eq!(tb.frames[0].code.as_deref(), Some("result = compute(data)"));
        assert_eq!(tb.frames[1].line, 8);
    }

    #[test]
    fn parse_name_error() {
        let stderr = r#"Traceback (most recent call last):
  File "figures/indicateurs.py", line 107, in qualite_service
    color = GRIS
NameError: name 'GRIS' is not defined
"#;
        let tb = PythonTraceback::parse(stderr).unwrap();
        assert_eq!(tb.exception_type, "NameError");
        assert_eq!(tb.exception_message, "name 'GRIS' is not defined");
        assert_eq!(tb.frames.len(), 1);
        assert_eq!(tb.frames[0].line, 107);
    }

    #[test]
    fn parse_no_traceback() {
        let stderr = "some random error output\n";
        assert!(PythonTraceback::parse(stderr).is_none());
    }

    #[test]
    fn traceback_display() {
        let tb = PythonTraceback {
            frames: vec![PythonFrame {
                file: "figures/plots.py".to_string(),
                line: 42,
                function: "plot_data".to_string(),
                code: None,
            }],
            exception_type: "ValueError".to_string(),
            exception_message: "invalid input".to_string(),
        };
        assert_eq!(
            tb.to_string(),
            "ValueError: invalid input (figures/plots.py:42)"
        );
    }
}
