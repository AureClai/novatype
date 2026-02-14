//! Discovery of @nova.figure decorated functions in Python files.

use crate::error::Result;
use crate::registry::Figure;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::path::Path;
use walkdir::WalkDir;

/// Discovers figures defined in Python files.
pub struct FigureDiscovery;

impl FigureDiscovery {
    /// Scan a directory for Python files containing @nova.figure decorators.
    ///
    /// Returns a list of discovered figures with their metadata.
    pub fn scan(dir: impl AsRef<Path>) -> Result<Vec<Figure>> {
        let dir = dir.as_ref();

        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut figures = Vec::new();

        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "py") {
                let discovered = Self::scan_file(path)?;
                figures.extend(discovered);
            }
        }

        tracing::debug!(count = figures.len(), "Discovered figures");
        Ok(figures)
    }

    /// Scan a single Python file for @nova.figure decorators.
    pub fn scan_file(path: impl AsRef<Path>) -> Result<Vec<Figure>> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        let source_hash = Self::compute_hash(&content);

        Self::parse_figures(&content, path, &source_hash)
    }

    /// Parse @nova.figure decorators from Python source code.
    fn parse_figures(content: &str, source_file: &Path, source_hash: &str) -> Result<Vec<Figure>> {
        let mut figures = Vec::new();

        // Pattern to match @nova.figure("name") or @nova.figure("name", depends=[...])
        // This regex captures:
        // 1. The figure name
        // 2. Optional depends parameter
        // 3. The function name on the next line
        // Note: \r?\n handles both Windows (\r\n) and Unix (\n) line endings
        let decorator_pattern = Regex::new(
            r#"@nova\.figure\(\s*["']([^"']+)["'](?:\s*,\s*depends\s*=\s*\[([^\]]*)\])?\s*\)\s*\r?\ndef\s+(\w+)"#,
        )?;

        let dep_pattern = Regex::new(r#"["']([^"']+)["']"#)?;

        for caps in decorator_pattern.captures_iter(content) {
            let name = caps.get(1).unwrap().as_str().to_string();
            let function_name = caps.get(3).unwrap().as_str().to_string();

            let mut figure = Figure::new(&name, source_file, &function_name).with_hash(source_hash);

            // Parse dependencies if present
            if let Some(deps_match) = caps.get(2) {
                let deps_str = deps_match.as_str();

                for dep_cap in dep_pattern.captures_iter(deps_str) {
                    let dep_path = dep_cap.get(1).unwrap().as_str();
                    figure = figure.with_dependency(dep_path);
                }
            }

            figures.push(figure);
        }

        Ok(figures)
    }

    /// Compute SHA256 hash of content.
    fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_parse_simple_figure() {
        let content = r#"
import nova
import matplotlib.pyplot as plt

@nova.figure("sine-wave")
def plot_sine():
    import numpy as np
    x = np.linspace(0, 2*np.pi, 100)
    plt.plot(x, np.sin(x))
"#;

        let figures =
            FigureDiscovery::parse_figures(content, Path::new("test.py"), "hash123").unwrap();

        assert_eq!(figures.len(), 1);
        assert_eq!(figures[0].name, "sine-wave");
        assert_eq!(figures[0].function_name, "plot_sine");
    }

    #[test]
    fn test_parse_figure_with_dependencies() {
        let content = r#"
import nova

@nova.figure("results", depends=["data/results.csv", "data/config.json"])
def plot_results():
    pass
"#;

        let figures =
            FigureDiscovery::parse_figures(content, Path::new("test.py"), "hash123").unwrap();

        assert_eq!(figures.len(), 1);
        assert_eq!(figures[0].name, "results");
        assert_eq!(figures[0].dependencies.len(), 2);
        assert_eq!(
            figures[0].dependencies[0],
            PathBuf::from("data/results.csv")
        );
    }

    #[test]
    fn test_parse_multiple_figures() {
        let content = r#"
import nova

@nova.figure("fig1")
def plot_one():
    pass

@nova.figure("fig2")
def plot_two():
    pass

@nova.figure("fig3", depends=["data.csv"])
def plot_three():
    pass
"#;

        let figures =
            FigureDiscovery::parse_figures(content, Path::new("test.py"), "hash123").unwrap();

        assert_eq!(figures.len(), 3);
        assert_eq!(figures[0].name, "fig1");
        assert_eq!(figures[1].name, "fig2");
        assert_eq!(figures[2].name, "fig3");
    }

    #[test]
    fn test_scan_directory() {
        let dir = TempDir::new().unwrap();

        // Create a Python file with figures
        let py_file = dir.path().join("plots.py");
        let mut file = std::fs::File::create(&py_file).unwrap();
        writeln!(
            file,
            r#"
import nova

@nova.figure("test-fig")
def make_figure():
    pass
"#
        )
        .unwrap();

        let figures = FigureDiscovery::scan(dir.path()).unwrap();
        assert_eq!(figures.len(), 1);
        assert_eq!(figures[0].name, "test-fig");
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let figures = FigureDiscovery::scan("/nonexistent/path").unwrap();
        assert!(figures.is_empty());
    }
}
