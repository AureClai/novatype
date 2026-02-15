//! # Nova Python
//!
//! Python integration for NovaType documents.
//!
//! This crate enables seamless integration of Python-generated figures
//! in Typst documents. Instead of embedding Python code directly in .typ files,
//! users write Python scripts with decorated functions that export figures.
//!
//! ## Architecture
//!
//! ```text
//! figures/
//! ├── __init__.py
//! ├── analysis.py     # @nova.figure("results")
//! └── plots.py        # @nova.figure("sine-wave")
//!           ↓
//!     nova-python (this crate)
//!           ↓
//!     .nova/cache/
//!     ├── results.svg
//!     └── sine-wave.svg
//!           ↓
//!     main.typ: #nova("results")
//! ```
//!
//! ## Example
//!
//! Python side (`figures/plots.py`):
//! ```python
//! import nova
//! import matplotlib.pyplot as plt
//! import numpy as np
//!
//! @nova.figure("sine-wave")
//! def plot_sine():
//!     x = np.linspace(0, 2*np.pi, 100)
//!     plt.plot(x, np.sin(x))
//! ```
//!
//! Typst side (`main.typ`):
//! ```typst
//! #figure(
//!   nova("sine-wave"),
//!   caption: [A sine wave]
//! )
//! ```

pub mod cache;
pub mod config;
pub mod discovery;
pub mod error;
pub mod executor;
pub mod registry;

pub use cache::FigureCache;
pub use config::{load_python_config, PythonConfig};
pub use discovery::FigureDiscovery;
pub use error::{Error, Result};
pub use executor::PythonExecutor;
pub use registry::{Figure, FigureRegistry};

/// Main entry point for Python figure generation.
///
/// Orchestrates the full pipeline:
/// 1. Load configuration from nova.toml
/// 2. Discover @nova.figure decorated functions
/// 3. Check cache for existing figures
/// 4. Execute Python for outdated/missing figures
/// 5. Return paths to generated SVGs
pub struct NovaPython {
    config: PythonConfig,
    cache: FigureCache,
    executor: PythonExecutor,
}

impl NovaPython {
    /// Create a new NovaPython instance from project configuration.
    pub fn new(config: PythonConfig) -> Result<Self> {
        let cache = FigureCache::new(&config.cache_dir)?;
        let executor = PythonExecutor::new(&config)?;

        Ok(Self {
            config,
            cache,
            executor,
        })
    }

    /// Load NovaPython from a project directory containing nova.toml.
    pub fn from_project(project_dir: impl AsRef<std::path::Path>) -> Result<Self> {
        let config = load_python_config(project_dir)?;
        Self::new(config)
    }

    /// Generate all figures, using cache when possible.
    ///
    /// Returns a registry of all available figures with their SVG paths.
    pub async fn generate_figures(&self) -> Result<FigureRegistry> {
        // Discover all decorated figures in Python files
        let discovered = FigureDiscovery::scan(&self.config.figures_dir)?;

        let mut registry = FigureRegistry::new();

        for figure in discovered {
            let cache_key = self.cache.compute_key(&figure)?;

            if let Some(cached_path) = self.cache.get(&cache_key)? {
                // Use cached version
                tracing::debug!(name = %figure.name, "Using cached figure");
                registry.register(figure.name.clone(), cached_path);
            } else {
                // Generate new figure
                tracing::info!(name = %figure.name, "Generating figure");
                let svg_path = self.executor.execute_figure(&figure).await?;
                self.cache.store(&cache_key, &svg_path)?;
                registry.register(figure.name.clone(), svg_path);
            }
        }

        Ok(registry)
    }

    /// Get the path to a specific figure's SVG.
    ///
    /// Returns None if the figure doesn't exist.
    pub fn get_figure_path(&self, name: &str) -> Option<std::path::PathBuf> {
        self.cache.get_by_name(name).ok().flatten()
    }

    /// Invalidate cache for a specific figure.
    pub fn invalidate(&self, name: &str) -> Result<()> {
        self.cache.invalidate_by_name(name)
    }

    /// Clear all cached figures.
    pub fn clear_cache(&self) -> Result<()> {
        self.cache.clear()
    }

    /// Check if any Python files have changed since last generation.
    pub fn needs_regeneration(&self) -> Result<bool> {
        let discovered = FigureDiscovery::scan(&self.config.figures_dir)?;

        for figure in discovered {
            let cache_key = self.cache.compute_key(&figure)?;
            if self.cache.get(&cache_key)?.is_none() {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
