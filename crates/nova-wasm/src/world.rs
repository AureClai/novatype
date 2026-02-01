//! WASM World implementation for Typst compilation.
//!
//! This module provides a minimal World implementation that works in WebAssembly
//! without filesystem access.
//!
//! ## Virtual Files
//!
//! You can embed virtual files in your source using special comments:
//!
//! ```typst
//! // === refs.bib ===
//! // @article{einstein1905,
//! //   author = {Albert Einstein},
//! //   title = {On the Electrodynamics of Moving Bodies},
//! //   year = {1905},
//! // }
//! // === end ===
//! ```

use std::collections::HashMap;
use std::sync::OnceLock;

use chrono::{Datelike, Local, Utc};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};

/// Fonts embedded in the WASM module.
static EMBEDDED_FONTS: OnceLock<EmbeddedFonts> = OnceLock::new();

/// A World implementation for WebAssembly.
///
/// This provides a minimal environment for compiling Typst documents
/// without filesystem access. Virtual files can be embedded in the source.
pub struct WasmWorld {
    /// The main source file.
    main: FileId,
    /// The source content (without virtual file blocks).
    source: Source,
    /// Virtual files extracted from the source.
    virtual_files: HashMap<String, String>,
    /// The standard library.
    library: LazyHash<Library>,
}

impl WasmWorld {
    /// Create a new WASM world with the given source.
    pub fn new(source: &str) -> Self {
        // Initialize embedded fonts if not already done
        let fonts = EMBEDDED_FONTS.get_or_init(EmbeddedFonts::new);
        let _ = fonts; // Ensure fonts are loaded

        // Extract virtual files and clean source
        let (clean_source, virtual_files) = Self::extract_virtual_files(source);

        // Create a virtual file ID for the main source (no package, just a path)
        let path = VirtualPath::new("main.typ");
        let main = FileId::new(None, path);

        // Create the source
        let source = Source::new(main, clean_source);

        // Create the library
        let library = Library::builder().build();

        Self {
            main,
            source,
            virtual_files,
            library: LazyHash::new(library),
        }
    }

    /// Extract virtual files from source.
    ///
    /// Format:
    /// ```text
    /// // === filename.ext ===
    /// // file content line 1
    /// // file content line 2
    /// // === end ===
    /// ```
    fn extract_virtual_files(source: &str) -> (String, HashMap<String, String>) {
        let mut virtual_files = HashMap::new();
        let mut clean_lines = Vec::new();
        let mut current_file: Option<String> = None;
        let mut current_content = Vec::new();

        for line in source.lines() {
            let trimmed = line.trim();

            // Check for file start: // === filename.ext ===
            if trimmed.starts_with("// ===") && trimmed.ends_with("===") && current_file.is_none() {
                let inner = trimmed
                    .strip_prefix("// ===")
                    .unwrap()
                    .strip_suffix("===")
                    .unwrap()
                    .trim();

                if inner != "end" && !inner.is_empty() {
                    current_file = Some(inner.to_string());
                    continue;
                }
            }

            // Check for file end: // === end ===
            if trimmed == "// === end ===" {
                if let Some(filename) = current_file.take() {
                    let content = current_content.join("\n");
                    virtual_files.insert(filename, content);
                    current_content.clear();
                }
                continue;
            }

            // If inside a virtual file block, collect content
            if current_file.is_some() {
                // Remove leading "// " from content lines
                let content_line = if let Some(stripped) = trimmed.strip_prefix("// ") {
                    stripped
                } else if trimmed == "//" {
                    ""
                } else {
                    trimmed
                };
                current_content.push(content_line.to_string());
            } else {
                // Regular source line
                clean_lines.push(line.to_string());
            }
        }

        (clean_lines.join("\n"), virtual_files)
    }

    /// Get a virtual file by name.
    fn get_virtual_file(&self, name: &str) -> Option<&String> {
        self.virtual_files.get(name)
    }
}

impl World for WasmWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        let fonts = EMBEDDED_FONTS.get_or_init(EmbeddedFonts::new);
        &fonts.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main {
            Ok(self.source.clone())
        } else {
            // Check virtual files for .typ files
            let path = id.vpath().as_rootless_path().to_string_lossy();
            if let Some(content) = self.get_virtual_file(path.as_ref()) {
                let source = Source::new(id, content.clone());
                Ok(source)
            } else {
                Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
            }
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main {
            // Return the source as bytes
            Ok(Bytes::new(self.source.text().as_bytes().to_vec()))
        } else {
            // Check virtual files
            let path = id.vpath().as_rootless_path().to_string_lossy();
            if let Some(content) = self.get_virtual_file(path.as_ref()) {
                Ok(Bytes::new(content.as_bytes().to_vec()))
            } else {
                Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
            }
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        let fonts = EMBEDDED_FONTS.get_or_init(EmbeddedFonts::new);
        fonts.fonts.get(index).cloned()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = Utc::now();
        let with_offset = match offset {
            None => Local::now().naive_local(),
            Some(hours) => {
                let seconds = i32::try_from(hours).ok()?.checked_mul(3600)?;
                let offset = chrono::FixedOffset::east_opt(seconds)?;
                now.with_timezone(&offset).naive_local()
            }
        };

        Datetime::from_ymd(
            with_offset.year(),
            with_offset.month().try_into().ok()?,
            with_offset.day().try_into().ok()?,
        )
    }
}

/// Container for embedded fonts.
struct EmbeddedFonts {
    /// Font metadata book.
    book: LazyHash<FontBook>,
    /// Loaded fonts.
    fonts: Vec<Font>,
}

impl EmbeddedFonts {
    /// Initialize embedded fonts from typst-assets.
    fn new() -> Self {
        let mut book = FontBook::new();
        let mut fonts = Vec::new();

        // Load embedded fonts from typst-assets
        for font_data in typst_assets::fonts() {
            let buffer = Bytes::new(font_data.to_vec());
            for font in Font::iter(buffer) {
                book.push(font.info().clone());
                fonts.push(font);
            }
        }

        Self {
            book: LazyHash::new(book),
            fonts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_world() {
        let world = WasmWorld::new("Hello, world!");
        let source = world.source(world.main()).unwrap();
        assert_eq!(source.text(), "Hello, world!");
    }

    #[test]
    fn fonts_loaded() {
        let world = WasmWorld::new("");
        // Check that we can get a font (index 0 should exist)
        assert!(world.font(0).is_some(), "Should have at least one font loaded");
    }

    #[test]
    fn extract_virtual_files() {
        let source = r#"
// === refs.bib ===
// @article{test,
//   author = {Test},
// }
// === end ===

= Hello World
"#;
        let world = WasmWorld::new(source);

        // Check virtual file exists
        assert!(world.virtual_files.contains_key("refs.bib"));
        let bib = world.virtual_files.get("refs.bib").unwrap();
        assert!(bib.contains("@article{test"));

        // Check source is clean
        let clean = world.source.text();
        assert!(!clean.contains("=== refs.bib ==="));
        assert!(clean.contains("Hello World"));
    }
}
