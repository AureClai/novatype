//! Document compiler implementation.
//!
//! The [`Compiler`] orchestrates the document compilation pipeline,
//! coordinating between parsing, validation, and rendering.

use crate::document::Document;
use crate::error::{Error, Result};
use crate::traits::TypstCompiler;
use std::path::Path;
use tracing::{debug, info, instrument};

/// Configuration for the document compiler.
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Root directory for resolving relative paths.
    pub root_dir: Option<std::path::PathBuf>,

    /// Whether to enable incremental compilation.
    pub incremental: bool,

    /// Output format.
    pub output_format: OutputFormat,

    /// Whether to validate schema strictly.
    pub strict_validation: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            root_dir: None,
            incremental: true,
            output_format: OutputFormat::Pdf,
            strict_validation: true,
        }
    }
}

/// Supported output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// PDF document.
    #[default]
    Pdf,
    /// SVG pages.
    Svg,
    /// PNG images.
    Png,
}

/// The main document compiler.
///
/// Coordinates the full compilation pipeline:
/// 1. Load and parse document
/// 2. Validate metadata against schema
/// 3. Resolve citations and data
/// 4. Compile via Typst
/// 5. Post-process output
pub struct Compiler<T: TypstCompiler> {
    /// Compiler configuration.
    config: CompilerConfig,

    /// Typst compiler backend.
    typst: T,
}

impl<T: TypstCompiler> Compiler<T> {
    /// Create a new compiler with the given configuration and backend.
    #[must_use]
    pub fn new(config: CompilerConfig, typst: T) -> Self {
        Self { config, typst }
    }

    /// Compile a document from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the source document
    ///
    /// # Returns
    ///
    /// The compiled output as bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The document is invalid
    /// - Compilation fails
    #[instrument(skip(self, path))]
    pub fn compile_file(&self, path: impl AsRef<Path>) -> Result<CompileOutput> {
        let path = path.as_ref();
        info!(?path, "Compiling document");

        if !path.exists() {
            return Err(Error::SourceNotFound {
                path: path.display().to_string(),
            });
        }

        let document = Document::from_file(path)?;
        self.compile_document(&document)
    }

    /// Compile a document from source string.
    ///
    /// # Arguments
    ///
    /// * `source` - The document source code
    ///
    /// # Returns
    ///
    /// The compiled output.
    #[instrument(skip(self, source))]
    pub fn compile_source(&self, source: &str) -> Result<CompileOutput> {
        debug!("Compiling from source string");
        let document = Document::from_content(source);
        self.compile_document(&document)
    }

    /// Compile a document object.
    ///
    /// This is the main compilation entry point.
    #[instrument(skip(self, document))]
    pub fn compile_document(&self, document: &Document) -> Result<CompileOutput> {
        debug!("Starting compilation pipeline");

        // Step 1: Validate metadata
        if self.config.strict_validation && document.has_metadata() {
            self.validate_metadata(document)?;
        }

        // Step 2: Pre-process content
        let processed = self.preprocess(document)?;

        // Step 3: Compile with Typst
        let output = match self.config.output_format {
            OutputFormat::Pdf => {
                let bytes = self.typst.compile_pdf(&processed)?;
                CompileOutput::Pdf(bytes)
            }
            OutputFormat::Svg => {
                let pages = self.typst.compile_svg(&processed)?;
                CompileOutput::Svg(pages)
            }
            OutputFormat::Png => {
                // PNG is derived from SVG
                let pages = self.typst.compile_svg(&processed)?;
                // TODO: Implement SVG to PNG conversion
                CompileOutput::Svg(pages)
            }
        };

        info!("Compilation successful");
        Ok(output)
    }

    /// Validate document metadata against schema.
    fn validate_metadata(&self, document: &Document) -> Result<()> {
        debug!("Validating metadata");

        if let Some(template) = document.template() {
            // TODO: Load and validate against template schema
            debug!(?template, "Template validation not yet implemented");
        }

        Ok(())
    }

    /// Pre-process document content.
    fn preprocess(&self, document: &Document) -> Result<String> {
        debug!("Pre-processing document");

        // TODO: Implement preprocessing pipeline
        // - Resolve includes
        // - Process citations
        // - Generate plots

        Ok(document.content.clone())
    }
}

/// Output from the compiler.
#[derive(Debug)]
pub enum CompileOutput {
    /// PDF document bytes.
    Pdf(Vec<u8>),
    /// SVG pages.
    Svg(Vec<String>),
}

impl CompileOutput {
    /// Check if output is PDF.
    #[must_use]
    pub fn is_pdf(&self) -> bool {
        matches!(self, Self::Pdf(_))
    }

    /// Check if output is SVG.
    #[must_use]
    pub fn is_svg(&self) -> bool {
        matches!(self, Self::Svg(_))
    }

    /// Get PDF bytes if output is PDF.
    #[must_use]
    pub fn as_pdf(&self) -> Option<&[u8]> {
        match self {
            Self::Pdf(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// Get SVG pages if output is SVG.
    #[must_use]
    pub fn as_svg(&self) -> Option<&[String]> {
        match self {
            Self::Svg(pages) => Some(pages),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::MockTypstCompiler;

    fn create_mock_compiler() -> Compiler<MockTypstCompiler> {
        let mut mock = MockTypstCompiler::new();
        mock.expect_compile_pdf()
            .returning(|_| Ok(vec![0x25, 0x50, 0x44, 0x46])); // %PDF magic bytes
        mock.expect_compile_svg()
            .returning(|_| Ok(vec!["<svg></svg>".to_string()]));

        Compiler::new(CompilerConfig::default(), mock)
    }

    #[test]
    fn compiler_config_default() {
        let config = CompilerConfig::default();
        assert!(config.incremental);
        assert!(config.strict_validation);
        assert_eq!(config.output_format, OutputFormat::Pdf);
    }

    #[test]
    fn compile_source_pdf() {
        let compiler = create_mock_compiler();
        let result = compiler.compile_source("Hello");
        assert!(result.is_ok());
        assert!(result.unwrap().is_pdf());
    }

    #[test]
    fn compile_source_svg() {
        let mut mock = MockTypstCompiler::new();
        mock.expect_compile_svg()
            .returning(|_| Ok(vec!["<svg></svg>".to_string()]));

        let config = CompilerConfig {
            output_format: OutputFormat::Svg,
            ..Default::default()
        };
        let compiler = Compiler::new(config, mock);

        let result = compiler.compile_source("Hello");
        assert!(result.is_ok());
        assert!(result.unwrap().is_svg());
    }

    #[test]
    fn compile_file_not_found() {
        let compiler = create_mock_compiler();
        let result = compiler.compile_file("/nonexistent/file.typ");
        assert!(matches!(result, Err(Error::SourceNotFound { .. })));
    }

    #[test]
    fn compile_output_accessors() {
        let pdf = CompileOutput::Pdf(vec![1, 2, 3]);
        assert!(pdf.is_pdf());
        assert!(!pdf.is_svg());
        assert_eq!(pdf.as_pdf(), Some(&[1u8, 2, 3][..]));
        assert!(pdf.as_svg().is_none());

        let svg = CompileOutput::Svg(vec!["<svg/>".to_string()]);
        assert!(!svg.is_pdf());
        assert!(svg.is_svg());
        assert!(svg.as_pdf().is_none());
    }
}
