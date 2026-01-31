//! WASM World implementation for Typst compilation.
//!
//! This module provides a minimal World implementation that works in WebAssembly
//! without filesystem access.

use std::sync::OnceLock;

use chrono::{Datelike, Local, Utc};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

/// Fonts embedded in the WASM module.
static EMBEDDED_FONTS: OnceLock<EmbeddedFonts> = OnceLock::new();

/// A World implementation for WebAssembly.
///
/// This provides a minimal environment for compiling Typst documents
/// without filesystem access.
pub struct WasmWorld {
    /// The main source file.
    main: FileId,
    /// The source content.
    source: Source,
    /// The standard library.
    library: LazyHash<Library>,
}

impl WasmWorld {
    /// Create a new WASM world with the given source.
    pub fn new(source: &str) -> Self {
        // Initialize embedded fonts if not already done
        let fonts = EMBEDDED_FONTS.get_or_init(EmbeddedFonts::new);
        let _ = fonts; // Ensure fonts are loaded

        // Create a virtual file ID for the main source
        let path = VirtualPath::new("main.typ").unwrap();
        let rooted_path = RootedPath::new(VirtualRoot::Project, path);
        let main = FileId::new(rooted_path);

        // Create the source
        let source = Source::new(main, source.to_string());

        // Create the library
        let library = Library::default();

        Self {
            main,
            source,
            library: LazyHash::new(library),
        }
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
            Err(FileError::NotFound(id.vpath().get_without_slash().into()))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main {
            // Convert EcoString to owned String for 'static lifetime
            let text: String = self.source.text().to_string();
            Ok(Bytes::from_string(text))
        } else {
            Err(FileError::NotFound(id.vpath().get_without_slash().into()))
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
            let buffer = Bytes::new(font_data);
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
        assert_eq!(world.source(world.main()).unwrap().text().as_str(), "Hello, world!");
    }

    #[test]
    fn fonts_loaded() {
        let world = WasmWorld::new("");
        let book = world.book();
        // Should have some fonts loaded
        assert!(!book.is_empty());
    }
}
