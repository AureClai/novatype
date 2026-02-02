//! # nova-template
//!
//! Template management for NovaType document composition system.
//!
//! This crate provides functionality for:
//! - Installing templates from local directories, GitHub, or URLs
//! - Managing a local cache of installed templates
//! - Loading template configuration and generating Typst code
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use nova_template::{TemplateCache, TemplateSource, TemplateInstaller};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create cache
//!     let mut cache = TemplateCache::new()?;
//!
//!     // Install from GitHub
//!     let source = TemplateSource::parse("github:novatype/templates/article")?;
//!     let installer = TemplateInstaller::new();
//!     installer.install(&mut cache, &source).await?;
//!
//!     // List installed templates
//!     for template in cache.list_installed() {
//!         println!("{}: {}", template.name, template.version);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Template Structure
//!
//! A template is a directory containing:
//!
//! ```text
//! my-template/
//! ├── template.toml    # Template metadata and configuration
//! ├── main.typ         # Main Typst entry point
//! ├── lib.typ          # Helper functions (optional)
//! └── assets/          # Images, fonts, etc. (optional)
//! ```
//!
//! ## template.toml Format
//!
//! ```toml
//! [template]
//! name = "my-template"
//! version = "1.0.0"
//! description = "A custom template"
//! authors = ["Author Name"]
//! category = "article"  # article, book, report, memo, slides, letter, cv, other
//!
//! [schema]
//! required = ["title", "author"]
//!
//! [colors]
//! primary = "#1a365d"
//! secondary = "#e2e8f0"
//!
//! [page]
//! paper = "a4"
//!
//! [page.margin]
//! top = "2.5cm"
//! bottom = "2.5cm"
//! left = "2cm"
//! right = "2cm"
//!
//! [header]
//! right = "My Document"
//!
//! [footer]
//! center = "{page}"
//! ```

pub mod cache;
pub mod error;
pub mod source;
pub mod template;

pub use cache::{CacheStats, TemplateCache};
pub use error::{Error, Result};
pub use source::{TemplateInstaller, TemplateSource};
pub use template::{
    HeaderFooterConfig, InstalledTemplate, MarginConfig, PageConfig, SchemaConfig, Template,
    TemplateCategory, TemplateInfo,
};
