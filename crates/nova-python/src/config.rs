//! Configuration for Python integration.
//!
//! The [`PythonConfig`] struct is defined in `novatype-core` and re-exported here.
//! This module provides helper functions for loading the Python-specific configuration.

use crate::error::{Error, Result};
use std::path::Path;

// Re-export PythonConfig from nova-core (canonical definition).
pub use novatype_core::config::PythonConfig;

/// Load Python configuration from a project directory.
///
/// Reads `nova.toml` and extracts the `[python]` section.
/// Returns an error if nova.toml is missing or has no `[python]` section.
pub fn load_python_config(project_dir: impl AsRef<Path>) -> Result<PythonConfig> {
    let project_dir = project_dir.as_ref();

    let config = novatype_core::NovaConfig::from_project(project_dir).map_err(|e| match e {
        novatype_core::ConfigError::NotFound { path } => Error::ConfigNotFound { path },
        other => Error::InvalidConfig {
            message: other.to_string(),
        },
    })?;

    config.python.ok_or_else(|| Error::InvalidConfig {
        message: "No [python] section in nova.toml".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = PythonConfig::default();
        assert_eq!(config.python, "python");
        assert_eq!(config.figures_dir, PathBuf::from("figures"));
        assert_eq!(config.timeout, 60);
    }

    #[test]
    fn test_load_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("nova.toml");
        let mut file = std::fs::File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"
[python]
python = "python3"
figures_dir = "src/figures"
timeout = 120
"#
        )
        .unwrap();

        let config = load_python_config(dir.path()).unwrap();
        assert_eq!(config.python, "python3");
        assert_eq!(config.figures_dir, dir.path().join("src/figures"));
        assert_eq!(config.timeout, 120);
    }

    #[test]
    fn test_config_not_found() {
        let dir = TempDir::new().unwrap();
        let result = load_python_config(dir.path());
        assert!(matches!(result, Err(Error::ConfigNotFound { .. })));
    }

    #[test]
    fn test_no_python_section() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("nova.toml"), "[project]\nname = \"test\"\n").unwrap();

        let result = load_python_config(dir.path());
        assert!(matches!(result, Err(Error::InvalidConfig { .. })));
    }
}
