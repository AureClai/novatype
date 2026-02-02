//! Native World implementation for Typst compilation.
//!
//! This module provides a World implementation that works on the native filesystem,
//! supporting file reading, font discovery, and all Typst features.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use chrono::{Datelike, Local, Utc};
use parking_lot::RwLock;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};

/// Global font cache for native compilation.
static NATIVE_FONTS: OnceLock<RwLock<Option<NativeFonts>>> = OnceLock::new();

/// Custom font paths to load.
static CUSTOM_FONT_PATHS: OnceLock<RwLock<Vec<PathBuf>>> = OnceLock::new();

/// Set custom font paths for compilation.
/// This will cause fonts to be reloaded on the next compilation.
pub fn set_font_paths(paths: Vec<PathBuf>) {
    let paths_lock = CUSTOM_FONT_PATHS.get_or_init(|| RwLock::new(Vec::new()));
    *paths_lock.write() = paths;

    // Clear cached fonts so they'll be reloaded
    let fonts_lock = NATIVE_FONTS.get_or_init(|| RwLock::new(None));
    *fonts_lock.write() = None;
}

/// Get or initialize the native fonts.
fn get_native_fonts() -> &'static RwLock<Option<NativeFonts>> {
    let lock = NATIVE_FONTS.get_or_init(|| RwLock::new(None));

    // Initialize if needed
    if lock.read().is_none() {
        let fonts = NativeFonts::new();
        *lock.write() = Some(fonts);
    }

    lock
}

/// A World implementation for native compilation.
///
/// This provides a full environment for compiling Typst documents
/// with filesystem access.
pub struct NativeWorld {
    /// The main source file ID.
    main: FileId,
    /// Root directory for resolving paths.
    root: PathBuf,
    /// The standard library.
    library: LazyHash<Library>,
    /// Source cache (file ID -> source).
    sources: RwLock<HashMap<FileId, Source>>,
    /// File cache (file ID -> bytes).
    files: RwLock<HashMap<FileId, Bytes>>,
}

impl NativeWorld {
    /// Create a new native world from a file path.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };

        // Determine root directory
        let root = abs_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        // Create file ID for main file
        let file_name = abs_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("main.typ");
        let vpath = VirtualPath::new(file_name);
        let main = FileId::new(None, vpath);

        // Read the main source file
        let content = std::fs::read_to_string(&abs_path)?;
        let source = Source::new(main, content);

        // Initialize fonts
        let _ = get_native_fonts();

        // Create the library
        let library = Library::builder().build();

        let mut sources = HashMap::new();
        sources.insert(main, source);

        Ok(Self {
            main,
            root,
            library: LazyHash::new(library),
            sources: RwLock::new(sources),
            files: RwLock::new(HashMap::new()),
        })
    }

    /// Create a new native world from source string.
    pub fn from_source(source: &str, root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();

        // Create file ID for main source
        let vpath = VirtualPath::new("main.typ");
        let main = FileId::new(None, vpath);

        let source = Source::new(main, source.to_string());

        // Initialize fonts
        let _ = get_native_fonts();

        // Create the library
        let library = Library::builder().build();

        let mut sources = HashMap::new();
        sources.insert(main, source);

        Self {
            main,
            root,
            library: LazyHash::new(library),
            sources: RwLock::new(sources),
            files: RwLock::new(HashMap::new()),
        }
    }

    /// Resolve a file path relative to root.
    fn resolve_path(&self, id: FileId) -> PathBuf {
        let vpath = id.vpath().as_rootless_path();
        self.root.join(vpath)
    }
}

impl World for NativeWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        let fonts_lock = get_native_fonts();
        let guard = fonts_lock.read();
        // SAFETY: We know the fonts are initialized because get_native_fonts() ensures it
        let fonts = guard.as_ref().unwrap();
        // We need to return a reference with 'static lifetime
        // This is safe because the fonts are stored in a static
        unsafe { &*(&fonts.book as *const _) }
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        // Check cache first
        if let Some(source) = self.sources.read().get(&id) {
            return Ok(source.clone());
        }

        // Read from filesystem
        let path = self.resolve_path(id);
        let content = std::fs::read_to_string(&path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => FileError::NotFound(path.clone()),
            std::io::ErrorKind::PermissionDenied => FileError::AccessDenied,
            _ => FileError::Other(Some(ecow::eco_format!("{}", e))),
        })?;

        let source = Source::new(id, content);
        self.sources.write().insert(id, source.clone());
        Ok(source)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        // Check cache first
        if let Some(bytes) = self.files.read().get(&id) {
            return Ok(bytes.clone());
        }

        // Read from filesystem
        let path = self.resolve_path(id);
        let data = std::fs::read(&path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => FileError::NotFound(path.clone()),
            std::io::ErrorKind::PermissionDenied => FileError::AccessDenied,
            _ => FileError::Other(Some(ecow::eco_format!("{}", e))),
        })?;

        let bytes = Bytes::new(data);
        self.files.write().insert(id, bytes.clone());
        Ok(bytes)
    }

    fn font(&self, index: usize) -> Option<Font> {
        let fonts_lock = get_native_fonts();
        let guard = fonts_lock.read();
        guard
            .as_ref()
            .and_then(|fonts| fonts.fonts.get(index).cloned())
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

/// Container for native fonts.
struct NativeFonts {
    /// Font metadata book.
    book: LazyHash<FontBook>,
    /// Loaded fonts.
    fonts: Vec<Font>,
}

impl NativeFonts {
    /// Initialize fonts from typst-assets and custom font paths.
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

        // Load fonts from custom paths
        if let Some(paths_lock) = CUSTOM_FONT_PATHS.get() {
            let paths = paths_lock.read();
            for path in paths.iter() {
                Self::load_fonts_from_path(&mut book, &mut fonts, path);
            }
        }

        Self {
            book: LazyHash::new(book),
            fonts,
        }
    }

    /// Load fonts from a directory path.
    fn load_fonts_from_path(book: &mut FontBook, fonts: &mut Vec<Font>, path: &Path) {
        if path.is_file() {
            Self::load_font_file(book, fonts, path);
        } else if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        Self::load_font_file(book, fonts, &entry_path);
                    } else if entry_path.is_dir() {
                        // Recurse into subdirectories
                        Self::load_fonts_from_path(book, fonts, &entry_path);
                    }
                }
            }
        }
    }

    /// Load a single font file.
    fn load_font_file(book: &mut FontBook, fonts: &mut Vec<Font>, path: &Path) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(
            ext.to_lowercase().as_str(),
            "ttf" | "otf" | "ttc" | "woff" | "woff2"
        ) {
            return;
        }

        if let Ok(data) = std::fs::read(path) {
            let buffer = Bytes::new(data);
            for font in Font::iter(buffer) {
                tracing::debug!("Loaded font: {} from {:?}", font.info().family, path);
                book.push(font.info().clone());
                fonts.push(font);
            }
        }
    }
}

/// Compile a document to PDF.
pub fn compile_pdf(world: &NativeWorld) -> Result<Vec<u8>, Vec<String>> {
    let result = typst::compile::<typst::layout::PagedDocument>(world);

    // Collect warnings
    for warning in &result.warnings {
        tracing::warn!("Typst warning: {}", warning.message);
    }

    match result.output {
        Ok(document) => {
            let options = typst_pdf::PdfOptions::default();
            let pdf = typst_pdf::pdf(&document, &options).map_err(|errors| {
                errors
                    .iter()
                    .map(|e| format!("{}", e.message))
                    .collect::<Vec<_>>()
            })?;
            Ok(pdf)
        }
        Err(errors) => {
            let error_messages: Vec<String> =
                errors.iter().map(|e| format!("{}", e.message)).collect();
            Err(error_messages)
        }
    }
}

/// Compile a document to SVG pages.
pub fn compile_svg(world: &NativeWorld) -> Result<Vec<String>, Vec<String>> {
    let result = typst::compile::<typst::layout::PagedDocument>(world);

    // Collect warnings
    for warning in &result.warnings {
        tracing::warn!("Typst warning: {}", warning.message);
    }

    match result.output {
        Ok(document) => {
            let svg_pages: Vec<String> = document.pages.iter().map(typst_svg::svg).collect();
            Ok(svg_pages)
        }
        Err(errors) => {
            let error_messages: Vec<String> =
                errors.iter().map(|e| format!("{}", e.message)).collect();
            Err(error_messages)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn create_world_from_source() {
        let world = NativeWorld::from_source("Hello, world!", ".");
        let source = world.source(world.main()).unwrap();
        assert_eq!(source.text(), "Hello, world!");
    }

    #[test]
    fn fonts_loaded() {
        let world = NativeWorld::from_source("", ".");
        assert!(
            world.font(0).is_some(),
            "Should have at least one font loaded"
        );
    }

    #[test]
    fn compile_simple_document() {
        let world = NativeWorld::from_source("Hello, World!", ".");
        let result = compile_pdf(&world);
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());

        let pdf = result.unwrap();
        assert!(!pdf.is_empty());
        // Check PDF magic bytes
        assert_eq!(&pdf[0..5], b"%PDF-");
    }

    #[test]
    fn compile_to_svg() {
        let world = NativeWorld::from_source("Hello, SVG!", ".");
        let result = compile_svg(&world);
        assert!(result.is_ok());

        let pages = result.unwrap();
        assert!(!pages.is_empty());
        assert!(pages[0].contains("<svg"));
    }

    #[test]
    fn create_world_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.typ");
        std::fs::write(&file_path, "Test content").unwrap();

        let world = NativeWorld::new(&file_path).unwrap();
        let source = world.source(world.main()).unwrap();
        assert_eq!(source.text(), "Test content");
    }
}
