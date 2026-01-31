//! Template schema definitions.
//!
//! Templates define both the visual styling and the required metadata
//! structure for a document type.

use crate::error::{Error, Result};
#[cfg(feature = "validation")]
use crate::schema::Schema;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A template definition combining visual styling and metadata schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSchema {
    /// Template identifier.
    pub name: String,

    /// Human-readable display name.
    pub display_name: String,

    /// Template description.
    pub description: String,

    /// Version string.
    pub version: String,

    /// Template author(s).
    pub authors: Vec<String>,

    /// Target use case (article, book, report, etc.).
    pub category: TemplateCategory,

    /// Metadata schema definition.
    #[serde(default)]
    pub metadata_schema: serde_json::Value,

    /// Required files (main.typ, etc.).
    #[serde(default)]
    pub required_files: Vec<String>,

    /// Optional configuration options.
    #[serde(default)]
    pub options: Vec<TemplateOption>,
}

impl TemplateSchema {
    /// Load a template schema from a directory.
    ///
    /// Expects a `template.toml` or `template.json` file.
    ///
    /// # Errors
    ///
    /// Returns an error if the template cannot be loaded.
    pub fn from_directory(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        // Try TOML first
        let toml_path = path.join("template.toml");
        if toml_path.exists() {
            let content = std::fs::read_to_string(&toml_path)?;
            let template: Self = toml::from_str(&content)?;
            return Ok(template);
        }

        // Try JSON
        let json_path = path.join("template.json");
        if json_path.exists() {
            let content = std::fs::read_to_string(&json_path)?;
            let template: Self = serde_json::from_str(&content)?;
            return Ok(template);
        }

        Err(Error::SchemaNotFound {
            name: path.display().to_string(),
        })
    }

    /// Get a validator for this template's metadata schema.
    ///
    /// # Errors
    ///
    /// Returns an error if the schema is invalid.
    #[cfg(feature = "validation")]
    pub fn metadata_validator(&self) -> Result<Schema> {
        Schema::new(&self.name, self.metadata_schema.clone())
    }

    /// Check if a file is required by this template.
    #[must_use]
    pub fn requires_file(&self, filename: &str) -> bool {
        self.required_files.iter().any(|f| f == filename)
    }
}

/// Template categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateCategory {
    /// Scientific article.
    Article,
    /// Book or book chapter.
    Book,
    /// Business report.
    Report,
    /// Curriculum vitae / Resume.
    Cv,
    /// Presentation slides.
    Presentation,
    /// Technical documentation.
    Documentation,
    /// Letter or correspondence.
    Letter,
    /// Other/custom.
    Other,
}

impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Article => write!(f, "article"),
            Self::Book => write!(f, "book"),
            Self::Report => write!(f, "report"),
            Self::Cv => write!(f, "cv"),
            Self::Presentation => write!(f, "presentation"),
            Self::Documentation => write!(f, "documentation"),
            Self::Letter => write!(f, "letter"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// A configurable option for a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateOption {
    /// Option name/key.
    pub name: String,

    /// Human-readable label.
    pub label: String,

    /// Option description.
    pub description: String,

    /// Type of option.
    pub option_type: OptionType,

    /// Default value.
    pub default: serde_json::Value,

    /// Whether this option is required.
    #[serde(default)]
    pub required: bool,
}

/// Types of template options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OptionType {
    /// Text string.
    String,
    /// Boolean flag.
    Boolean,
    /// Integer number.
    Integer,
    /// Floating point number.
    Number,
    /// Selection from a list.
    Enum(Vec<String>),
    /// Color value.
    Color,
    /// File path.
    File,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_template() -> TemplateSchema {
        TemplateSchema {
            name: "ieee-article".to_string(),
            display_name: "IEEE Article".to_string(),
            description: "Template for IEEE journal articles".to_string(),
            version: "1.0.0".to_string(),
            authors: vec!["NovaType Team".to_string()],
            category: TemplateCategory::Article,
            metadata_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" }
                },
                "required": ["title"]
            }),
            required_files: vec!["main.typ".to_string()],
            options: vec![],
        }
    }

    #[test]
    fn template_serialization() {
        let template = sample_template();
        let json = serde_json::to_string_pretty(&template).unwrap();
        assert!(json.contains("ieee-article"));
        assert!(json.contains("IEEE Article"));
    }

    #[test]
    fn template_deserialization() {
        let json = r#"{
            "name": "test",
            "display_name": "Test Template",
            "description": "A test template",
            "version": "0.1.0",
            "authors": ["Test Author"],
            "category": "article",
            "metadata_schema": {},
            "required_files": ["main.typ"],
            "options": []
        }"#;

        let template: TemplateSchema = serde_json::from_str(json).unwrap();
        assert_eq!(template.name, "test");
        assert_eq!(template.category, TemplateCategory::Article);
    }

    #[test]
    fn template_requires_file() {
        let template = sample_template();
        assert!(template.requires_file("main.typ"));
        assert!(!template.requires_file("other.typ"));
    }

    #[test]
    fn template_category_display() {
        assert_eq!(TemplateCategory::Article.to_string(), "article");
        assert_eq!(TemplateCategory::Book.to_string(), "book");
        assert_eq!(TemplateCategory::Cv.to_string(), "cv");
    }

    #[test]
    #[cfg(feature = "validation")]
    fn template_metadata_validator() {
        let template = sample_template();
        let validator = template.metadata_validator().unwrap();

        let valid = serde_json::json!({ "title": "Test" });
        let invalid = serde_json::json!({});

        assert!(validator.is_valid(&valid));
        assert!(!validator.is_valid(&invalid));
    }
}
