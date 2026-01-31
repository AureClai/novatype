//! Frontmatter parsing for document metadata.
//!
//! Supports YAML and TOML frontmatter formats:
//!
//! ```text
//! ---
//! title: My Document
//! author: John Doe
//! ---
//! ```

use crate::error::{Error, Result};
use serde::de::DeserializeOwned;

/// Parser for document frontmatter.
#[derive(Debug, Clone, Default)]
pub struct FrontmatterParser {
    /// Strict mode fails on unknown fields.
    strict: bool,
}

impl FrontmatterParser {
    /// Create a new frontmatter parser.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable strict validation mode.
    #[must_use]
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Parse frontmatter from document content.
    ///
    /// Detects the frontmatter format (YAML or TOML) and parses it.
    ///
    /// # Returns
    ///
    /// A tuple of (parsed metadata, remaining content).
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn parse<'a, T: DeserializeOwned>(&self, content: &'a str) -> Result<(Option<T>, &'a str)> {
        let trimmed = content.trim_start();
        let offset = content.len() - trimmed.len();

        // Check for YAML frontmatter (---)
        if trimmed.starts_with("---") {
            return self.parse_yaml(&content[offset..]);
        }

        // Check for TOML frontmatter (+++)
        if trimmed.starts_with("+++") {
            return self.parse_toml(&content[offset..]);
        }

        // No frontmatter
        Ok((None, &content[offset..]))
    }

    /// Parse YAML frontmatter.
    fn parse_yaml<'a, T: DeserializeOwned>(&self, content: &'a str) -> Result<(Option<T>, &'a str)> {
        let inner = &content[3..]; // Skip opening ---

        let end = inner
            .find("\n---")
            .ok_or_else(|| Error::FrontmatterParse {
                message: "unclosed YAML frontmatter".to_string(),
                line: None,
            })?;

        let yaml_content = inner[..end].trim();
        let remaining_start = 3 + end + 4; // Skip "---" + content + "\n---"
        let remaining = &content[remaining_start..];

        if yaml_content.is_empty() {
            let trimmed = remaining.trim_start();
            let offset = remaining.len() - trimmed.len();
            return Ok((None, &remaining[offset..]));
        }

        let parsed: T = serde_yaml::from_str(yaml_content)?;
        let trimmed = remaining.trim_start();
        let offset = remaining.len() - trimmed.len();
        Ok((Some(parsed), &remaining[offset..]))
    }

    /// Parse TOML frontmatter.
    fn parse_toml<'a, T: DeserializeOwned>(&self, content: &'a str) -> Result<(Option<T>, &'a str)> {
        let inner = &content[3..]; // Skip opening +++

        let end = inner
            .find("\n+++")
            .ok_or_else(|| Error::FrontmatterParse {
                message: "unclosed TOML frontmatter".to_string(),
                line: None,
            })?;

        let toml_content = inner[..end].trim();
        let remaining_start = 3 + end + 4; // Skip "+++" + content + "\n+++"
        let remaining = &content[remaining_start..];

        if toml_content.is_empty() {
            let trimmed = remaining.trim_start();
            let offset = remaining.len() - trimmed.len();
            return Ok((None, &remaining[offset..]));
        }

        let parsed: T = toml::from_str(toml_content)?;
        let trimmed = remaining.trim_start();
        let offset = remaining.len() - trimmed.len();
        Ok((Some(parsed), &remaining[offset..]))
    }

    /// Extract raw frontmatter string without parsing.
    ///
    /// Useful for validation before full parsing.
    #[must_use]
    pub fn extract_raw<'a>(&self, content: &'a str) -> Option<(&'a str, FrontmatterFormat)> {
        let content = content.trim_start();

        if content.starts_with("---") {
            let content = &content[3..];
            if let Some(end) = content.find("\n---") {
                return Some((&content[..end], FrontmatterFormat::Yaml));
            }
        }

        if content.starts_with("+++") {
            let content = &content[3..];
            if let Some(end) = content.find("\n+++") {
                return Some((&content[..end], FrontmatterFormat::Toml));
            }
        }

        None
    }
}

/// Frontmatter format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontmatterFormat {
    /// YAML format (---).
    Yaml,
    /// TOML format (+++).
    Toml,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestMeta {
        title: String,
        author: Option<String>,
    }

    #[test]
    fn parse_yaml_frontmatter() {
        let content = r#"---
title: Test Document
author: John Doe
---
Hello, world!"#;

        let parser = FrontmatterParser::new();
        let (meta, remaining): (Option<TestMeta>, _) = parser.parse(content).unwrap();

        let meta = meta.expect("should have metadata");
        assert_eq!(meta.title, "Test Document");
        assert_eq!(meta.author, Some("John Doe".to_string()));
        assert_eq!(remaining, "Hello, world!");
    }

    #[test]
    fn parse_toml_frontmatter() {
        let content = r#"+++
title = "Test Document"
author = "Jane Doe"
+++
Content here"#;

        let parser = FrontmatterParser::new();
        let (meta, remaining): (Option<TestMeta>, _) = parser.parse(content).unwrap();

        let meta = meta.expect("should have metadata");
        assert_eq!(meta.title, "Test Document");
        assert_eq!(meta.author, Some("Jane Doe".to_string()));
        assert_eq!(remaining, "Content here");
    }

    #[test]
    fn parse_no_frontmatter() {
        let content = "Just regular content";

        let parser = FrontmatterParser::new();
        let (meta, remaining): (Option<TestMeta>, _) = parser.parse(content).unwrap();

        assert!(meta.is_none());
        assert_eq!(remaining, "Just regular content");
    }

    #[test]
    fn parse_unclosed_frontmatter() {
        let content = "---\ntitle: Test\nNo closing delimiter";

        let parser = FrontmatterParser::new();
        let result: Result<(Option<TestMeta>, _)> = parser.parse(content);

        assert!(matches!(result, Err(Error::FrontmatterParse { .. })));
    }

    #[test]
    fn extract_raw_yaml() {
        let content = "---\ntitle: Test\n---\nContent";
        let parser = FrontmatterParser::new();

        let (raw, format) = parser.extract_raw(content).unwrap();
        assert_eq!(raw.trim(), "title: Test");
        assert_eq!(format, FrontmatterFormat::Yaml);
    }

    #[test]
    fn extract_raw_toml() {
        let content = "+++\ntitle = \"Test\"\n+++\nContent";
        let parser = FrontmatterParser::new();

        let (raw, format) = parser.extract_raw(content).unwrap();
        assert_eq!(raw.trim(), "title = \"Test\"");
        assert_eq!(format, FrontmatterFormat::Toml);
    }
}
