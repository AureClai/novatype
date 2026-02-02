//! # Nova Font
//!
//! Font management with Google Fonts integration for NovaType.
//!
//! This crate provides:
//! - Google Fonts API integration
//! - Local font cache management
//! - Font search and discovery
//! - Automatic font downloading
//!
//! ## Example
//!
//! ```rust,ignore
//! use nova_font::{FontManager, GoogleFontsClient};
//!
//! let manager = FontManager::new()?;
//!
//! // Search for fonts
//! let fonts = manager.search("mono").await?;
//!
//! // Install a font
//! manager.install("JetBrains Mono").await?;
//!
//! // List installed fonts
//! let installed = manager.list_installed()?;
//! ```

pub mod cache;
pub mod error;
pub mod google_fonts;
pub mod manager;

pub use cache::FontCache;
pub use error::{Error, Result};
pub use google_fonts::{FontFamily, FontVariant, GoogleFontsClient};
pub use manager::FontManager;

/// Font categories from Google Fonts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontCategory {
    /// Serif fonts (e.g., Times New Roman).
    Serif,
    /// Sans-serif fonts (e.g., Arial).
    SansSerif,
    /// Display/decorative fonts.
    Display,
    /// Handwriting fonts.
    Handwriting,
    /// Monospace fonts (e.g., Courier).
    Monospace,
}

impl FontCategory {
    /// Convert to Google Fonts API category string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Serif => "serif",
            Self::SansSerif => "sans-serif",
            Self::Display => "display",
            Self::Handwriting => "handwriting",
            Self::Monospace => "monospace",
        }
    }
}

impl std::str::FromStr for FontCategory {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "serif" => Ok(Self::Serif),
            "sans-serif" => Ok(Self::SansSerif),
            "display" => Ok(Self::Display),
            "handwriting" => Ok(Self::Handwriting),
            "monospace" => Ok(Self::Monospace),
            _ => Err(()),
        }
    }
}

/// Font weight values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Regular = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

impl FontWeight {
    /// Convert from numeric weight.
    #[must_use]
    pub fn from_num(n: u16) -> Option<Self> {
        match n {
            100 => Some(Self::Thin),
            200 => Some(Self::ExtraLight),
            300 => Some(Self::Light),
            400 => Some(Self::Regular),
            500 => Some(Self::Medium),
            600 => Some(Self::SemiBold),
            700 => Some(Self::Bold),
            800 => Some(Self::ExtraBold),
            900 => Some(Self::Black),
            _ => None,
        }
    }

    /// Convert to numeric weight.
    #[must_use]
    pub fn as_num(&self) -> u16 {
        *self as u16
    }
}

impl Default for FontWeight {
    fn default() -> Self {
        Self::Regular
    }
}
