//! Configuration for Python integration.
//!
//! Configuration is read from `nova.toml` in the project root.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Python integration configuration.
///
/// Loaded from the `[python]` section of `nova.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    /// Path to the Python executable.
    /// Defaults to "python" (uses PATH).
    #[serde(default = "default_python")]
    pub python: String,

    /// Directory containing Python figure scripts.
    /// Defaults to "figures".
    #[serde(default = "default_figures_dir")]
    pub figures_dir: PathBuf,

    /// Directory for cached SVG outputs.
    /// Defaults to ".nova/cache".
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,

    /// Virtual environment path (optional).
    /// If set, activates this venv before running Python.
    pub venv: Option<PathBuf>,

    /// Additional Python paths to add to PYTHONPATH.
    #[serde(default)]
    pub python_path: Vec<PathBuf>,

    /// Environment variables to set for Python execution.
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,

    /// Timeout for figure generation in seconds.
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Project root directory (set automatically).
    #[serde(skip)]
    pub project_root: PathBuf,
}

fn default_python() -> String {
    "python".to_string()
}

fn default_figures_dir() -> PathBuf {
    PathBuf::from("figures")
}

fn default_cache_dir() -> PathBuf {
    PathBuf::from(".nova/cache")
}

fn default_timeout() -> u64 {
    60
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            python: default_python(),
            figures_dir: default_figures_dir(),
            cache_dir: default_cache_dir(),
            venv: None,
            python_path: Vec::new(),
            env: std::collections::HashMap::new(),
            timeout: default_timeout(),
            project_root: PathBuf::new(),
        }
    }
}

impl PythonConfig {
    /// Load configuration from a project directory.
    ///
    /// Looks for `nova.toml` in the given directory and reads
    /// the `[python]` section.
    pub fn from_project(project_dir: impl AsRef<Path>) -> Result<Self> {
        let project_dir = project_dir.as_ref();
        let config_path = project_dir.join("nova.toml");

        if !config_path.exists() {
            return Err(Error::ConfigNotFound {
                path: config_path,
            });
        }

        let content = std::fs::read_to_string(&config_path)?;
        let nova_config: NovaToml = toml::from_str(&content)?;

        let mut config = nova_config.python.unwrap_or_default();
        config.project_root = project_dir.to_path_buf();

        // Resolve relative paths
        config.figures_dir = project_dir.join(&config.figures_dir);
        config.cache_dir = project_dir.join(&config.cache_dir);
        if let Some(ref venv) = config.venv {
            config.venv = Some(project_dir.join(venv));
        }
        config.python_path = config
            .python_path
            .into_iter()
            .map(|p| project_dir.join(p))
            .collect();

        Ok(config)
    }

    /// Get the effective Python executable path.
    ///
    /// If a venv is configured, returns the Python from that venv.
    pub fn python_executable(&self) -> PathBuf {
        if let Some(ref venv) = self.venv {
            #[cfg(windows)]
            let python = venv.join("Scripts").join("python.exe");
            #[cfg(not(windows))]
            let python = venv.join("bin").join("python");
            python
        } else {
            PathBuf::from(&self.python)
        }
    }

    /// Build the PYTHONPATH environment variable.
    pub fn build_python_path(&self) -> String {
        // Include project root so "import figures.xxx" works
        let mut paths: Vec<String> = vec![self.project_root.display().to_string()];

        // Also add figures_dir parent for nested imports
        if let Some(parent) = self.figures_dir.parent() {
            paths.push(parent.display().to_string());
        }

        for p in &self.python_path {
            paths.push(p.display().to_string());
        }

        #[cfg(windows)]
        let separator = ";";
        #[cfg(not(windows))]
        let separator = ":";

        paths.join(separator)
    }
}

/// Root structure of nova.toml.
#[derive(Debug, Deserialize)]
struct NovaToml {
    /// Python configuration section.
    python: Option<PythonConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
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

        let config = PythonConfig::from_project(dir.path()).unwrap();
        assert_eq!(config.python, "python3");
        assert_eq!(config.figures_dir, dir.path().join("src/figures"));
        assert_eq!(config.timeout, 120);
    }

    #[test]
    fn test_config_not_found() {
        let dir = TempDir::new().unwrap();
        let result = PythonConfig::from_project(dir.path());
        assert!(matches!(result, Err(Error::ConfigNotFound { .. })));
    }
}
