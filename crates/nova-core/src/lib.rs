//! # Nova Core
//!
//! Core orchestration engine for NovaType document composition system.
//!
//! This crate provides the main entry point for document compilation,
//! coordinating between schema validation, citation resolution, and
//! data visualization modules.
//!
//! ## Architecture
//!
//! The core is built around trait-based abstractions to ensure:
//! - **Testability**: All dependencies can be mocked
//! - **Modularity**: Features can be enabled/disabled independently
//! - **Extensibility**: New processors can be added without modifying core
//!
//! ## Example
//!
//! ```rust,ignore
//! use nova_core::{Compiler, CompilerConfig};
//!
//! let config = CompilerConfig::default();
//! let compiler = Compiler::new(config)?;
//! let output = compiler.compile("document.typ")?;
//! ```

pub mod compiler;
pub mod document;
pub mod error;
pub mod native_world;
pub mod traits;

pub use compiler::{Compiler, CompilerConfig};
pub use document::Document;
pub use error::{Error, Result};
pub use native_world::{compile_pdf, compile_svg, set_font_paths, NativeWorld};
pub use traits::*;
