//! Font manager - high-level API for font operations.

use crate::cache::{FontCache, InstalledFont};
use crate::error::{Error, Result};
use crate::google_fonts::{FontFamily, GoogleFontsClient};
use crate::FontCategory;
use std::path::PathBuf;
use tracing::{debug, info, instrument};

/// High-level font manager.
///
/// Combines Google Fonts API access with local caching.
#[derive(Debug)]
pub struct FontManager {
    /// Google Fonts API client.
    client: GoogleFontsClient,

    /// Local font cache.
    cache: FontCache,
}

impl FontManager {
    /// Create a new font manager with default settings.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache cannot be initialized.
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: GoogleFontsClient::new(),
            cache: FontCache::new()?,
        })
    }

    /// Create a font manager with a custom cache directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache cannot be initialized.
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            client: GoogleFontsClient::new(),
            cache: FontCache::with_dir(cache_dir)?,
        })
    }

    /// Set the Google Fonts API key.
    #[must_use]
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.client = self.client.with_api_key(key);
        self
    }

    /// Search for fonts by name.
    ///
    /// # Arguments
    ///
    /// * `query` - Search query (case-insensitive)
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    #[instrument(skip(self))]
    pub async fn search(&self, query: &str) -> Result<Vec<FontSearchResult>> {
        let fonts = self.client.search(query).await?;

        Ok(fonts
            .into_iter()
            .map(|f| FontSearchResult {
                installed: self.cache.is_installed(&f.family),
                family: f,
            })
            .collect())
    }

    /// Search for fonts by category.
    ///
    /// # Arguments
    ///
    /// * `category` - Font category
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    #[instrument(skip(self))]
    pub async fn search_by_category(
        &self,
        category: FontCategory,
    ) -> Result<Vec<FontSearchResult>> {
        let fonts = self.client.search_by_category(category).await?;

        Ok(fonts
            .into_iter()
            .map(|f| FontSearchResult {
                installed: self.cache.is_installed(&f.family),
                family: f,
            })
            .collect())
    }

    /// Install a font from Google Fonts.
    ///
    /// Downloads all variants of the font.
    ///
    /// # Arguments
    ///
    /// * `name` - Font family name
    ///
    /// # Errors
    ///
    /// Returns an error if the font is not found or download fails.
    #[instrument(skip(self))]
    pub async fn install(&mut self, name: &str) -> Result<Vec<PathBuf>> {
        if self.cache.is_installed(name) {
            return Err(Error::AlreadyInstalled {
                name: name.to_string(),
            });
        }

        info!(font = %name, "Installing font");
        let family = self.client.get_font(name).await?;

        self.install_family(&family).await
    }

    /// Install specific variants of a font.
    ///
    /// # Arguments
    ///
    /// * `name` - Font family name
    /// * `variants` - Variants to install (e.g., ["regular", "700", "700italic"])
    ///
    /// # Errors
    ///
    /// Returns an error if the font is not found or download fails.
    #[instrument(skip(self))]
    pub async fn install_variants(
        &mut self,
        name: &str,
        variants: &[&str],
    ) -> Result<Vec<PathBuf>> {
        let family = self.client.get_font(name).await?;

        let mut paths = Vec::new();
        for variant in variants {
            if let Some(url) = family.file_url(variant) {
                debug!(variant, %url, "Downloading variant");
                let data = self.client.download_font(url).await?;
                let path = self.cache.store_font(&family, variant, &data)?;
                paths.push(path);
            } else {
                debug!(variant, "Variant not available");
            }
        }

        Ok(paths)
    }

    /// Install all variants of a font family.
    async fn install_family(&mut self, family: &FontFamily) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        for (variant, url) in &family.files {
            debug!(%variant, %url, "Downloading variant");
            let data = self.client.download_font(url).await?;
            let path = self.cache.store_font(family, variant, &data)?;
            paths.push(path);
        }

        Ok(paths)
    }

    /// Uninstall a font.
    ///
    /// # Errors
    ///
    /// Returns an error if the font is not installed.
    #[instrument(skip(self))]
    pub fn uninstall(&mut self, name: &str) -> Result<()> {
        self.cache.remove_font(name)
    }

    /// List all installed fonts.
    #[must_use]
    pub fn list_installed(&self) -> Vec<&InstalledFont> {
        self.cache.list_installed()
    }

    /// Check if a font is installed.
    #[must_use]
    pub fn is_installed(&self, name: &str) -> bool {
        self.cache.is_installed(name)
    }

    /// Get all font paths for Typst.
    ///
    /// Returns paths to all installed font directories.
    #[must_use]
    pub fn typst_font_paths(&self) -> Vec<PathBuf> {
        self.cache.typst_font_paths()
    }

    /// Get the cache directory path.
    #[must_use]
    pub fn cache_dir(&self) -> &std::path::Path {
        self.cache.cache_dir()
    }

    /// Get the total cache size in bytes.
    pub fn cache_size(&self) -> Result<u64> {
        self.cache.cache_size()
    }

    /// Clear the font cache.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache cannot be cleared.
    pub fn clear_cache(&mut self) -> Result<()> {
        self.cache.clear()
    }

    /// Get information about a font.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn get_font_info(&self, name: &str) -> Result<FontInfo> {
        let family = self.client.get_font(name).await?;
        let installed = self.cache.is_installed(name);
        let local_files = if installed {
            self.cache.font_files(name).ok()
        } else {
            None
        };

        Ok(FontInfo {
            family,
            installed,
            local_files,
        })
    }
}

/// Font search result.
#[derive(Debug)]
pub struct FontSearchResult {
    /// Whether the font is installed locally.
    pub installed: bool,

    /// Font family information.
    pub family: FontFamily,
}

/// Detailed font information.
#[derive(Debug)]
pub struct FontInfo {
    /// Font family information from Google Fonts.
    pub family: FontFamily,

    /// Whether the font is installed locally.
    pub installed: bool,

    /// Local file paths (if installed).
    pub local_files: Option<Vec<PathBuf>>,
}

/// Predefined font bundles.
pub mod bundles {
    /// Academic fonts bundle.
    pub const ACADEMIC: &[&str] = &[
        "EB Garamond",
        "Libre Baskerville",
        "Source Serif Pro",
        "Crimson Text",
    ];

    /// Modern fonts bundle.
    pub const MODERN: &[&str] = &["Inter", "Roboto", "Open Sans", "Lato"];

    /// Monospace fonts bundle.
    pub const MONOSPACE: &[&str] = &[
        "JetBrains Mono",
        "Fira Code",
        "Source Code Pro",
        "Roboto Mono",
    ];

    /// Classic fonts bundle.
    pub const CLASSIC: &[&str] = &["Playfair Display", "Merriweather", "Lora", "PT Serif"];

    /// Minimal bundle (one sans, one serif, one mono).
    pub const MINIMAL: &[&str] = &["Inter", "Source Serif Pro", "JetBrains Mono"];
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_manager() -> (FontManager, TempDir) {
        let dir = TempDir::new().unwrap();
        let manager = FontManager::with_cache_dir(dir.path().to_path_buf()).unwrap();
        (manager, dir)
    }

    #[test]
    fn manager_creation() {
        let (manager, _dir) = test_manager();
        assert!(manager.list_installed().is_empty());
    }

    #[test]
    fn font_not_installed() {
        let (manager, _dir) = test_manager();
        assert!(!manager.is_installed("Nonexistent Font"));
    }

    #[test]
    fn bundles_not_empty() {
        assert!(!bundles::ACADEMIC.is_empty());
        assert!(!bundles::MODERN.is_empty());
        assert!(!bundles::MONOSPACE.is_empty());
        assert!(!bundles::CLASSIC.is_empty());
        assert!(!bundles::MINIMAL.is_empty());
    }
}
