//! Figure registry for tracking discovered and generated figures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A discovered Python figure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Figure {
    /// Unique name of the figure (from @nova.figure decorator).
    pub name: String,

    /// Source file containing the figure function.
    pub source_file: PathBuf,

    /// Name of the Python function.
    pub function_name: String,

    /// File dependencies declared via depends=[...].
    pub dependencies: Vec<PathBuf>,

    /// Hash of the source code for cache invalidation.
    pub source_hash: String,
}

impl Figure {
    /// Create a new figure definition.
    pub fn new(
        name: impl Into<String>,
        source_file: impl Into<PathBuf>,
        function_name: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            source_file: source_file.into(),
            function_name: function_name.into(),
            dependencies: Vec::new(),
            source_hash: String::new(),
        }
    }

    /// Add a file dependency.
    pub fn with_dependency(mut self, path: impl Into<PathBuf>) -> Self {
        self.dependencies.push(path.into());
        self
    }

    /// Set the source hash.
    pub fn with_hash(mut self, hash: impl Into<String>) -> Self {
        self.source_hash = hash.into();
        self
    }
}

/// Registry of available figures and their generated SVG paths.
#[derive(Debug, Default)]
pub struct FigureRegistry {
    /// Map of figure name to SVG path.
    figures: HashMap<String, PathBuf>,
}

impl FigureRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a figure with its generated SVG path.
    pub fn register(&mut self, name: String, svg_path: PathBuf) {
        self.figures.insert(name, svg_path);
    }

    /// Get the SVG path for a figure by name.
    pub fn get(&self, name: &str) -> Option<&PathBuf> {
        self.figures.get(name)
    }

    /// Check if a figure exists in the registry.
    pub fn contains(&self, name: &str) -> bool {
        self.figures.contains_key(name)
    }

    /// Get all figure names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.figures.keys()
    }

    /// Get the number of registered figures.
    pub fn len(&self) -> usize {
        self.figures.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.figures.is_empty()
    }

    /// Iterate over all figures.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &PathBuf)> {
        self.figures.iter()
    }
}

impl IntoIterator for FigureRegistry {
    type Item = (String, PathBuf);
    type IntoIter = std::collections::hash_map::IntoIter<String, PathBuf>;

    fn into_iter(self) -> Self::IntoIter {
        self.figures.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_figure_creation() {
        let fig = Figure::new("test", "figures/test.py", "plot_test")
            .with_dependency("data/test.csv")
            .with_hash("abc123");

        assert_eq!(fig.name, "test");
        assert_eq!(fig.source_file, PathBuf::from("figures/test.py"));
        assert_eq!(fig.function_name, "plot_test");
        assert_eq!(fig.dependencies.len(), 1);
        assert_eq!(fig.source_hash, "abc123");
    }

    #[test]
    fn test_registry() {
        let mut registry = FigureRegistry::new();
        registry.register("fig1".to_string(), PathBuf::from("cache/fig1.svg"));
        registry.register("fig2".to_string(), PathBuf::from("cache/fig2.svg"));

        assert!(registry.contains("fig1"));
        assert!(registry.contains("fig2"));
        assert!(!registry.contains("fig3"));
        assert_eq!(registry.len(), 2);
    }
}
