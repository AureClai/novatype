//! Template management command.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use nova_template::{TemplateCache, TemplateInstaller, TemplateSource};
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
    /// List installed templates.
    List,

    /// Show template details.
    Info {
        /// Template name.
        name: String,
    },

    /// Install a template from local path, GitHub, or URL.
    Install {
        /// Template source (path, github:owner/repo, or URL).
        source: String,
    },

    /// Remove an installed template.
    Remove {
        /// Template name.
        name: String,
    },

    /// Create a new template scaffold.
    New {
        /// Template name.
        name: String,

        /// Directory to create template in.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show cache information.
    Cache,
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
        TemplateCommands::Remove { name } => remove_template(&name).await,
        TemplateCommands::New { name, output } => create_template(&name, output).await,
        TemplateCommands::Cache => show_cache_info().await,
    }
}

/// List installed templates.
async fn list_templates() -> Result<()> {
    info!("Listing installed templates");

    let cache = TemplateCache::new().context("Failed to open template cache")?;
    let templates = cache.list_installed();

    if templates.is_empty() {
        println!("No templates installed.");
        println!();
        println!("Install a template with:");
        println!("  nova template install <source>");
        println!();
        println!("Sources:");
        println!("  ./path/to/template     Local directory");
        println!("  github:owner/repo      GitHub repository");
        println!("  owner/repo             GitHub shorthand");
        println!("  https://...            URL to ZIP file");
        return Ok(());
    }

    println!("Installed templates ({}):\n", templates.len());

    for template in templates {
        println!(
            "  {} ({})",
            template.name,
            template.version
        );
        if let Some(ref desc) = template.description {
            println!("    {}", desc);
        }
        println!("    Category: {}", template.category);
        println!("    Source: {}", template.source);
        println!();
    }

    Ok(())
}

/// Show template information.
async fn show_template_info(name: &str) -> Result<()> {
    info!("Showing template info: {}", name);

    let cache = TemplateCache::new().context("Failed to open template cache")?;

    let installed = cache.get(name).ok_or_else(|| {
        anyhow::anyhow!("Template not found: {}. Use 'nova template list' to see installed templates.", name)
    })?;

    let template = cache
        .load_template(name)
        .context("Failed to load template")?;

    println!("Template: {}", template.display_name());
    println!("Name:     {}", template.name());
    println!("Version:  {}", template.version());
    println!("Category: {}", template.template.category);
    println!();

    if let Some(ref desc) = template.template.description {
        println!("Description:");
        println!("  {}", desc);
        println!();
    }

    if !template.template.authors.is_empty() {
        println!("Authors:");
        for author in &template.template.authors {
            println!("  - {}", author);
        }
        println!();
    }

    if let Some(ref license) = template.template.license {
        println!("License: {}", license);
        println!();
    }

    if !template.schema.required.is_empty() {
        println!("Required frontmatter fields:");
        for field in &template.schema.required {
            println!("  - {}", field);
        }
        println!();
    }

    if !template.colors.is_empty() {
        println!("Colors:");
        for (name, color) in &template.colors {
            println!("  {}: {}", name, color);
        }
        println!();
    }

    println!("Installed from: {}", installed.source);
    println!("Installed at:   {}", installed.installed_at);
    println!("Location:       {}", installed.path.display());

    Ok(())
}

/// Install a template.
async fn install_template(source: &str) -> Result<()> {
    info!("Installing template from: {}", source);

    let source = TemplateSource::parse(source).context("Invalid template source")?;
    println!("Source: {}", source.display_name());

    let mut cache = TemplateCache::new().context("Failed to open template cache")?;
    let installer = TemplateInstaller::new();

    println!("Installing...");
    let installed = installer
        .install(&mut cache, &source)
        .await
        .context("Failed to install template")?;

    println!();
    println!("Installed template: {}", installed.name);
    println!("  Version:  {}", installed.version);
    println!("  Category: {}", installed.category);
    println!("  Location: {}", installed.path.display());
    println!();
    println!("Use with: nova init my-project --template {}", installed.name);

    Ok(())
}

/// Remove an installed template.
async fn remove_template(name: &str) -> Result<()> {
    info!("Removing template: {}", name);

    let mut cache = TemplateCache::new().context("Failed to open template cache")?;

    cache.remove(name).context("Failed to remove template")?;

    println!("Removed template: {}", name);

    Ok(())
}

/// Create a new template scaffold.
async fn create_template(name: &str, output: Option<PathBuf>) -> Result<()> {
    info!("Creating template: {}", name);

    let output_dir = output.unwrap_or_else(|| PathBuf::from(name));

    if output_dir.exists() {
        anyhow::bail!("Directory already exists: {:?}", output_dir);
    }

    std::fs::create_dir_all(&output_dir)?;

    // Create template.toml
    let template_toml = format!(
        r##"[template]
name = "{name}"
display_name = "{name}"
description = "A custom NovaType template"
version = "0.1.0"
authors = ["Your Name"]
category = "article"
license = "MIT"

[schema]
required = ["title"]

[colors]
primary = "#1a365d"
secondary = "#e2e8f0"
accent = "#3182ce"

[page]
paper = "a4"
numbering = "1"

[page.margin]
top = "2.5cm"
bottom = "2.5cm"
left = "2cm"
right = "2cm"

[header]
right = "{{title}}"

[footer]
center = "{{page}}"
"##
    );
    std::fs::write(output_dir.join("template.toml"), template_toml)?;

    // Create main.typ
    let main_typ = r#"// Template: main.typ
// This is the main entry point for the template

// Import template configuration (colors, etc.)
#import "lib.typ": *

#let template(
  title: none,
  authors: (),
  date: none,
  body,
) = {
  // Document metadata
  set document(title: title, author: authors.map(a => a.name))

  // Page setup
  set page(
    paper: "a4",
    margin: (top: 2.5cm, bottom: 2.5cm, left: 2cm, right: 2cm),
    header: align(right)[
      #set text(size: 9pt, fill: primary)
      #title
    ],
    footer: align(center)[
      #counter(page).display("1")
    ],
  )

  // Text settings
  set text(font: "New Computer Modern", size: 11pt)
  set par(justify: true)

  // Heading styles
  show heading.where(level: 1): it => {
    set text(fill: primary)
    v(1em)
    it
    v(0.5em)
  }

  // Title block
  align(center)[
    #text(20pt, weight: "bold", fill: primary)[#title]

    #v(1em)

    #for author in authors [
      #author.name \
      #if "affiliation" in author [
        #text(size: 10pt, style: "italic")[#author.affiliation] \
      ]
    ]

    #v(0.5em)

    #if date != none [
      #text(size: 10pt)[#date]
    ]
  ]

  v(2em)

  // Document body
  body
}
"#;
    std::fs::write(output_dir.join("main.typ"), main_typ)?;

    // Create lib.typ for helpers
    let lib_typ = r##"// Template: lib.typ
// Helper functions and shared definitions

// Colors from template.toml (override these or import from config)
#let primary = rgb("#1a365d")
#let secondary = rgb("#e2e8f0")
#let accent = rgb("#3182ce")

// Helper function for callout boxes
#let callout(title: none, body) = {
  block(
    fill: secondary,
    inset: 12pt,
    radius: 4pt,
    width: 100%,
    [
      #if title != none [
        #text(weight: "bold", fill: primary)[#title]
        #v(0.5em)
      ]
      #body
    ]
  )
}

// Helper function for code blocks
#let code-block(lang: none, code) = {
  block(
    fill: luma(245),
    inset: 10pt,
    radius: 4pt,
    width: 100%,
    [
      #if lang != none [
        #text(size: 8pt, fill: luma(100))[#lang]
        #v(0.3em)
      ]
      #raw(code)
    ]
  )
}
"##;
    std::fs::write(output_dir.join("lib.typ"), lib_typ)?;

    println!("Created template: {:?}", output_dir);
    println!();
    println!("Files created:");
    println!("  template.toml  - Template configuration");
    println!("  main.typ       - Main template entry point");
    println!("  lib.typ        - Helper functions");
    println!();
    println!("Next steps:");
    println!("  1. Edit template.toml to customize metadata");
    println!("  2. Modify main.typ for your layout");
    println!("  3. Install with: nova template install {:?}", output_dir);

    Ok(())
}

/// Show cache information.
async fn show_cache_info() -> Result<()> {
    let cache = TemplateCache::new().context("Failed to open template cache")?;
    let stats = cache.stats();

    println!("Template cache:");
    println!("  Location:  {}", cache.cache_dir().display());
    println!("  Templates: {}", stats.template_count);
    println!("  Size:      {}", format_bytes(stats.total_size));

    Ok(())
}

/// Format bytes as human-readable string.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn list_templates_succeeds() {
        // This might fail if cache dir can't be created, but shouldn't panic
        let _ = list_templates().await;
    }

    #[tokio::test]
    async fn template_info_not_found() {
        let result = show_template_info("nonexistent-template-xyz").await;
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
        assert!(template_dir.join("lib.typ").exists());
    }

    #[test]
    fn format_bytes_works() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
}
