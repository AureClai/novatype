//! Citation formatting.
//!
//! Formats citations according to various style guides.

use crate::bibtex::BibEntry;
use crate::error::{Error, Result};

/// Citation style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CitationStyle {
    /// IEEE style.
    Ieee,
    /// APA 7th edition.
    Apa,
    /// Chicago author-date.
    Chicago,
    /// Harvard style.
    Harvard,
    /// Nature journal style.
    Nature,
    /// Vancouver (numeric).
    Vancouver,
}

impl CitationStyle {
    /// Parse a style name.
    ///
    /// # Errors
    ///
    /// Returns an error if the style is not recognized.
    pub fn from_name(name: &str) -> Result<Self> {
        match name.to_lowercase().as_str() {
            "ieee" => Ok(Self::Ieee),
            "apa" | "apa7" => Ok(Self::Apa),
            "chicago" => Ok(Self::Chicago),
            "harvard" => Ok(Self::Harvard),
            "nature" => Ok(Self::Nature),
            "vancouver" => Ok(Self::Vancouver),
            _ => Err(Error::UnsupportedStyle {
                style: name.to_string(),
            }),
        }
    }

    /// Get the style name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ieee => "ieee",
            Self::Apa => "apa",
            Self::Chicago => "chicago",
            Self::Harvard => "harvard",
            Self::Nature => "nature",
            Self::Vancouver => "vancouver",
        }
    }
}

/// Citation formatter.
#[derive(Debug, Default)]
pub struct Formatter {
    /// Current style.
    style: Option<CitationStyle>,
}

impl Formatter {
    /// Create a new formatter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the citation style.
    #[must_use]
    pub fn with_style(mut self, style: CitationStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Format a bibliography entry.
    ///
    /// # Errors
    ///
    /// Returns an error if the style is not set.
    pub fn format_entry(&self, entry: &BibEntry) -> Result<String> {
        let style = self.style.ok_or_else(|| Error::UnsupportedStyle {
            style: "none".to_string(),
        })?;

        match style {
            CitationStyle::Ieee => self.format_ieee(entry),
            CitationStyle::Apa => self.format_apa(entry),
            CitationStyle::Chicago => self.format_chicago(entry),
            CitationStyle::Harvard => self.format_harvard(entry),
            CitationStyle::Nature => self.format_nature(entry),
            CitationStyle::Vancouver => self.format_vancouver(entry),
        }
    }

    /// Format in IEEE style.
    fn format_ieee(&self, entry: &BibEntry) -> Result<String> {
        let mut parts = Vec::new();

        // Authors
        if let Some(authors) = entry.author() {
            let formatted = Self::format_authors_ieee(authors);
            parts.push(formatted);
        }

        // Title
        if let Some(title) = entry.title() {
            parts.push(format!("\"{title},\""));
        }

        // Journal/container
        if let Some(journal) = entry.field("journal") {
            parts.push(format!("*{journal}*"));
        }

        // Volume, issue, pages
        let mut loc_parts = Vec::new();
        if let Some(vol) = entry.field("volume") {
            loc_parts.push(format!("vol. {vol}"));
        }
        if let Some(num) = entry.field("number") {
            loc_parts.push(format!("no. {num}"));
        }
        if let Some(pages) = entry.field("pages") {
            loc_parts.push(format!("pp. {}", pages.replace("--", "–")));
        }
        if !loc_parts.is_empty() {
            parts.push(loc_parts.join(", "));
        }

        // Year
        if let Some(year) = entry.year() {
            parts.push(year.to_string());
        }

        // DOI
        if let Some(doi) = entry.doi() {
            parts.push(format!("doi: {doi}"));
        }

        Ok(parts.join(", ") + ".")
    }

    /// Format authors in IEEE style.
    fn format_authors_ieee(authors: &str) -> String {
        let authors: Vec<&str> = authors.split(" and ").collect();

        match authors.len() {
            0 => String::new(),
            1 => Self::format_single_author_ieee(authors[0]),
            2 => format!(
                "{} and {}",
                Self::format_single_author_ieee(authors[0]),
                Self::format_single_author_ieee(authors[1])
            ),
            _ => {
                let first = Self::format_single_author_ieee(authors[0]);
                format!("{first} et al.")
            }
        }
    }

    /// Format a single author in IEEE style (initials + last name).
    fn format_single_author_ieee(author: &str) -> String {
        let parts: Vec<&str> = author.split_whitespace().collect();
        if parts.len() >= 2 {
            let last = parts.last().unwrap();
            let initials: String = parts[..parts.len() - 1]
                .iter()
                .map(|p| format!("{}.", p.chars().next().unwrap_or(' ')))
                .collect::<Vec<_>>()
                .join(" ");
            format!("{initials} {last}")
        } else {
            author.to_string()
        }
    }

    /// Format in APA style.
    fn format_apa(&self, entry: &BibEntry) -> Result<String> {
        let mut parts = Vec::new();

        // Authors (Last, F. M.)
        if let Some(authors) = entry.author() {
            parts.push(Self::format_authors_apa(authors));
        }

        // Year
        if let Some(year) = entry.year() {
            parts.push(format!("({year})"));
        }

        // Title
        if let Some(title) = entry.title() {
            parts.push(format!("{title}."));
        }

        // Journal
        if let Some(journal) = entry.field("journal") {
            let mut journal_part = format!("*{journal}*");
            if let Some(vol) = entry.field("volume") {
                journal_part.push_str(&format!(", *{vol}*"));
                if let Some(num) = entry.field("number") {
                    journal_part.push_str(&format!("({num})"));
                }
            }
            if let Some(pages) = entry.field("pages") {
                journal_part.push_str(&format!(", {}", pages.replace("--", "–")));
            }
            parts.push(journal_part);
        }

        // DOI
        if let Some(doi) = entry.doi() {
            parts.push(format!("https://doi.org/{doi}"));
        }

        Ok(parts.join(" "))
    }

    /// Format authors in APA style.
    fn format_authors_apa(authors: &str) -> String {
        let authors: Vec<&str> = authors.split(" and ").collect();

        authors
            .iter()
            .map(|&a| Self::format_single_author_apa(a))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Format a single author in APA style (Last, F. M.).
    fn format_single_author_apa(author: &str) -> String {
        let parts: Vec<&str> = author.split_whitespace().collect();
        if parts.len() >= 2 {
            let last = parts.last().unwrap();
            let initials: String = parts[..parts.len() - 1]
                .iter()
                .map(|p| format!("{}.", p.chars().next().unwrap_or(' ')))
                .collect::<Vec<_>>()
                .join(" ");
            format!("{last}, {initials}")
        } else {
            author.to_string()
        }
    }

    /// Format in Chicago style.
    fn format_chicago(&self, entry: &BibEntry) -> Result<String> {
        // Simplified Chicago author-date
        let mut parts = Vec::new();

        if let Some(authors) = entry.author() {
            parts.push(authors.replace(" and ", ", and "));
        }

        if let Some(year) = entry.year() {
            parts.push(format!("{year}."));
        }

        if let Some(title) = entry.title() {
            parts.push(format!("\"{title}.\""));
        }

        if let Some(journal) = entry.field("journal") {
            parts.push(format!("*{journal}*"));
        }

        Ok(parts.join(" "))
    }

    /// Format in Harvard style.
    fn format_harvard(&self, entry: &BibEntry) -> Result<String> {
        // Similar to APA but with slight differences
        self.format_apa(entry) // Simplified
    }

    /// Format in Nature style.
    fn format_nature(&self, entry: &BibEntry) -> Result<String> {
        let mut parts = Vec::new();

        if let Some(authors) = entry.author() {
            parts.push(Self::format_authors_ieee(authors));
        }

        if let Some(title) = entry.title() {
            parts.push(format!("{title}."));
        }

        if let Some(journal) = entry.field("journal") {
            parts.push(format!("*{journal}*"));
        }

        if let Some(vol) = entry.field("volume") {
            parts.push(format!("**{vol}**,"));
        }

        if let Some(pages) = entry.field("pages") {
            parts.push(pages.replace("--", "–"));
        }

        if let Some(year) = entry.year() {
            parts.push(format!("({year})."));
        }

        Ok(parts.join(" "))
    }

    /// Format in Vancouver style.
    fn format_vancouver(&self, entry: &BibEntry) -> Result<String> {
        // Numbered style, similar to IEEE
        self.format_ieee(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn sample_entry() -> BibEntry {
        let mut fields = HashMap::new();
        fields.insert("author".to_string(), "John Smith and Jane Doe".to_string());
        fields.insert("title".to_string(), "A Sample Article".to_string());
        fields.insert(
            "journal".to_string(),
            "Journal of Computer Science".to_string(),
        );
        fields.insert("year".to_string(), "2023".to_string());
        fields.insert("volume".to_string(), "10".to_string());
        fields.insert("number".to_string(), "2".to_string());
        fields.insert("pages".to_string(), "100--115".to_string());
        fields.insert("doi".to_string(), "10.1234/example".to_string());

        BibEntry {
            entry_type: "article".to_string(),
            key: "smith2023".to_string(),
            fields,
        }
    }

    #[test]
    fn citation_style_from_name() {
        assert_eq!(
            CitationStyle::from_name("ieee").unwrap(),
            CitationStyle::Ieee
        );
        assert_eq!(CitationStyle::from_name("APA").unwrap(), CitationStyle::Apa);
        assert!(CitationStyle::from_name("invalid").is_err());
    }

    #[test]
    fn format_ieee_style() {
        let formatter = Formatter::new().with_style(CitationStyle::Ieee);
        let entry = sample_entry();
        let result = formatter.format_entry(&entry).unwrap();

        assert!(result.contains("J. Smith"));
        assert!(result.contains("J. Doe"));
        assert!(result.contains("A Sample Article"));
        assert!(result.contains("vol. 10"));
    }

    #[test]
    fn format_apa_style() {
        let formatter = Formatter::new().with_style(CitationStyle::Apa);
        let entry = sample_entry();
        let result = formatter.format_entry(&entry).unwrap();

        assert!(result.contains("Smith, J."));
        assert!(result.contains("(2023)"));
    }

    #[test]
    fn format_without_style_fails() {
        let formatter = Formatter::new();
        let entry = sample_entry();
        assert!(formatter.format_entry(&entry).is_err());
    }

    #[test]
    fn format_authors_ieee_multiple() {
        let result = Formatter::format_authors_ieee("John Smith and Jane Doe and Bob Wilson");
        assert_eq!(result, "J. Smith et al.");
    }
}
