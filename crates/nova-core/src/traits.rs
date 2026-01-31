//! Core traits defining the interfaces between NovaType modules.
//!
//! These traits establish clear contracts that enable:
//! - Loose coupling between components
//! - Easy mocking for unit tests
//! - Plugin-style extensibility
//!
//! # Design Principles
//!
//! 1. **Single Responsibility**: Each trait has one purpose
//! 2. **Mockability**: All traits are object-safe where possible
//! 3. **Error Transparency**: Errors propagate with full context

use crate::document::Document;
use crate::error::Result;

#[cfg(test)]
use mockall::automock;

/// Processes document content through a transformation pipeline.
///
/// Implementations might include:
/// - Markdown to Typst conversion
/// - Template expansion
/// - Content validation
#[cfg_attr(test, automock)]
pub trait ContentProcessor: Send + Sync {
    /// Process the document content.
    ///
    /// # Arguments
    ///
    /// * `document` - The document to process
    ///
    /// # Returns
    ///
    /// The processed document content as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if processing fails.
    fn process(&self, document: &Document) -> Result<String>;
}

/// Resolves and formats citations.
///
/// This trait abstracts citation handling to support multiple backends:
/// - Local BibTeX files
/// - CrossRef API
/// - Zotero integration
#[cfg_attr(test, automock)]
pub trait CitationProvider: Send + Sync {
    /// Resolve a citation by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The citation key (e.g., "smith2023")
    ///
    /// # Returns
    ///
    /// The resolved citation data.
    fn resolve(&self, key: &str) -> Result<Citation>;

    /// Format a citation according to a style.
    ///
    /// # Arguments
    ///
    /// * `citation` - The citation to format
    /// * `style` - The citation style (e.g., "apa", "ieee")
    ///
    /// # Returns
    ///
    /// The formatted citation string.
    fn format(&self, citation: &Citation, style: &str) -> Result<String>;

    /// Batch resolve multiple citations.
    ///
    /// Default implementation calls `resolve` for each key.
    /// Implementations may override for better performance.
    fn resolve_batch(&self, keys: &[String]) -> Vec<Result<Citation>> {
        keys.iter().map(|key| self.resolve(key)).collect()
    }
}

/// Renders data visualizations.
///
/// Supports multiple chart types and data formats.
#[cfg_attr(test, automock)]
pub trait DataVisualizer: Send + Sync {
    /// Render a visualization from data.
    ///
    /// # Arguments
    ///
    /// * `data` - The data source
    /// * `config` - Visualization configuration
    ///
    /// # Returns
    ///
    /// SVG output as a string.
    fn render(&self, data: &DataSource, config: &PlotConfig) -> Result<String>;

    /// List supported chart types.
    fn supported_types(&self) -> Vec<ChartType>;
}

/// Compiles Typst documents to output formats.
#[cfg_attr(test, automock)]
pub trait TypstCompiler: Send + Sync {
    /// Compile a Typst document to PDF.
    ///
    /// # Arguments
    ///
    /// * `source` - The Typst source code
    ///
    /// # Returns
    ///
    /// The compiled PDF as bytes.
    fn compile_pdf(&self, source: &str) -> Result<Vec<u8>>;

    /// Compile a Typst document to SVG pages.
    ///
    /// # Arguments
    ///
    /// * `source` - The Typst source code
    ///
    /// # Returns
    ///
    /// Vector of SVG strings, one per page.
    fn compile_svg(&self, source: &str) -> Result<Vec<String>>;
}

/// Citation data structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    /// Unique citation key.
    pub key: String,
    /// Citation type (article, book, etc.).
    pub citation_type: CitationType,
    /// Article/book title.
    pub title: String,
    /// List of authors.
    pub authors: Vec<Author>,
    /// Publication year.
    pub year: Option<u16>,
    /// DOI if available.
    pub doi: Option<String>,
    /// Additional fields.
    pub extra: std::collections::HashMap<String, String>,
}

/// Types of citations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CitationType {
    /// Journal article.
    Article,
    /// Book.
    Book,
    /// Book chapter.
    InBook,
    /// Conference paper.
    InProceedings,
    /// Technical report.
    TechReport,
    /// Thesis (PhD, Masters).
    Thesis,
    /// Website.
    Online,
    /// Other/unknown type.
    Misc,
}

/// Author information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Author {
    /// First name(s).
    pub given: String,
    /// Family name.
    pub family: String,
}

impl Author {
    /// Create a new author.
    #[must_use]
    pub fn new(given: impl Into<String>, family: impl Into<String>) -> Self {
        Self {
            given: given.into(),
            family: family.into(),
        }
    }

    /// Format as "Family, Given".
    #[must_use]
    pub fn format_family_first(&self) -> String {
        format!("{}, {}", self.family, self.given)
    }

    /// Format as "Given Family".
    #[must_use]
    pub fn format_given_first(&self) -> String {
        format!("{} {}", self.given, self.family)
    }
}

/// Data source for visualizations.
#[derive(Debug, Clone)]
pub enum DataSource {
    /// CSV data with headers.
    Csv {
        /// CSV content.
        content: String,
        /// Whether first row is headers.
        has_headers: bool,
    },
    /// JSON data.
    Json(serde_json::Value),
    /// Inline data points.
    Inline(Vec<DataPoint>),
}

/// A single data point.
#[derive(Debug, Clone)]
pub struct DataPoint {
    /// X value.
    pub x: f64,
    /// Y value.
    pub y: f64,
    /// Optional label.
    pub label: Option<String>,
}

/// Configuration for plots.
#[derive(Debug, Clone)]
pub struct PlotConfig {
    /// Chart type.
    pub chart_type: ChartType,
    /// Chart title.
    pub title: Option<String>,
    /// X-axis label.
    pub x_label: Option<String>,
    /// Y-axis label.
    pub y_label: Option<String>,
    /// Width in points.
    pub width: f64,
    /// Height in points.
    pub height: f64,
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self {
            chart_type: ChartType::Line,
            title: None,
            x_label: None,
            y_label: None,
            width: 400.0,
            height: 300.0,
        }
    }
}

/// Supported chart types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartType {
    /// Line chart.
    Line,
    /// Bar chart.
    Bar,
    /// Scatter plot.
    Scatter,
    /// Pie chart.
    Pie,
    /// Area chart.
    Area,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn author_format_family_first() {
        let author = Author::new("Albert", "Einstein");
        assert_eq!(author.format_family_first(), "Einstein, Albert");
    }

    #[test]
    fn author_format_given_first() {
        let author = Author::new("Albert", "Einstein");
        assert_eq!(author.format_given_first(), "Albert Einstein");
    }

    #[test]
    fn plot_config_default() {
        let config = PlotConfig::default();
        assert_eq!(config.chart_type, ChartType::Line);
        assert_eq!(config.width, 400.0);
        assert_eq!(config.height, 300.0);
    }

    #[test]
    fn mock_content_processor() {
        let mut mock = MockContentProcessor::new();
        mock.expect_process()
            .returning(|_| Ok("processed content".to_string()));

        let doc = Document::default();
        let result = mock.process(&doc);
        assert!(result.is_ok());
    }
}
