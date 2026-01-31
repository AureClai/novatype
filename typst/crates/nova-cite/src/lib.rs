//! # Nova Cite
//!
//! Smart citation management with external API integration.
//!
//! This crate provides:
//! - BibTeX/BibLaTeX parsing
//! - CrossRef API integration
//! - Zotero integration
//! - Citation formatting (APA, IEEE, Chicago, etc.)
//!
//! ## Example
//!
//! ```rust,ignore
//! use nova_cite::{CitationManager, CrossRefProvider};
//!
//! let manager = CitationManager::new()
//!     .with_provider(CrossRefProvider::new());
//!
//! let citation = manager.resolve("10.1234/example").await?;
//! let formatted = manager.format(&citation, "ieee")?;
//! ```

pub mod bibtex;
pub mod crossref;
pub mod error;
pub mod formatter;
pub mod manager;

pub use error::{Error, Result};
pub use formatter::{CitationStyle, Formatter};
pub use manager::CitationManager;
