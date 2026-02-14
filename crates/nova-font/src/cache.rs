//! Font cache management.
//!
//! Handles local storage of downloaded fonts.

use crate::error::{Error, Result};
use crate::google_fonts::FontFamily;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, instrument};

/// Default cache directory name.
const CACHE_DIR_NAME: &str = "nova-fonts";

/// Cache metadata file name.
const METADATA_FILE: &str = "fonts.json";

/// Font cache manager.
#[derive(Debug)]
pub struct FontCache {
    /// Cache directory path.
    cache_dir: PathBuf,

    /// Cached font metadata.
    metadata: CacheMetadata,
}

impl FontCache {
    /// Create a new font cache with the default directory.
    ///
    /// The default directory is `~/.local/share/nova-fonts` on Linux,
    /// `~/Library/Application Support/nova-fonts` on macOS,
    /// and `%APPDATA%/nova-fonts` on Windows.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be created.
    pub fn new() -> Result<Self> {
        let cache_dir = Self::default_cache_dir()?;
        Self::with_dir(cache_dir)
    }

    /// Create a font cache with a custom directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or accessed.
    pub fn with_dir(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)?;

        let metadata_path = cache_dir.join(METADATA_FILE);
        let metadata = if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            CacheMetadata::default()
        };

        Ok(Self {
            cache_dir,
            metadata,
        })
    }

    /// Get the default cache directory.
    fn default_cache_dir() -> Result<PathBuf> {
        dirs::data_dir()
            .map(|d| d.join(CACHE_DIR_NAME))
            .ok_or_else(|| Error::Cache {
                message: "Could not determine data directory".to_string(),
            })
    }

    /// Get the cache directory path.
    #[must_use]
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get the fonts directory (where font files are stored).
    #[must_use]
    pub fn fonts_dir(&self) -> PathBuf {
        self.cache_dir.join("fonts")
    }

    /// Check if a font is installed.
    #[must_use]
    pub fn is_installed(&self, family_name: &str) -> bool {
        self.metadata.fonts.contains_key(family_name)
    }

    /// Get the path to an installed font family's directory.
    #[must_use]
    pub fn font_dir(&self, family_name: &str) -> PathBuf {
        self.fonts_dir().join(sanitize_filename(family_name))
    }

    /// Get all installed font paths for a family.
    pub fn font_files(&self, family_name: &str) -> Result<Vec<PathBuf>> {
        let entry = self
            .metadata
            .fonts
            .get(family_name)
            .ok_or_else(|| Error::FontNotFound {
                name: family_name.to_string(),
            })?;

        Ok(entry.files.iter().map(|f| f.path.clone()).collect())
    }

    /// List all installed fonts.
    #[must_use]
    pub fn list_installed(&self) -> Vec<&InstalledFont> {
        self.metadata.fonts.values().collect()
    }

    /// Store a font file in the cache.
    ///
    /// # Arguments
    ///
    /// * `family` - Font family metadata
    /// * `variant` - Variant name (e.g., "regular", "700italic")
    /// * `data` - Font file data
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    #[instrument(skip(self, data))]
    pub fn store_font(
        &mut self,
        family: &FontFamily,
        variant: &str,
        data: &[u8],
    ) -> Result<PathBuf> {
        let font_dir = self.font_dir(&family.family);
        fs::create_dir_all(&font_dir)?;

        // Determine file extension from URL or default to .ttf
        let extension = family
            .file_url(variant)
            .and_then(|url| url.rsplit('.').next())
            .unwrap_or("ttf");

        let filename = format!(
            "{}-{}.{}",
            sanitize_filename(&family.family),
            variant,
            extension
        );
        let file_path = font_dir.join(&filename);

        // Calculate checksum
        let checksum = hex::encode(Sha256::digest(data));

        debug!(?file_path, %checksum, "Storing font file");
        fs::write(&file_path, data)?;

        // Update metadata
        let entry = self
            .metadata
            .fonts
            .entry(family.family.clone())
            .or_insert_with(|| InstalledFont {
                family: family.family.clone(),
                category: family.category.clone(),
                version: family.version.clone(),
                files: Vec::new(),
            });

        // Add file if not already present
        if !entry.files.iter().any(|f| f.variant == variant) {
            entry.files.push(FontFile {
                variant: variant.to_string(),
                path: file_path.clone(),
                checksum,
            });
        }

        self.save_metadata()?;
        info!(family = %family.family, variant, "Font installed");

        Ok(file_path)
    }

    /// Remove a font from the cache.
    ///
    /// # Errors
    ///
    /// Returns an error if the font cannot be removed.
    #[instrument(skip(self))]
    pub fn remove_font(&mut self, family_name: &str) -> Result<()> {
        if !self.is_installed(family_name) {
            return Err(Error::FontNotFound {
                name: family_name.to_string(),
            });
        }

        let font_dir = self.font_dir(family_name);
        if font_dir.exists() {
            fs::remove_dir_all(&font_dir)?;
        }

        self.metadata.fonts.remove(family_name);
        self.save_metadata()?;

        info!(family = %family_name, "Font removed");
        Ok(())
    }

    /// Clear all cached fonts.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache cannot be cleared.
    pub fn clear(&mut self) -> Result<()> {
        let fonts_dir = self.fonts_dir();
        if fonts_dir.exists() {
            fs::remove_dir_all(&fonts_dir)?;
        }
        fs::create_dir_all(&fonts_dir)?;

        self.metadata.fonts.clear();
        self.save_metadata()?;

        info!("Font cache cleared");
        Ok(())
    }

    /// Get the total size of the cache in bytes.
    pub fn cache_size(&self) -> Result<u64> {
        let mut total = 0;
        for entry in &self.metadata.fonts {
            for file in &entry.1.files {
                if file.path.exists() {
                    total += fs::metadata(&file.path)?.len();
                }
            }
        }
        Ok(total)
    }

    /// Save metadata to disk.
    fn save_metadata(&self) -> Result<()> {
        let metadata_path = self.cache_dir.join(METADATA_FILE);
        let content = serde_json::to_string_pretty(&self.metadata)?;
        fs::write(metadata_path, content)?;
        Ok(())
    }

    /// Get all font paths for Typst.
    ///
    /// Returns paths to all installed font directories.
    #[must_use]
    pub fn typst_font_paths(&self) -> Vec<PathBuf> {
        self.metadata
            .fonts
            .keys()
            .map(|name| self.font_dir(name))
            .filter(|p| p.exists())
            .collect()
    }
}

/// Sanitize a filename (remove invalid characters).
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else if c == ' ' {
                '-'
            } else {
                '_'
            }
        })
        .collect()
}

/// Cache metadata stored in JSON.
#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheMetadata {
    /// Installed fonts by family name.
    fonts: HashMap<String, InstalledFont>,
}

/// Information about an installed font.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledFont {
    /// Font family name.
    pub family: String,

    /// Font category.
    pub category: Option<String>,

    /// Font version.
    pub version: Option<String>,

    /// Installed font files.
    pub files: Vec<FontFile>,
}

/// A single font file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFile {
    /// Variant name (e.g., "regular", "700italic").
    pub variant: String,

    /// Path to the font file.
    pub path: PathBuf,

    /// SHA256 checksum.
    pub checksum: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_cache() -> (FontCache, TempDir) {
        let dir = TempDir::new().unwrap();
        let cache = FontCache::with_dir(dir.path().to_path_buf()).unwrap();
        (cache, dir)
    }

    #[test]
    fn sanitize_filenames() {
        assert_eq!(sanitize_filename("JetBrains Mono"), "JetBrains-Mono");
        assert_eq!(sanitize_filename("Noto Sans JP"), "Noto-Sans-JP");
        assert_eq!(sanitize_filename("Font/Name"), "Font_Name");
    }

    #[test]
    fn cache_creation() {
        let (cache, _dir) = test_cache();
        assert!(cache.cache_dir().exists());
        assert!(cache.list_installed().is_empty());
    }

    #[test]
    fn font_not_installed() {
        let (cache, _dir) = test_cache();
        assert!(!cache.is_installed("Test Font"));
    }

    #[test]
    fn store_and_retrieve() {
        let (mut cache, _dir) = test_cache();

        let family = FontFamily {
            family: "Test Font".to_string(),
            category: Some("sans-serif".to_string()),
            variants: vec!["regular".to_string()],
            subsets: vec!["latin".to_string()],
            version: Some("v1".to_string()),
            last_modified: None,
            files: HashMap::from([(
                "regular".to_string(),
                "http://example.com/font.ttf".to_string(),
            )]),
        };

        let data = b"fake font data";
        let path = cache.store_font(&family, "regular", data).unwrap();

        assert!(path.exists());
        assert!(cache.is_installed("Test Font"));
        assert_eq!(cache.list_installed().len(), 1);
    }

    #[test]
    fn remove_font() {
        let (mut cache, _dir) = test_cache();

        let family = FontFamily {
            family: "Test Font".to_string(),
            category: None,
            variants: vec!["regular".to_string()],
            subsets: vec![],
            version: None,
            last_modified: None,
            files: HashMap::new(),
        };

        cache.store_font(&family, "regular", b"data").unwrap();
        assert!(cache.is_installed("Test Font"));

        cache.remove_font("Test Font").unwrap();
        assert!(!cache.is_installed("Test Font"));
    }
}
