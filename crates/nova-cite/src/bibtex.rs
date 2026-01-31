//! BibTeX file parsing.
//!
//! Parses BibTeX and BibLaTeX bibliography files into structured data.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// A parsed BibTeX bibliography.
#[derive(Debug, Default, Clone)]
pub struct Bibliography {
    /// All entries indexed by key.
    entries: HashMap<String, BibEntry>,
}

impl Bibliography {
    /// Create a new empty bibliography.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a BibTeX string.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn parse(content: &str) -> Result<Self> {
        let mut bib = Self::new();
        let mut current_entry: Option<PartialEntry> = None;
        let mut brace_depth = 0;
        let mut in_field_value = false;
        let mut field_value = String::new();
        let mut current_field_name = String::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('%') {
                continue;
            }

            // Check for entry start: @type{key,
            if line.starts_with('@') && current_entry.is_none() {
                if let Some(entry) = Self::parse_entry_start(line, line_num)? {
                    current_entry = Some(entry);
                    brace_depth = 1;
                }
                continue;
            }

            // Inside an entry
            if let Some(ref mut entry) = current_entry {
                // Count braces
                for ch in line.chars() {
                    match ch {
                        '{' => brace_depth += 1,
                        '}' => brace_depth -= 1,
                        _ => {}
                    }
                }

                // Entry closed
                if brace_depth == 0 {
                    let finished = current_entry.take().unwrap();
                    bib.entries.insert(finished.key.clone(), finished.into());
                    continue;
                }

                // Parse field
                if !in_field_value {
                    if let Some((name, value, complete)) = Self::parse_field_start(line) {
                        if complete {
                            entry.fields.insert(name, value);
                        } else {
                            current_field_name = name;
                            field_value = value;
                            in_field_value = true;
                        }
                    }
                } else {
                    // Continue multi-line field
                    field_value.push(' ');
                    field_value.push_str(line.trim_end_matches(','));

                    if line.ends_with(',') || line.ends_with('}') {
                        entry.fields.insert(
                            std::mem::take(&mut current_field_name),
                            Self::clean_field_value(&field_value),
                        );
                        field_value.clear();
                        in_field_value = false;
                    }
                }
            }
        }

        Ok(bib)
    }

    /// Parse entry start line.
    fn parse_entry_start(line: &str, line_num: usize) -> Result<Option<PartialEntry>> {
        // Format: @type{key,
        let line = &line[1..]; // Skip @

        let type_end = line.find('{').ok_or_else(|| Error::BibtexParse {
            message: "missing opening brace".to_string(),
            line: Some(line_num + 1),
        })?;

        let entry_type = line[..type_end].trim().to_lowercase();
        let rest = &line[type_end + 1..];

        let key_end = rest.find(',').unwrap_or(rest.len());
        let key = rest[..key_end].trim().to_string();

        if key.is_empty() {
            return Err(Error::BibtexParse {
                message: "empty citation key".to_string(),
                line: Some(line_num + 1),
            });
        }

        Ok(Some(PartialEntry {
            entry_type,
            key,
            fields: HashMap::new(),
        }))
    }

    /// Parse a field line.
    fn parse_field_start(line: &str) -> Option<(String, String, bool)> {
        let eq_pos = line.find('=')?;
        let name = line[..eq_pos].trim().to_lowercase();
        let value_part = line[eq_pos + 1..].trim();

        let value = Self::clean_field_value(value_part.trim_end_matches(','));
        let complete = value_part.ends_with(',') || value_part.ends_with('}');

        Some((name, value, complete))
    }

    /// Clean field value (remove braces/quotes).
    fn clean_field_value(value: &str) -> String {
        let value = value.trim();
        let value = value.strip_prefix('{').unwrap_or(value);
        let value = value.strip_suffix('}').unwrap_or(value);
        let value = value.strip_prefix('"').unwrap_or(value);
        let value = value.strip_suffix('"').unwrap_or(value);
        value.trim().to_string()
    }

    /// Load bibliography from a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Get an entry by key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&BibEntry> {
        self.entries.get(key)
    }

    /// Check if a key exists.
    #[must_use]
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Get the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the bibliography is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all entries.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &BibEntry)> {
        self.entries.iter()
    }
}

/// Partial entry during parsing.
struct PartialEntry {
    entry_type: String,
    key: String,
    fields: HashMap<String, String>,
}

impl From<PartialEntry> for BibEntry {
    fn from(partial: PartialEntry) -> Self {
        Self {
            entry_type: partial.entry_type,
            key: partial.key,
            fields: partial.fields,
        }
    }
}

/// A single BibTeX entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BibEntry {
    /// Entry type (article, book, etc.).
    pub entry_type: String,

    /// Citation key.
    pub key: String,

    /// All fields.
    pub fields: HashMap<String, String>,
}

impl BibEntry {
    /// Get a field value.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields.get(name).map(String::as_str)
    }

    /// Get the title.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.field("title")
    }

    /// Get the author(s).
    #[must_use]
    pub fn author(&self) -> Option<&str> {
        self.field("author")
    }

    /// Get the year.
    #[must_use]
    pub fn year(&self) -> Option<&str> {
        self.field("year")
    }

    /// Get the DOI.
    #[must_use]
    pub fn doi(&self) -> Option<&str> {
        self.field("doi")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_BIBTEX: &str = r#"
@article{smith2023,
    author = {John Smith and Jane Doe},
    title = {A Sample Article},
    journal = {Journal of Examples},
    year = {2023},
    volume = {10},
    pages = {1-15},
    doi = {10.1234/example}
}

@book{knuth1984,
    author = {Donald E. Knuth},
    title = {The TeXbook},
    publisher = {Addison-Wesley},
    year = {1984}
}
"#;

    #[test]
    fn parse_bibtex() {
        let bib = Bibliography::parse(SAMPLE_BIBTEX).unwrap();
        assert_eq!(bib.len(), 2);
        assert!(bib.contains("smith2023"));
        assert!(bib.contains("knuth1984"));
    }

    #[test]
    fn get_entry_fields() {
        let bib = Bibliography::parse(SAMPLE_BIBTEX).unwrap();
        let entry = bib.get("smith2023").unwrap();

        assert_eq!(entry.entry_type, "article");
        assert_eq!(entry.title(), Some("A Sample Article"));
        assert_eq!(entry.author(), Some("John Smith and Jane Doe"));
        assert_eq!(entry.year(), Some("2023"));
        assert_eq!(entry.doi(), Some("10.1234/example"));
    }

    #[test]
    fn empty_bibliography() {
        let bib = Bibliography::new();
        assert!(bib.is_empty());
        assert_eq!(bib.len(), 0);
    }

    #[test]
    fn iterate_entries() {
        let bib = Bibliography::parse(SAMPLE_BIBTEX).unwrap();
        let keys: Vec<_> = bib.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"smith2023"));
        assert!(keys.contains(&"knuth1984"));
    }
}
