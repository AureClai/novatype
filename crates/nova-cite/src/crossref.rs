//! CrossRef API integration.
//!
//! Provides citation lookup via the CrossRef REST API.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

/// CrossRef API client.
#[derive(Debug, Clone)]
pub struct CrossRefClient {
    /// HTTP client.
    client: reqwest::Client,

    /// Base API URL.
    base_url: String,

    /// Optional polite pool email.
    mailto: Option<String>,
}

impl CrossRefClient {
    /// Create a new CrossRef client.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.crossref.org".to_string(),
            mailto: None,
        }
    }

    /// Set the mailto parameter for polite pool access.
    ///
    /// CrossRef provides faster responses for requests that include
    /// an email address.
    #[must_use]
    pub fn with_mailto(mut self, email: impl Into<String>) -> Self {
        self.mailto = Some(email.into());
        self
    }

    /// Look up a work by DOI.
    ///
    /// # Arguments
    ///
    /// * `doi` - The DOI to look up (with or without prefix)
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the DOI is not found.
    #[instrument(skip(self))]
    pub async fn get_by_doi(&self, doi: &str) -> Result<CrossRefWork> {
        let doi = Self::normalize_doi(doi);
        debug!(%doi, "Looking up DOI");

        let url = format!("{}/works/{}", self.base_url, doi);
        let mut request = self.client.get(&url);

        if let Some(ref email) = self.mailto {
            request = request.query(&[("mailto", email)]);
        }

        let response = request.send().await?;

        match response.status() {
            status if status.is_success() => {
                let wrapper: CrossRefResponse<CrossRefWork> = response.json().await?;
                Ok(wrapper.message)
            }
            reqwest::StatusCode::NOT_FOUND => Err(Error::NotFound {
                key: doi.to_string(),
            }),
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60);
                Err(Error::RateLimited {
                    retry_after_secs: retry_after,
                })
            }
            status => Err(Error::Api {
                message: format!("unexpected status: {status}"),
                status_code: Some(status.as_u16()),
            }),
        }
    }

    /// Search for works by query.
    ///
    /// # Arguments
    ///
    /// * `query` - Search query
    /// * `limit` - Maximum number of results
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    #[instrument(skip(self))]
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<CrossRefWork>> {
        debug!(%query, %limit, "Searching CrossRef");

        let url = format!("{}/works", self.base_url);
        let mut request = self
            .client
            .get(&url)
            .query(&[("query", query), ("rows", &limit.to_string())]);

        if let Some(ref email) = self.mailto {
            request = request.query(&[("mailto", email)]);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Error::Api {
                message: format!("search failed: {}", response.status()),
                status_code: Some(response.status().as_u16()),
            });
        }

        let wrapper: CrossRefResponse<CrossRefList> = response.json().await?;
        Ok(wrapper.message.items)
    }

    /// Normalize a DOI (remove URL prefix if present).
    fn normalize_doi(doi: &str) -> &str {
        doi.strip_prefix("https://doi.org/")
            .or_else(|| doi.strip_prefix("http://doi.org/"))
            .or_else(|| doi.strip_prefix("doi:"))
            .unwrap_or(doi)
    }
}

impl Default for CrossRefClient {
    fn default() -> Self {
        Self::new()
    }
}

/// CrossRef API response wrapper.
#[derive(Debug, Deserialize)]
struct CrossRefResponse<T> {
    #[allow(dead_code)]
    status: String,
    message: T,
}

/// CrossRef list response.
#[derive(Debug, Deserialize)]
struct CrossRefList {
    items: Vec<CrossRefWork>,
}

/// A work from CrossRef.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRefWork {
    /// DOI.
    #[serde(rename = "DOI")]
    pub doi: String,

    /// Work type (journal-article, book, etc.).
    #[serde(rename = "type")]
    pub work_type: String,

    /// Title(s).
    #[serde(default)]
    pub title: Vec<String>,

    /// Authors.
    #[serde(default)]
    pub author: Vec<CrossRefAuthor>,

    /// Container title (journal name).
    #[serde(rename = "container-title", default)]
    pub container_title: Vec<String>,

    /// Publisher.
    #[serde(default)]
    pub publisher: Option<String>,

    /// Publication date.
    #[serde(default)]
    pub published: Option<CrossRefDate>,

    /// Volume.
    #[serde(default)]
    pub volume: Option<String>,

    /// Issue.
    #[serde(default)]
    pub issue: Option<String>,

    /// Page range.
    #[serde(default)]
    pub page: Option<String>,

    /// ISSN.
    #[serde(rename = "ISSN", default)]
    pub issn: Vec<String>,

    /// Abstract.
    #[serde(rename = "abstract", default)]
    pub abstract_text: Option<String>,
}

impl CrossRefWork {
    /// Get the primary title.
    #[must_use]
    pub fn primary_title(&self) -> Option<&str> {
        self.title.first().map(String::as_str)
    }

    /// Get the publication year.
    #[must_use]
    pub fn year(&self) -> Option<u16> {
        self.published
            .as_ref()
            .and_then(|d| d.date_parts.first())
            .and_then(|parts| parts.first())
            .and_then(|&y| u16::try_from(y).ok())
    }

    /// Get the primary container title (journal name).
    #[must_use]
    pub fn journal(&self) -> Option<&str> {
        self.container_title.first().map(String::as_str)
    }
}

/// CrossRef author.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRefAuthor {
    /// Given name.
    #[serde(default)]
    pub given: Option<String>,

    /// Family name.
    #[serde(default)]
    pub family: Option<String>,

    /// ORCID.
    #[serde(rename = "ORCID", default)]
    pub orcid: Option<String>,
}

impl CrossRefAuthor {
    /// Format as "Family, Given".
    #[must_use]
    pub fn format_full(&self) -> String {
        match (&self.family, &self.given) {
            (Some(f), Some(g)) => format!("{f}, {g}"),
            (Some(f), None) => f.clone(),
            (None, Some(g)) => g.clone(),
            (None, None) => String::new(),
        }
    }
}

/// CrossRef date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRefDate {
    /// Date parts: [[year, month, day]].
    #[serde(rename = "date-parts", default)]
    pub date_parts: Vec<Vec<i32>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_doi_variants() {
        assert_eq!(
            CrossRefClient::normalize_doi("https://doi.org/10.1234/test"),
            "10.1234/test"
        );
        assert_eq!(
            CrossRefClient::normalize_doi("http://doi.org/10.1234/test"),
            "10.1234/test"
        );
        assert_eq!(
            CrossRefClient::normalize_doi("doi:10.1234/test"),
            "10.1234/test"
        );
        assert_eq!(
            CrossRefClient::normalize_doi("10.1234/test"),
            "10.1234/test"
        );
    }

    #[test]
    fn crossref_author_format() {
        let author = CrossRefAuthor {
            given: Some("John".to_string()),
            family: Some("Doe".to_string()),
            orcid: None,
        };
        assert_eq!(author.format_full(), "Doe, John");

        let family_only = CrossRefAuthor {
            given: None,
            family: Some("Doe".to_string()),
            orcid: None,
        };
        assert_eq!(family_only.format_full(), "Doe");
    }

    #[test]
    fn crossref_work_accessors() {
        let work = CrossRefWork {
            doi: "10.1234/test".to_string(),
            work_type: "journal-article".to_string(),
            title: vec!["Test Article".to_string()],
            author: vec![],
            container_title: vec!["Test Journal".to_string()],
            publisher: Some("Test Publisher".to_string()),
            published: Some(CrossRefDate {
                date_parts: vec![vec![2023, 5, 15]],
            }),
            volume: Some("10".to_string()),
            issue: Some("2".to_string()),
            page: Some("100-110".to_string()),
            issn: vec![],
            abstract_text: None,
        };

        assert_eq!(work.primary_title(), Some("Test Article"));
        assert_eq!(work.year(), Some(2023));
        assert_eq!(work.journal(), Some("Test Journal"));
    }

    #[test]
    fn client_with_mailto() {
        let client = CrossRefClient::new().with_mailto("test@example.com");
        assert_eq!(client.mailto, Some("test@example.com".to_string()));
    }
}
