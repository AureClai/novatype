//! Caching system for generated figures.
//!
//! Caches SVG outputs based on source code hash and dependency modification times.

use crate::error::Result;
use crate::registry::Figure;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Cache for generated figure SVGs.
pub struct FigureCache {
    /// Directory where cached files are stored.
    cache_dir: PathBuf,
}

impl FigureCache {
    /// Create a new cache in the given directory.
    pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir)?;
        }

        Ok(Self { cache_dir })
    }

    /// Compute a cache key for a figure based on its source and dependencies.
    pub fn compute_key(&self, figure: &Figure) -> Result<String> {
        let mut hasher = Sha256::new();

        // Include source hash
        hasher.update(figure.source_hash.as_bytes());

        // Include dependency modification times
        for dep in &figure.dependencies {
            if dep.exists() {
                if let Ok(metadata) = std::fs::metadata(dep) {
                    if let Ok(modified) = metadata.modified() {
                        let duration = modified
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default();
                        hasher.update(duration.as_secs().to_le_bytes());
                    }
                }
            }
        }

        // Include figure name to avoid collisions
        hasher.update(figure.name.as_bytes());

        Ok(hex::encode(hasher.finalize()))
    }

    /// Get cached SVG path if it exists and is valid.
    pub fn get(&self, cache_key: &str) -> Result<Option<PathBuf>> {
        let cache_path = self.key_to_path(cache_key);

        if cache_path.exists() {
            Ok(Some(cache_path))
        } else {
            Ok(None)
        }
    }

    /// Get cached SVG by figure name (searches for any version).
    pub fn get_by_name(&self, name: &str) -> Result<Option<PathBuf>> {
        // Look for name.svg in cache directory
        let direct_path = self.cache_dir.join(format!("{}.svg", name));

        if direct_path.exists() {
            return Ok(Some(direct_path));
        }

        // Search for any file matching the name pattern
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if file_name.starts_with(name) && file_name.ends_with(".svg") {
                return Ok(Some(entry.path()));
            }
        }

        Ok(None)
    }

    /// Store a generated SVG in the cache.
    pub fn store(&self, cache_key: &str, svg_path: &Path) -> Result<PathBuf> {
        let cache_path = self.key_to_path(cache_key);

        // Copy or move the SVG to cache
        std::fs::copy(svg_path, &cache_path)?;

        tracing::debug!(path = %cache_path.display(), "Cached figure");
        Ok(cache_path)
    }

    /// Store SVG content directly by figure name.
    pub fn store_svg(&self, name: &str, svg_content: &str) -> Result<PathBuf> {
        let cache_path = self.cache_dir.join(format!("{}.svg", name));
        std::fs::write(&cache_path, svg_content)?;
        Ok(cache_path)
    }

    /// Invalidate a cached figure by name.
    pub fn invalidate_by_name(&self, name: &str) -> Result<()> {
        // Remove all files matching the name
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if file_name.starts_with(name) && file_name.ends_with(".svg") {
                std::fs::remove_file(entry.path())?;
                tracing::debug!(name, "Invalidated cached figure");
            }
        }

        Ok(())
    }

    /// Clear all cached figures.
    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|ext| ext == "svg") {
                    std::fs::remove_file(entry.path())?;
                }
            }
        }

        tracing::info!("Cleared figure cache");
        Ok(())
    }

    /// Get the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Convert a cache key to a file path.
    fn key_to_path(&self, cache_key: &str) -> PathBuf {
        // Use first 16 chars of hash + .svg
        let short_key = &cache_key[..cache_key.len().min(16)];
        self.cache_dir.join(format!("{}.svg", short_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_creation() {
        let dir = TempDir::new().unwrap();
        let cache_dir = dir.path().join("cache");

        let cache = FigureCache::new(&cache_dir).unwrap();
        assert!(cache_dir.exists());
    }

    #[test]
    fn test_store_and_get() {
        let dir = TempDir::new().unwrap();
        let cache = FigureCache::new(dir.path().join("cache")).unwrap();

        // Create a temp SVG file
        let svg_path = dir.path().join("test.svg");
        std::fs::write(&svg_path, "<svg></svg>").unwrap();

        // Store it
        let cache_key = "abc123def456";
        let cached_path = cache.store(cache_key, &svg_path).unwrap();
        assert!(cached_path.exists());

        // Retrieve it
        let retrieved = cache.get(cache_key).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), cached_path);
    }

    #[test]
    fn test_store_svg_by_name() {
        let dir = TempDir::new().unwrap();
        let cache = FigureCache::new(dir.path().join("cache")).unwrap();

        let path = cache.store_svg("my-figure", "<svg>test</svg>").unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("my-figure"));

        let retrieved = cache.get_by_name("my-figure").unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_invalidate() {
        let dir = TempDir::new().unwrap();
        let cache = FigureCache::new(dir.path().join("cache")).unwrap();

        cache.store_svg("to-delete", "<svg></svg>").unwrap();
        assert!(cache.get_by_name("to-delete").unwrap().is_some());

        cache.invalidate_by_name("to-delete").unwrap();
        assert!(cache.get_by_name("to-delete").unwrap().is_none());
    }

    #[test]
    fn test_compute_key() {
        let dir = TempDir::new().unwrap();
        let cache = FigureCache::new(dir.path()).unwrap();

        let fig1 = Figure::new("fig1", "test.py", "plot").with_hash("hash1");
        let fig2 = Figure::new("fig1", "test.py", "plot").with_hash("hash2");

        let key1 = cache.compute_key(&fig1).unwrap();
        let key2 = cache.compute_key(&fig2).unwrap();

        // Different hashes should produce different keys
        assert_ne!(key1, key2);
    }
}
