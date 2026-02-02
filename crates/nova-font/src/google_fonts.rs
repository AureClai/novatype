//! Google Fonts integration.
//!
//! Provides font discovery and download from Google Fonts.
//! Uses the CSS API which doesn't require an API key.

use crate::error::{Error, Result};
use crate::FontCategory;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, instrument};

/// Google Fonts CSS API base URL.
const CSS_API_URL: &str = "https://fonts.googleapis.com/css2";

/// User agent to request TTF fonts (not WOFF2).
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";

/// Google Fonts client.
#[derive(Debug, Clone)]
pub struct GoogleFontsClient {
    /// HTTP client.
    client: reqwest::Client,
}

impl GoogleFontsClient {
    /// Create a new Google Fonts client.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    /// Set the API key (kept for compatibility but not used with CSS API).
    #[must_use]
    pub fn with_api_key(self, _key: impl Into<String>) -> Self {
        // API key not needed for CSS API
        self
    }

    /// Get font information by fetching CSS and parsing it.
    ///
    /// # Errors
    ///
    /// Returns an error if the font is not found or the request fails.
    #[instrument(skip(self))]
    pub async fn get_font(&self, name: &str) -> Result<FontFamily> {
        debug!(font = %name, "Fetching font info");

        // Build CSS URL with all weights
        let weights = "100;200;300;400;500;600;700;800;900";
        let family_encoded = name.replace(' ', "+");

        // Try regular weights first
        let url = format!(
            "{}?family={}:wght@{}&display=swap",
            CSS_API_URL, family_encoded, weights
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            // Try without weight specification (for variable fonts or simple fonts)
            let simple_url = format!("{}?family={}&display=swap", CSS_API_URL, family_encoded);
            let simple_response = self.client.get(&simple_url).send().await?;

            if !simple_response.status().is_success() {
                return Err(Error::FontNotFound {
                    name: name.to_string(),
                });
            }

            let css = simple_response.text().await?;
            return self.parse_css_to_family(name, &css);
        }

        let css = response.text().await?;

        // Also try to get italic variants
        let italic_url = format!(
            "{}?family={}:ital,wght@0,{};1,{}&display=swap",
            CSS_API_URL, family_encoded, weights, weights
        );

        let italic_css = if let Ok(resp) = self.client.get(&italic_url).send().await {
            if resp.status().is_success() {
                resp.text().await.ok()
            } else {
                None
            }
        } else {
            None
        };

        let combined_css = if let Some(italic) = italic_css {
            format!("{}\n{}", css, italic)
        } else {
            css
        };

        self.parse_css_to_family(name, &combined_css)
    }

    /// Parse CSS response to extract font family information.
    fn parse_css_to_family(&self, name: &str, css: &str) -> Result<FontFamily> {
        let mut files = HashMap::new();
        let mut variants = Vec::new();

        // Parse @font-face blocks
        for block in css.split("@font-face") {
            if block.trim().is_empty() {
                continue;
            }

            // Extract weight
            let weight = self.extract_css_value(block, "font-weight")
                .and_then(|w| w.parse::<u16>().ok())
                .unwrap_or(400);

            // Extract style
            let is_italic = self.extract_css_value(block, "font-style")
                .map(|s| s == "italic")
                .unwrap_or(false);

            // Extract URL
            if let Some(url) = self.extract_font_url(block) {
                let variant = match (weight, is_italic) {
                    (400, false) => "regular".to_string(),
                    (400, true) => "italic".to_string(),
                    (w, false) => w.to_string(),
                    (w, true) => format!("{}italic", w),
                };

                if !variants.contains(&variant) {
                    variants.push(variant.clone());
                    files.insert(variant, url);
                }
            }
        }

        if files.is_empty() {
            return Err(Error::FontNotFound {
                name: name.to_string(),
            });
        }

        // Sort variants
        variants.sort_by(|a, b| {
            let weight_a = Self::variant_sort_key(a);
            let weight_b = Self::variant_sort_key(b);
            weight_a.cmp(&weight_b)
        });

        Ok(FontFamily {
            family: name.to_string(),
            category: None, // CSS API doesn't provide category
            variants,
            subsets: vec!["latin".to_string()],
            version: None,
            last_modified: None,
            files,
        })
    }

    /// Extract a CSS property value.
    fn extract_css_value(&self, block: &str, property: &str) -> Option<String> {
        let pattern = format!("{}:", property);
        let start = block.find(&pattern)?;
        let rest = &block[start + pattern.len()..];
        let end = rest.find([';', '}'])?;
        Some(rest[..end].trim().to_string())
    }

    /// Extract font URL from CSS src property.
    fn extract_font_url(&self, block: &str) -> Option<String> {
        // Look for url() in src property
        let src_start = block.find("src:")?;
        let src_block = &block[src_start..];

        // Find url(...)
        let url_start = src_block.find("url(")?;
        let url_rest = &src_block[url_start + 4..];
        let url_end = url_rest.find(')')?;

        let url = url_rest[..url_end].trim().trim_matches('"').trim_matches('\'');
        Some(url.to_string())
    }

    /// Get sort key for variant ordering.
    fn variant_sort_key(variant: &str) -> (u16, bool) {
        if variant == "regular" {
            (400, false)
        } else if variant == "italic" {
            (400, true)
        } else if variant.ends_with("italic") {
            let weight: u16 = variant.trim_end_matches("italic").parse().unwrap_or(400);
            (weight, true)
        } else {
            let weight: u16 = variant.parse().unwrap_or(400);
            (weight, false)
        }
    }

    /// Search fonts by name (searches popular fonts).
    ///
    /// Note: Without API key, we use a curated list of popular fonts.
    #[instrument(skip(self))]
    pub async fn search(&self, query: &str) -> Result<Vec<FontFamily>> {
        let query_lower = query.to_lowercase();

        // Filter popular fonts by query
        let matching: Vec<_> = POPULAR_FONTS
            .iter()
            .filter(|(name, _)| name.to_lowercase().contains(&query_lower))
            .collect();

        let mut results = Vec::new();
        for (name, category) in matching.iter().take(10) {
            if let Ok(mut family) = self.get_font(name).await {
                family.category = Some(category.to_string());
                results.push(family);
            }
        }

        Ok(results)
    }

    /// Search fonts by category.
    #[instrument(skip(self))]
    pub async fn search_by_category(&self, category: FontCategory) -> Result<Vec<FontFamily>> {
        let category_str = category.as_str();

        let matching: Vec<_> = POPULAR_FONTS
            .iter()
            .filter(|(_, cat)| *cat == category_str)
            .collect();

        let mut results = Vec::new();
        for (name, cat) in matching.iter().take(10) {
            if let Ok(mut family) = self.get_font(name).await {
                family.category = Some(cat.to_string());
                results.push(family);
            }
        }

        Ok(results)
    }

    /// Download a font file.
    #[instrument(skip(self))]
    pub async fn download_font(&self, url: &str) -> Result<Vec<u8>> {
        debug!(%url, "Downloading font");

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(Error::DownloadFailed {
                font_name: url.to_string(),
                reason: format!("HTTP {}", response.status()),
            });
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

impl Default for GoogleFontsClient {
    fn default() -> Self {
        Self::new()
    }
}

/// A font family from Google Fonts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamily {
    /// Font family name.
    pub family: String,

    /// Font category (serif, sans-serif, etc.).
    pub category: Option<String>,

    /// Available variants (regular, bold, italic, etc.).
    #[serde(default)]
    pub variants: Vec<String>,

    /// Available subsets (latin, cyrillic, etc.).
    #[serde(default)]
    pub subsets: Vec<String>,

    /// Version string.
    pub version: Option<String>,

    /// Last modified date.
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,

    /// Font files by variant.
    #[serde(default)]
    pub files: HashMap<String, String>,
}

impl FontFamily {
    /// Get the category as an enum.
    #[must_use]
    pub fn category_enum(&self) -> Option<FontCategory> {
        self.category.as_deref().and_then(FontCategory::from_str)
    }

    /// Check if the font has a specific variant.
    #[must_use]
    pub fn has_variant(&self, variant: &str) -> bool {
        self.variants.iter().any(|v| v == variant)
    }

    /// Get the URL for a specific variant.
    #[must_use]
    pub fn file_url(&self, variant: &str) -> Option<&str> {
        self.files.get(variant).map(String::as_str)
    }

    /// Get all available variants as structured data.
    #[must_use]
    pub fn parsed_variants(&self) -> Vec<FontVariant> {
        self.variants
            .iter()
            .filter_map(|v| FontVariant::parse(v))
            .collect()
    }

    /// Check if the font is monospace.
    #[must_use]
    pub fn is_monospace(&self) -> bool {
        self.category.as_deref() == Some("monospace")
    }

    /// Check if the font supports a specific subset.
    #[must_use]
    pub fn supports_subset(&self, subset: &str) -> bool {
        self.subsets.iter().any(|s| s == subset)
    }
}

/// A parsed font variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontVariant {
    /// Weight (100-900).
    pub weight: u16,

    /// Whether this is an italic variant.
    pub italic: bool,
}

impl FontVariant {
    /// Parse a variant string like "regular", "700", "700italic".
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();

        if s == "regular" {
            return Some(Self {
                weight: 400,
                italic: false,
            });
        }

        if s == "italic" {
            return Some(Self {
                weight: 400,
                italic: true,
            });
        }

        let italic = s.ends_with("italic");
        let weight_str = if italic {
            s.trim_end_matches("italic")
        } else {
            &s
        };

        let weight = if weight_str.is_empty() {
            400
        } else {
            weight_str.parse().ok()?
        };

        Some(Self { weight, italic })
    }

    /// Convert to a variant string.
    #[must_use]
    pub fn to_string(&self) -> String {
        match (self.weight, self.italic) {
            (400, false) => "regular".to_string(),
            (400, true) => "italic".to_string(),
            (w, false) => w.to_string(),
            (w, true) => format!("{}italic", w),
        }
    }
}

/// Popular fonts with their categories (for search without API key).
const POPULAR_FONTS: &[(&str, &str)] = &[
    // Sans-serif
    ("Inter", "sans-serif"),
    ("Roboto", "sans-serif"),
    ("Open Sans", "sans-serif"),
    ("Lato", "sans-serif"),
    ("Montserrat", "sans-serif"),
    ("Poppins", "sans-serif"),
    ("Nunito", "sans-serif"),
    ("Raleway", "sans-serif"),
    ("Ubuntu", "sans-serif"),
    ("Noto Sans", "sans-serif"),
    ("Work Sans", "sans-serif"),
    ("Quicksand", "sans-serif"),
    ("Barlow", "sans-serif"),
    ("Rubik", "sans-serif"),
    ("Mulish", "sans-serif"),
    ("DM Sans", "sans-serif"),
    ("Manrope", "sans-serif"),
    ("Outfit", "sans-serif"),
    ("Plus Jakarta Sans", "sans-serif"),
    ("Space Grotesk", "sans-serif"),

    // Serif
    ("Playfair Display", "serif"),
    ("Merriweather", "serif"),
    ("Lora", "serif"),
    ("PT Serif", "serif"),
    ("Source Serif Pro", "serif"),
    ("Libre Baskerville", "serif"),
    ("EB Garamond", "serif"),
    ("Crimson Text", "serif"),
    ("Cormorant Garamond", "serif"),
    ("Noto Serif", "serif"),
    ("IBM Plex Serif", "serif"),
    ("Bitter", "serif"),
    ("Vollkorn", "serif"),
    ("Spectral", "serif"),
    ("Newsreader", "serif"),

    // Monospace
    ("JetBrains Mono", "monospace"),
    ("Fira Code", "monospace"),
    ("Source Code Pro", "monospace"),
    ("Roboto Mono", "monospace"),
    ("IBM Plex Mono", "monospace"),
    ("Ubuntu Mono", "monospace"),
    ("Space Mono", "monospace"),
    ("Inconsolata", "monospace"),
    ("Cousine", "monospace"),
    ("Anonymous Pro", "monospace"),
    ("PT Mono", "monospace"),
    ("DM Mono", "monospace"),

    // Display
    ("Bebas Neue", "display"),
    ("Oswald", "display"),
    ("Anton", "display"),
    ("Archivo Black", "display"),
    ("Righteous", "display"),
    ("Titan One", "display"),
    ("Fredoka One", "display"),
    ("Bungee", "display"),
    ("Lobster", "display"),
    ("Pacifico", "display"),

    // Handwriting
    ("Dancing Script", "handwriting"),
    ("Pacifico", "handwriting"),
    ("Caveat", "handwriting"),
    ("Satisfy", "handwriting"),
    ("Great Vibes", "handwriting"),
    ("Kaushan Script", "handwriting"),
    ("Sacramento", "handwriting"),
    ("Indie Flower", "handwriting"),
    ("Shadows Into Light", "handwriting"),
    ("Amatic SC", "handwriting"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_variant_regular() {
        let v = FontVariant::parse("regular").unwrap();
        assert_eq!(v.weight, 400);
        assert!(!v.italic);
    }

    #[test]
    fn parse_variant_bold() {
        let v = FontVariant::parse("700").unwrap();
        assert_eq!(v.weight, 700);
        assert!(!v.italic);
    }

    #[test]
    fn parse_variant_bold_italic() {
        let v = FontVariant::parse("700italic").unwrap();
        assert_eq!(v.weight, 700);
        assert!(v.italic);
    }

    #[test]
    fn parse_variant_italic() {
        let v = FontVariant::parse("italic").unwrap();
        assert_eq!(v.weight, 400);
        assert!(v.italic);
    }

    #[test]
    fn variant_to_string() {
        assert_eq!(
            FontVariant {
                weight: 400,
                italic: false
            }
            .to_string(),
            "regular"
        );
        assert_eq!(
            FontVariant {
                weight: 700,
                italic: true
            }
            .to_string(),
            "700italic"
        );
    }

    #[test]
    fn popular_fonts_not_empty() {
        assert!(!POPULAR_FONTS.is_empty());
    }

    #[test]
    fn popular_fonts_have_categories() {
        for (name, category) in POPULAR_FONTS {
            assert!(!name.is_empty());
            assert!(!category.is_empty());
        }
    }
}
