//! # Nova Schema
//!
//! Schema validation and metadata management for NovaType documents.
//!
//! This crate provides:
//! - JSON Schema validation for document metadata (requires `validation` feature)
//! - Template schema definitions
//! - Frontmatter parsing (YAML/TOML)
//!
//! ## Example
//!
//! ```rust,ignore
//! use nova_schema::{Schema, ValidationResult};
//!
//! let schema = Schema::load("ieee-article")?;
//! let result = schema.validate(&metadata)?;
//! ```

pub mod error;
pub mod frontmatter;
#[cfg(feature = "validation")]
pub mod schema;
pub mod template;

pub use error::{Error, Result};
pub use frontmatter::FrontmatterParser;
#[cfg(feature = "validation")]
pub use schema::{article_schema, Schema, ValidationResult};
pub use template::TemplateSchema;
