//! Document representation and manipulation.
//!
//! This module defines the core [`Document`] type that represents
//! a NovaType document throughout the compilation pipeline.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a NovaType document.
///
/// A document consists of:
/// - Source content (Typst markup)
/// - Metadata (frontmatter)
/// - Associated resources (images, data files)
#[derive(Debug, Clone, Default)]
pub struct Document {
    /// Path to the source file, if loaded from disk.
    pub source_path: Option<PathBuf>,

    /// Raw source content.
    pub content: String,

    /// Parsed metadata from frontmatter.
    pub metadata: Metadata,

    /// Resolved dependencies (images, includes, data files).
    pub dependencies: Vec<Dependency>,
}

impl Document {
    /// Create a new empty document.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a document from source content.
    #[must_use]
    pub fn from_content(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }

    /// Create a document from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn from_file(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        Ok(Self {
            source_path: Some(path.to_path_buf()),
            content,
            ..Default::default()
        })
    }

    /// Check if the document has metadata.
    #[must_use]
    pub fn has_metadata(&self) -> bool {
        !self.metadata.is_empty()
    }

    /// Get the document title from metadata.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.metadata.title.as_deref()
    }

    /// Get the template name from metadata.
    #[must_use]
    pub fn template(&self) -> Option<&str> {
        self.metadata.template.as_deref()
    }
}

/// Document metadata extracted from frontmatter.
///
/// Supports common fields with additional custom fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    /// Document title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Document authors.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<AuthorMeta>,

    /// Document date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,

    /// Template to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// Citation style.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_style: Option<String>,

    /// Bibliography files.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bibliography: Vec<String>,

    /// Custom fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Metadata {
    /// Check if metadata is empty (no fields set).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.authors.is_empty()
            && self.date.is_none()
            && self.template.is_none()
            && self.citation_style.is_none()
            && self.bibliography.is_empty()
            && self.extra.is_empty()
    }
}

/// Author metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorMeta {
    /// Author name.
    pub name: String,

    /// Email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Affiliation/institution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affiliation: Option<String>,

    /// ORCID identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orcid: Option<String>,
}

/// A document dependency (resource).
#[derive(Debug, Clone)]
pub struct Dependency {
    /// Path to the dependency.
    pub path: PathBuf,

    /// Type of dependency.
    pub kind: DependencyKind,

    /// Whether the dependency has been resolved.
    pub resolved: bool,
}

/// Types of document dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyKind {
    /// Image file.
    Image,
    /// Data file (CSV, JSON).
    Data,
    /// Included Typst file.
    Include,
    /// Bibliography file.
    Bibliography,
    /// Font file.
    Font,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn document_new_is_empty() {
        let doc = Document::new();
        assert!(doc.content.is_empty());
        assert!(doc.source_path.is_none());
        assert!(!doc.has_metadata());
    }

    #[test]
    fn document_from_content() {
        let doc = Document::from_content("Hello, world!");
        assert_eq!(doc.content, "Hello, world!");
    }

    #[test]
    fn document_from_file() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Test content").unwrap();

        let doc = Document::from_file(temp.path()).unwrap();
        assert!(doc.content.contains("Test content"));
        assert!(doc.source_path.is_some());
    }

    #[test]
    fn metadata_empty_check() {
        let meta = Metadata::default();
        assert!(meta.is_empty());

        let meta_with_title = Metadata {
            title: Some("Test".to_string()),
            ..Default::default()
        };
        assert!(!meta_with_title.is_empty());
    }

    #[test]
    fn document_title_accessor() {
        let mut doc = Document::new();
        assert!(doc.title().is_none());

        doc.metadata.title = Some("My Document".to_string());
        assert_eq!(doc.title(), Some("My Document"));
    }

    #[test]
    fn metadata_serialization() {
        let meta = Metadata {
            title: Some("Test Document".to_string()),
            authors: vec![AuthorMeta {
                name: "John Doe".to_string(),
                email: Some("john@example.com".to_string()),
                affiliation: None,
                orcid: None,
            }],
            ..Default::default()
        };

        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("Test Document"));
        assert!(json.contains("John Doe"));
    }
}
