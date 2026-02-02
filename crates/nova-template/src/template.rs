//! Template definition and loading.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Main template file name.
pub const TEMPLATE_FILE: &str = "template.toml";

/// Main Typst entry point.
pub const MAIN_TYP: &str = "main.typ";

/// Template metadata and configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Template information.
    pub template: TemplateInfo,

    /// Schema configuration (required fields, etc.).
    #[serde(default)]
    pub schema: SchemaConfig,

    /// Color definitions.
    #[serde(default)]
    pub colors: HashMap<String, String>,

    /// Page configuration.
    #[serde(default)]
    pub page: Option<PageConfig>,

    /// Header configuration.
    #[serde(default)]
    pub header: Option<HeaderFooterConfig>,

    /// Footer configuration.
    #[serde(default)]
    pub footer: Option<HeaderFooterConfig>,
}

/// Basic template information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Template name (identifier).
    pub name: String,

    /// Display name.
    #[serde(default)]
    pub display_name: Option<String>,

    /// Template version.
    pub version: String,

    /// Short description.
    #[serde(default)]
    pub description: Option<String>,

    /// Authors.
    #[serde(default)]
    pub authors: Vec<String>,

    /// License.
    #[serde(default)]
    pub license: Option<String>,

    /// Template category.
    #[serde(default)]
    pub category: TemplateCategory,

    /// Repository URL.
    #[serde(default)]
    pub repository: Option<String>,

    /// Keywords for search.
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Minimum NovaType version required.
    #[serde(default)]
    pub min_nova_version: Option<String>,
}

/// Template categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateCategory {
    /// Scientific article.
    #[default]
    Article,
    /// Book or thesis.
    Book,
    /// Business report.
    Report,
    /// Memo or note.
    Memo,
    /// Presentation slides.
    Slides,
    /// Letter.
    Letter,
    /// CV / Resume.
    Cv,
    /// Other.
    Other,
}

impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Article => write!(f, "article"),
            Self::Book => write!(f, "book"),
            Self::Report => write!(f, "report"),
            Self::Memo => write!(f, "memo"),
            Self::Slides => write!(f, "slides"),
            Self::Letter => write!(f, "letter"),
            Self::Cv => write!(f, "cv"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Schema configuration for document validation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchemaConfig {
    /// Required frontmatter fields.
    #[serde(default)]
    pub required: Vec<String>,

    /// Optional fields with defaults.
    #[serde(default)]
    pub optional: HashMap<String, serde_json::Value>,
}

/// Page configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageConfig {
    /// Paper size (a4, letter, etc.).
    #[serde(default = "default_paper")]
    pub paper: String,

    /// Page margins.
    #[serde(default)]
    pub margin: Option<MarginConfig>,

    /// Page numbering style.
    #[serde(default)]
    pub numbering: Option<String>,
}

fn default_paper() -> String {
    "a4".to_string()
}

/// Margin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginConfig {
    #[serde(default)]
    pub top: Option<String>,
    #[serde(default)]
    pub bottom: Option<String>,
    #[serde(default)]
    pub left: Option<String>,
    #[serde(default)]
    pub right: Option<String>,
    /// Uniform margin (overridden by specific sides).
    #[serde(default)]
    pub all: Option<String>,
}

/// Header or footer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderFooterConfig {
    /// Left-aligned content.
    #[serde(default)]
    pub left: Option<String>,

    /// Center-aligned content.
    #[serde(default)]
    pub center: Option<String>,

    /// Right-aligned content.
    #[serde(default)]
    pub right: Option<String>,
}

impl Template {
    /// Load a template from a directory.
    pub fn from_directory(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let template_file = path.join(TEMPLATE_FILE);

        if !template_file.exists() {
            return Err(Error::MissingFile {
                file: TEMPLATE_FILE.to_string(),
            });
        }

        let content = fs::read_to_string(&template_file).map_err(|e| Error::Io {
            path: Some(template_file.clone()),
            source: e,
        })?;

        let template: Template = toml::from_str(&content)?;
        debug!(name = %template.template.name, "Loaded template");

        Ok(template)
    }

    /// Get the template name.
    pub fn name(&self) -> &str {
        &self.template.name
    }

    /// Get the display name (or name if not set).
    pub fn display_name(&self) -> &str {
        self.template
            .display_name
            .as_deref()
            .unwrap_or(&self.template.name)
    }

    /// Get the template version.
    pub fn version(&self) -> &str {
        &self.template.version
    }

    /// Validate that a template directory has all required files.
    pub fn validate_directory(path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        // Check template.toml exists
        if !path.join(TEMPLATE_FILE).exists() {
            return Err(Error::MissingFile {
                file: TEMPLATE_FILE.to_string(),
            });
        }

        // Check main.typ exists
        if !path.join(MAIN_TYP).exists() {
            return Err(Error::MissingFile {
                file: MAIN_TYP.to_string(),
            });
        }

        Ok(())
    }

    /// Generate Typst code for color definitions.
    pub fn generate_color_definitions(&self) -> String {
        let mut code = String::new();
        for (name, color) in &self.colors {
            // Convert kebab-case to valid Typst identifier
            let var_name = name.replace('-', "_");
            code.push_str(&format!("#let {} = rgb(\"{}\")\n", var_name, color));
        }
        if !code.is_empty() {
            code.push('\n');
        }
        code
    }

    /// Generate Typst code for page setup.
    pub fn generate_page_setup(&self) -> String {
        let Some(ref page) = self.page else {
            return String::new();
        };

        let mut args = vec![format!("paper: \"{}\"", page.paper)];

        if let Some(ref margin) = page.margin {
            let mut margin_parts = Vec::new();

            if let Some(ref all) = margin.all {
                margin_parts.push(format!("{}", all));
            } else {
                if let Some(ref top) = margin.top {
                    margin_parts.push(format!("top: {}", top));
                }
                if let Some(ref bottom) = margin.bottom {
                    margin_parts.push(format!("bottom: {}", bottom));
                }
                if let Some(ref left) = margin.left {
                    margin_parts.push(format!("left: {}", left));
                }
                if let Some(ref right) = margin.right {
                    margin_parts.push(format!("right: {}", right));
                }
            }

            if !margin_parts.is_empty() {
                if margin.all.is_some() {
                    args.push(format!("margin: {}", margin_parts[0]));
                } else {
                    args.push(format!("margin: ({})", margin_parts.join(", ")));
                }
            }
        }

        if let Some(ref numbering) = page.numbering {
            args.push(format!("numbering: \"{}\"", numbering));
        }

        format!("#set page({})\n", args.join(", "))
    }
}

/// Installed template entry in the cache.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledTemplate {
    /// Template name.
    pub name: String,

    /// Template version.
    pub version: String,

    /// Installation source.
    pub source: String,

    /// Path to template directory.
    pub path: PathBuf,

    /// Installation timestamp.
    pub installed_at: String,

    /// Template category.
    pub category: TemplateCategory,

    /// Description.
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_template(dir: &Path) {
        let template_toml = r##"
[template]
name = "test-template"
version = "1.0.0"
description = "A test template"
authors = ["Test Author"]
category = "article"

[colors]
primary = "#FF0000"
secondary = "#00FF00"

[page]
paper = "a4"

[page.margin]
top = "2cm"
bottom = "2cm"
"##;
        fs::write(dir.join("template.toml"), template_toml).unwrap();
        fs::write(dir.join("main.typ"), "// Test template\n").unwrap();
    }

    #[test]
    fn load_template() {
        let dir = TempDir::new().unwrap();
        create_test_template(dir.path());

        let template = Template::from_directory(dir.path()).unwrap();
        assert_eq!(template.name(), "test-template");
        assert_eq!(template.version(), "1.0.0");
        assert_eq!(template.template.category, TemplateCategory::Article);
    }

    #[test]
    fn generate_colors() {
        let dir = TempDir::new().unwrap();
        create_test_template(dir.path());

        let template = Template::from_directory(dir.path()).unwrap();
        let code = template.generate_color_definitions();

        assert!(code.contains("#let primary = rgb(\"#FF0000\")"));
        assert!(code.contains("#let secondary = rgb(\"#00FF00\")"));
    }

    #[test]
    fn generate_page_setup() {
        let dir = TempDir::new().unwrap();
        create_test_template(dir.path());

        let template = Template::from_directory(dir.path()).unwrap();
        let code = template.generate_page_setup();

        assert!(code.contains("paper: \"a4\""));
        assert!(code.contains("margin:"));
        assert!(code.contains("top: 2cm"));
    }

    #[test]
    fn validate_directory() {
        let dir = TempDir::new().unwrap();
        create_test_template(dir.path());

        assert!(Template::validate_directory(dir.path()).is_ok());
    }

    #[test]
    fn validate_missing_template_toml() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.typ"), "content").unwrap();

        let result = Template::validate_directory(dir.path());
        assert!(matches!(result, Err(Error::MissingFile { .. })));
    }

    #[test]
    fn template_category_display() {
        assert_eq!(TemplateCategory::Article.to_string(), "article");
        assert_eq!(TemplateCategory::Memo.to_string(), "memo");
    }
}
