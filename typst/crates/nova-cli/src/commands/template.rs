//! Template management command.

use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::info;

/// Arguments for the template command.
#[derive(Args, Debug)]
pub struct TemplateArgs {
    /// Template subcommand.
    #[command(subcommand)]
    pub command: TemplateCommands,
}

/// Template subcommands.
#[derive(Subcommand, Debug)]
pub enum TemplateCommands {
    /// List available templates.
    List,

    /// Show template details.
    Info {
        /// Template name.
        name: String,
    },

    /// Install a template from a package.
    Install {
        /// Template package name or path.
        source: String,
    },

    /// Create a new template.
    New {
        /// Template name.
        name: String,

        /// Directory to create template in.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// Execute the template command.
///
/// # Errors
///
/// Returns an error if the command fails.
pub async fn template(args: TemplateArgs) -> Result<()> {
    match args.command {
        TemplateCommands::List => list_templates().await,
        TemplateCommands::Info { name } => show_template_info(&name).await,
        TemplateCommands::Install { source } => install_template(&source).await,
        TemplateCommands::New { name, output } => create_template(&name, output).await,
    }
}

/// List available templates.
async fn list_templates() -> Result<()> {
    info!("Listing templates");

    println!("Built-in templates:");
    println!();
    println!("  article        - Simple article template");
    println!("  ieee-article   - IEEE journal article format");
    println!("  nature-article - Nature journal format");
    println!("  report         - Business/technical report");
    println!("  book           - Book with chapters");
    println!("  presentation   - Slide presentation");
    println!("  cv             - Curriculum vitae / resume");
    println!();
    println!("Use 'nova template info <name>' for details.");

    Ok(())
}

/// Show template information.
async fn show_template_info(name: &str) -> Result<()> {
    info!("Showing template info: {}", name);

    let info = match name {
        "article" => TemplateInfo {
            name: "article",
            version: "1.0.0",
            description: "Simple article template for general documents",
            author: "NovaType Team",
            required_fields: vec!["title"],
            optional_fields: vec!["authors", "date", "abstract"],
        },
        "ieee-article" => TemplateInfo {
            name: "ieee-article",
            version: "1.0.0",
            description: "IEEE journal article format with proper styling",
            author: "NovaType Team",
            required_fields: vec!["title", "authors"],
            optional_fields: vec!["abstract", "keywords", "date"],
        },
        "nature-article" => TemplateInfo {
            name: "nature-article",
            version: "1.0.0",
            description: "Nature journal article format",
            author: "NovaType Team",
            required_fields: vec!["title", "authors"],
            optional_fields: vec!["abstract", "keywords"],
        },
        "report" => TemplateInfo {
            name: "report",
            version: "1.0.0",
            description: "Business or technical report template",
            author: "NovaType Team",
            required_fields: vec!["title"],
            optional_fields: vec!["authors", "date", "organization"],
        },
        "book" => TemplateInfo {
            name: "book",
            version: "1.0.0",
            description: "Book template with chapters support",
            author: "NovaType Team",
            required_fields: vec!["title"],
            optional_fields: vec!["authors", "publisher", "isbn"],
        },
        _ => {
            anyhow::bail!("Template not found: {}", name);
        }
    };

    println!("Template: {}", info.name);
    println!("Version:  {}", info.version);
    println!("Author:   {}", info.author);
    println!();
    println!("Description:");
    println!("  {}", info.description);
    println!();
    println!("Required fields:");
    for field in &info.required_fields {
        println!("  - {}", field);
    }
    println!();
    println!("Optional fields:");
    for field in &info.optional_fields {
        println!("  - {}", field);
    }

    Ok(())
}

/// Install a template.
async fn install_template(source: &str) -> Result<()> {
    info!("Installing template: {}", source);

    // TODO: Implement template installation
    println!("Template installation not yet implemented.");
    println!("Source: {}", source);

    Ok(())
}

/// Create a new template.
async fn create_template(name: &str, output: Option<PathBuf>) -> Result<()> {
    info!("Creating template: {}", name);

    let output_dir = output.unwrap_or_else(|| PathBuf::from(name));

    if output_dir.exists() {
        anyhow::bail!("Directory already exists: {:?}", output_dir);
    }

    std::fs::create_dir_all(&output_dir)?;

    // Create template.toml
    let template_toml = format!(
        r#"[template]
name = "{name}"
display_name = "{name}"
description = "A custom NovaType template"
version = "0.1.0"
authors = ["Your Name"]
category = "article"

[metadata_schema]
# JSON Schema for document metadata
required = ["title"]

[metadata_schema.properties.title]
type = "string"
description = "Document title"

[metadata_schema.properties.authors]
type = "array"
description = "Document authors"
"#
    );
    std::fs::write(output_dir.join("template.toml"), template_toml)?;

    // Create main.typ
    let main_typ = r#"// Template main file
// This file defines the document structure and styling

#let template(
  title: none,
  authors: (),
  date: none,
  body,
) = {
  // Document settings
  set document(title: title, author: authors.map(a => a.name))
  set page(paper: "a4")
  set text(font: "New Computer Modern", size: 11pt)

  // Title
  align(center)[
    #text(17pt, weight: "bold")[#title]

    #v(1em)

    #for author in authors [
      #author.name \
    ]

    #v(1em)

    #if date != none [#date]
  ]

  v(2em)

  // Body
  body
}
"#;
    std::fs::write(output_dir.join("main.typ"), main_typ)?;

    println!("Created template: {:?}", output_dir);
    println!("  - template.toml");
    println!("  - main.typ");

    Ok(())
}

/// Template information structure.
struct TemplateInfo {
    name: &'static str,
    version: &'static str,
    description: &'static str,
    author: &'static str,
    required_fields: Vec<&'static str>,
    optional_fields: Vec<&'static str>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn list_templates_succeeds() {
        let result = list_templates().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn template_info_valid() {
        let result = show_template_info("article").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn template_info_invalid() {
        let result = show_template_info("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_template_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("my-template");

        let result = create_template("my-template", Some(template_dir.clone())).await;
        assert!(result.is_ok());
        assert!(template_dir.join("template.toml").exists());
        assert!(template_dir.join("main.typ").exists());
    }
}
