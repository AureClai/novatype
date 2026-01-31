//! Citation manager coordinating multiple sources.
//!
//! The [`CitationManager`] provides a unified interface for resolving
//! citations from local bibliographies and remote APIs.

use crate::bibtex::{BibEntry, Bibliography};
use crate::crossref::CrossRefClient;
use crate::error::{Error, Result};
use crate::formatter::{CitationStyle, Formatter};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, instrument};

/// Manages citation resolution and formatting.
#[derive(Debug)]
pub struct CitationManager {
    /// Local bibliographies.
    bibliographies: Vec<Bibliography>,

    /// CrossRef client (optional).
    crossref: Option<CrossRefClient>,

    /// Citation cache.
    cache: Arc<RwLock<HashMap<String, BibEntry>>>,

    /// Default citation style.
    default_style: CitationStyle,

    /// Formatter instance.
    formatter: Formatter,
}

impl CitationManager {
    /// Create a new citation manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bibliographies: Vec::new(),
            crossref: None,
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_style: CitationStyle::Ieee,
            formatter: Formatter::new().with_style(CitationStyle::Ieee),
        }
    }

    /// Add a local bibliography.
    #[must_use]
    pub fn with_bibliography(mut self, bib: Bibliography) -> Self {
        self.bibliographies.push(bib);
        self
    }

    /// Load and add a bibliography file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load_bibliography(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let bib = Bibliography::from_file(path)?;
        self.bibliographies.push(bib);
        Ok(())
    }

    /// Enable CrossRef API lookups.
    #[must_use]
    pub fn with_crossref(mut self, client: CrossRefClient) -> Self {
        self.crossref = Some(client);
        self
    }

    /// Set the default citation style.
    #[must_use]
    pub fn with_style(mut self, style: CitationStyle) -> Self {
        self.default_style = style;
        self.formatter = Formatter::new().with_style(style);
        self
    }

    /// Resolve a citation by key.
    ///
    /// Searches in order:
    /// 1. Cache
    /// 2. Local bibliographies
    /// 3. CrossRef API (if enabled and key looks like a DOI)
    ///
    /// # Errors
    ///
    /// Returns an error if the citation cannot be found.
    #[instrument(skip(self))]
    pub async fn resolve(&self, key: &str) -> Result<BibEntry> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(key) {
                debug!(%key, "Cache hit");
                return Ok(entry.clone());
            }
        }

        // Check local bibliographies
        for bib in &self.bibliographies {
            if let Some(entry) = bib.get(key) {
                debug!(%key, "Found in local bibliography");
                self.cache_entry(key, entry.clone()).await;
                return Ok(entry.clone());
            }
        }

        // Try CrossRef if key looks like a DOI
        if Self::looks_like_doi(key) {
            if let Some(ref client) = self.crossref {
                debug!(%key, "Trying CrossRef lookup");
                let work = client.get_by_doi(key).await?;
                let entry = Self::crossref_to_bibentry(key, &work);
                self.cache_entry(key, entry.clone()).await;
                return Ok(entry);
            }
        }

        Err(Error::NotFound {
            key: key.to_string(),
        })
    }

    /// Resolve multiple citations.
    ///
    /// Returns results in the same order as input keys.
    pub async fn resolve_batch(&self, keys: &[&str]) -> Vec<Result<BibEntry>> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.resolve(key).await);
        }
        results
    }

    /// Format a citation using the default style.
    ///
    /// # Errors
    ///
    /// Returns an error if formatting fails.
    pub fn format(&self, entry: &BibEntry) -> Result<String> {
        self.formatter.format_entry(entry)
    }

    /// Format a citation using a specific style.
    ///
    /// # Errors
    ///
    /// Returns an error if the style is not supported.
    pub fn format_with_style(&self, entry: &BibEntry, style: &str) -> Result<String> {
        let style = CitationStyle::from_name(style)?;
        let formatter = Formatter::new().with_style(style);
        formatter.format_entry(entry)
    }

    /// Cache an entry.
    async fn cache_entry(&self, key: &str, entry: BibEntry) {
        let mut cache = self.cache.write().await;
        cache.insert(key.to_string(), entry);
    }

    /// Check if a string looks like a DOI.
    fn looks_like_doi(key: &str) -> bool {
        key.starts_with("10.")
            || key.starts_with("https://doi.org/")
            || key.starts_with("doi:")
    }

    /// Convert CrossRef work to BibEntry.
    fn crossref_to_bibentry(key: &str, work: &crate::crossref::CrossRefWork) -> BibEntry {
        let mut fields = HashMap::new();

        if let Some(title) = work.primary_title() {
            fields.insert("title".to_string(), title.to_string());
        }

        if !work.author.is_empty() {
            let authors = work
                .author
                .iter()
                .map(|a| a.format_full())
                .collect::<Vec<_>>()
                .join(" and ");
            fields.insert("author".to_string(), authors);
        }

        if let Some(year) = work.year() {
            fields.insert("year".to_string(), year.to_string());
        }

        if let Some(journal) = work.journal() {
            fields.insert("journal".to_string(), journal.to_string());
        }

        if let Some(ref vol) = work.volume {
            fields.insert("volume".to_string(), vol.clone());
        }

        if let Some(ref issue) = work.issue {
            fields.insert("number".to_string(), issue.clone());
        }

        if let Some(ref pages) = work.page {
            fields.insert("pages".to_string(), pages.clone());
        }

        fields.insert("doi".to_string(), work.doi.clone());

        let entry_type = match work.work_type.as_str() {
            "journal-article" => "article",
            "book" => "book",
            "book-chapter" => "inbook",
            "proceedings-article" => "inproceedings",
            _ => "misc",
        };

        BibEntry {
            entry_type: entry_type.to_string(),
            key: key.to_string(),
            fields,
        }
    }

    /// Clear the citation cache.
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get the number of cached entries.
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

impl Default for CitationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bibliography() -> Bibliography {
        let bibtex = r#"
@article{test2023,
    author = {Test Author},
    title = {Test Article},
    journal = {Test Journal},
    year = {2023}
}
"#;
        Bibliography::parse(bibtex).unwrap()
    }

    #[tokio::test]
    async fn resolve_from_local_bibliography() {
        let manager = CitationManager::new().with_bibliography(sample_bibliography());

        let entry = manager.resolve("test2023").await.unwrap();
        assert_eq!(entry.key, "test2023");
        assert_eq!(entry.title(), Some("Test Article"));
    }

    #[tokio::test]
    async fn resolve_not_found() {
        let manager = CitationManager::new();
        let result = manager.resolve("nonexistent").await;
        assert!(matches!(result, Err(Error::NotFound { .. })));
    }

    #[tokio::test]
    async fn caching_works() {
        let manager = CitationManager::new().with_bibliography(sample_bibliography());

        // First resolve
        let _ = manager.resolve("test2023").await.unwrap();
        assert_eq!(manager.cache_size().await, 1);

        // Second resolve should hit cache
        let _ = manager.resolve("test2023").await.unwrap();
        assert_eq!(manager.cache_size().await, 1);

        // Clear cache
        manager.clear_cache().await;
        assert_eq!(manager.cache_size().await, 0);
    }

    #[test]
    fn looks_like_doi() {
        assert!(CitationManager::looks_like_doi("10.1234/test"));
        assert!(CitationManager::looks_like_doi("https://doi.org/10.1234/test"));
        assert!(CitationManager::looks_like_doi("doi:10.1234/test"));
        assert!(!CitationManager::looks_like_doi("smith2023"));
    }

    #[tokio::test]
    async fn format_with_default_style() {
        let manager = CitationManager::new()
            .with_bibliography(sample_bibliography())
            .with_style(CitationStyle::Ieee);

        let entry = manager.resolve("test2023").await.unwrap();
        let formatted = manager.format(&entry).unwrap();
        assert!(formatted.contains("Test Article"));
    }
}
