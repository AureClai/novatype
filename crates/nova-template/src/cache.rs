//! Template cache management.
//!
//! Handles local storage of installed templates.

use crate::error::{Error, Result};
use crate::template::{InstalledTemplate, Template, TemplateCategory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Default cache directory name.
const CACHE_DIR_NAME: &str = "nova-templates";

/// Cache metadata file name.
const METADATA_FILE: &str = "templates.json";

/// Template cache manager.
#[derive(Debug)]
pub struct TemplateCache {
    /// Cache directory path.
    cache_dir: PathBuf,

    /// Cached template metadata.
    metadata: CacheMetadata,
}

impl TemplateCache {
    /// Create a new template cache with the default directory.
    ///
    /// The default directory is:
    /// - Linux: `~/.local/share/nova-templates`
    /// - macOS: `~/Library/Application Support/nova-templates`
    /// - Windows: `%APPDATA%/nova-templates`
    pub fn new() -> Result<Self> {
        let cache_dir = Self::default_cache_dir()?;
        Self::with_dir(cache_dir)
    }

    /// Create a template cache with a custom directory.
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
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get the templates directory.
    pub fn templates_dir(&self) -> PathBuf {
        self.cache_dir.join("templates")
    }

    /// Get the path to an installed template.
    pub fn template_dir(&self, name: &str) -> PathBuf {
        self.templates_dir().join(sanitize_name(name))
    }

    /// Check if a template is installed.
    pub fn is_installed(&self, name: &str) -> bool {
        self.metadata.templates.contains_key(name)
    }

    /// Get an installed template.
    pub fn get(&self, name: &str) -> Option<&InstalledTemplate> {
        self.metadata.templates.get(name)
    }

    /// List all installed templates.
    pub fn list_installed(&self) -> Vec<&InstalledTemplate> {
        self.metadata.templates.values().collect()
    }

    /// List installed templates by category.
    pub fn list_by_category(&self, category: TemplateCategory) -> Vec<&InstalledTemplate> {
        self.metadata
            .templates
            .values()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Install a template from a directory.
    ///
    /// Copies the template files to the cache.
    pub fn install_from_directory(
        &mut self,
        source_dir: impl AsRef<Path>,
        source_name: &str,
    ) -> Result<InstalledTemplate> {
        let source_dir = source_dir.as_ref();

        // Validate template
        Template::validate_directory(source_dir)?;

        // Load template metadata
        let template = Template::from_directory(source_dir)?;
        let name = template.name().to_string();

        // Check if already installed
        if self.is_installed(&name) {
            return Err(Error::TemplateAlreadyExists { name });
        }

        // Create target directory
        let target_dir = self.template_dir(&name);
        fs::create_dir_all(&target_dir)?;

        // Copy all files
        copy_dir_recursive(source_dir, &target_dir)?;

        // Create installed entry
        let installed = InstalledTemplate {
            name: name.clone(),
            version: template.version().to_string(),
            source: source_name.to_string(),
            path: target_dir,
            installed_at: chrono::Utc::now().to_rfc3339(),
            category: template.template.category,
            description: template.template.description.clone(),
        };

        // Update metadata
        self.metadata.templates.insert(name.clone(), installed.clone());
        self.save_metadata()?;

        info!(name = %name, "Template installed");
        Ok(installed)
    }

    /// Remove an installed template.
    pub fn remove(&mut self, name: &str) -> Result<()> {
        if !self.is_installed(name) {
            return Err(Error::TemplateNotFound {
                name: name.to_string(),
            });
        }

        let template_dir = self.template_dir(name);
        if template_dir.exists() {
            fs::remove_dir_all(&template_dir)?;
        }

        self.metadata.templates.remove(name);
        self.save_metadata()?;

        info!(name = %name, "Template removed");
        Ok(())
    }

    /// Load a template from the cache.
    pub fn load_template(&self, name: &str) -> Result<Template> {
        let installed = self.get(name).ok_or_else(|| Error::TemplateNotFound {
            name: name.to_string(),
        })?;

        Template::from_directory(&installed.path)
    }

    /// Get the main.typ path for a template.
    pub fn template_main_path(&self, name: &str) -> Result<PathBuf> {
        let installed = self.get(name).ok_or_else(|| Error::TemplateNotFound {
            name: name.to_string(),
        })?;

        Ok(installed.path.join("main.typ"))
    }

    /// Save metadata to disk.
    fn save_metadata(&self) -> Result<()> {
        let metadata_path = self.cache_dir.join(METADATA_FILE);
        let content = serde_json::to_string_pretty(&self.metadata)?;
        fs::write(metadata_path, content)?;
        Ok(())
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        let total_size = self
            .metadata
            .templates
            .values()
            .map(|t| dir_size(&t.path).unwrap_or(0))
            .sum();

        CacheStats {
            template_count: self.metadata.templates.len(),
            total_size,
        }
    }
}

/// Cache metadata stored in JSON.
#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheMetadata {
    /// Installed templates by name.
    templates: HashMap<String, InstalledTemplate>,
}

/// Cache statistics.
#[derive(Debug)]
pub struct CacheStats {
    /// Number of installed templates.
    pub template_count: usize,
    /// Total size in bytes.
    pub total_size: u64,
}

/// Sanitize a template name for use as directory name.
fn sanitize_name(name: &str) -> String {
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

/// Copy a directory recursively.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
            debug!(?src_path, ?dst_path, "Copied file");
        }
    }

    Ok(())
}

/// Calculate directory size recursively.
fn dir_size(path: &Path) -> Result<u64> {
    let mut size = 0;

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                size += dir_size(&path)?;
            } else {
                size += entry.metadata()?.len();
            }
        }
    }

    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TEMPLATE_FILE;
    use tempfile::TempDir;

    fn create_test_template(dir: &Path) {
        let template_toml = r#"
[template]
name = "test-template"
version = "1.0.0"
category = "article"
"#;
        fs::write(dir.join(TEMPLATE_FILE), template_toml).unwrap();
        fs::write(dir.join("main.typ"), "// Test\n").unwrap();
    }

    #[test]
    fn cache_creation() {
        let dir = TempDir::new().unwrap();
        let cache = TemplateCache::with_dir(dir.path().to_path_buf()).unwrap();
        assert!(cache.list_installed().is_empty());
    }

    #[test]
    fn install_template() {
        let cache_dir = TempDir::new().unwrap();
        let template_dir = TempDir::new().unwrap();

        create_test_template(template_dir.path());

        let mut cache = TemplateCache::with_dir(cache_dir.path().to_path_buf()).unwrap();
        let installed = cache
            .install_from_directory(template_dir.path(), "local")
            .unwrap();

        assert_eq!(installed.name, "test-template");
        assert!(cache.is_installed("test-template"));
        assert_eq!(cache.list_installed().len(), 1);
    }

    #[test]
    fn remove_template() {
        let cache_dir = TempDir::new().unwrap();
        let template_dir = TempDir::new().unwrap();

        create_test_template(template_dir.path());

        let mut cache = TemplateCache::with_dir(cache_dir.path().to_path_buf()).unwrap();
        cache
            .install_from_directory(template_dir.path(), "local")
            .unwrap();

        cache.remove("test-template").unwrap();
        assert!(!cache.is_installed("test-template"));
    }

    #[test]
    fn install_duplicate_fails() {
        let cache_dir = TempDir::new().unwrap();
        let template_dir = TempDir::new().unwrap();

        create_test_template(template_dir.path());

        let mut cache = TemplateCache::with_dir(cache_dir.path().to_path_buf()).unwrap();
        cache
            .install_from_directory(template_dir.path(), "local")
            .unwrap();

        let result = cache.install_from_directory(template_dir.path(), "local");
        assert!(matches!(result, Err(Error::TemplateAlreadyExists { .. })));
    }

    #[test]
    fn sanitize_names() {
        assert_eq!(sanitize_name("my-template"), "my-template");
        assert_eq!(sanitize_name("My Template"), "My-Template");
        assert_eq!(sanitize_name("template/name"), "template_name");
    }
}
