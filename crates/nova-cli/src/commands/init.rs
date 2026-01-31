//! Init command implementation.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use tracing::info;

/// Arguments for the init command.
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Project name / directory.
    #[arg(required = true)]
    pub name: String,

    /// Template to use.
    #[arg(short, long, default_value = "article")]
    pub template: String,

    /// Don't create a git repository.
    #[arg(long)]
    pub no_git: bool,
}

/// Execute the init command.
///
/// # Errors
///
/// Returns an error if initialization fails.
pub async fn init(args: InitArgs) -> Result<()> {
    info!("Initializing project: {}", args.name);

    let project_dir = PathBuf::from(&args.name);

    // Check if directory already exists
    if project_dir.exists() {
        anyhow::bail!("Directory already exists: {:?}", project_dir);
    }

    // Create project directory
    std::fs::create_dir_all(&project_dir)
        .with_context(|| format!("Failed to create directory: {:?}", project_dir))?;

    // Create main document
    let main_content = generate_main_document(&args.template);
    let main_path = project_dir.join("main.typ");
    std::fs::write(&main_path, main_content)
        .with_context(|| format!("Failed to write {:?}", main_path))?;

    // Create nova.toml configuration
    let config_content = generate_config(&args.name, &args.template);
    let config_path = project_dir.join("nova.toml");
    std::fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write {:?}", config_path))?;

    // Create bibliography file
    let bib_path = project_dir.join("references.bib");
    std::fs::write(&bib_path, "% Bibliography file\n")
        .with_context(|| format!("Failed to write {:?}", bib_path))?;

    // Create .gitignore
    let gitignore_content = generate_gitignore();
    let gitignore_path = project_dir.join(".gitignore");
    std::fs::write(&gitignore_path, gitignore_content)
        .with_context(|| format!("Failed to write {:?}", gitignore_path))?;

    // Initialize git repository
    if !args.no_git {
        init_git_repo(&project_dir)?;
    }

    println!("Created project: {}", args.name);
    println!("  - main.typ");
    println!("  - nova.toml");
    println!("  - references.bib");
    println!("  - .gitignore");
    println!("\nGet started:");
    println!("  cd {}", args.name);
    println!("  nova compile main.typ");

    Ok(())
}

/// Generate the main document content based on template.
fn generate_main_document(template: &str) -> String {
    match template {
        "article" | "ieee-article" => r#"---
title: "My Article"
authors:
  - name: Author Name
    email: author@example.com
    affiliation: University
date: auto
template: ieee-article
citation_style: ieee
bibliography:
  - references.bib
---

= Introduction

Your introduction here.

= Methods

Describe your methods.

= Results

Present your results.

= Discussion

Discuss your findings.

= Conclusion

Summarize your conclusions.
"#
        .to_string(),
        "report" => r#"---
title: "My Report"
authors:
  - name: Author Name
date: auto
template: report
---

= Executive Summary

Brief summary of the report.

= Introduction

Background and context.

= Analysis

Main content of the report.

= Recommendations

Key recommendations.

= Appendix

Supporting materials.
"#
        .to_string(),
        "book" => r#"---
title: "My Book"
authors:
  - name: Author Name
template: book
---

= Preface

Welcome to this book.

= Chapter 1: Introduction

#include "chapters/01-introduction.typ"

= Chapter 2: Background

#include "chapters/02-background.typ"
"#
        .to_string(),
        _ => {
            format!(
                r#"---
title: "Untitled Document"
template: {}
---

= Introduction

Start writing here.
"#,
                template
            )
        }
    }
}

/// Generate nova.toml configuration.
fn generate_config(name: &str, template: &str) -> String {
    format!(
        r#"[project]
name = "{name}"
version = "0.1.0"

[document]
main = "main.typ"
template = "{template}"

[output]
format = "pdf"
directory = "build"

[citations]
style = "ieee"
bibliography = ["references.bib"]
"#
    )
}

/// Generate .gitignore content.
fn generate_gitignore() -> String {
    r#"# Build output
/build/
*.pdf

# Editor files
.vscode/
.idea/
*.swp
*~

# OS files
.DS_Store
Thumbs.db

# NovaType cache
.nova/
"#
    .to_string()
}

/// Initialize a git repository.
fn init_git_repo(path: &PathBuf) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            info!("Initialized git repository");
            Ok(())
        }
        Ok(_) => {
            // Git failed but we don't want to fail the whole init
            info!("Could not initialize git repository");
            Ok(())
        }
        Err(_) => {
            // Git not available
            info!("Git not available, skipping repository initialization");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn init_creates_project_structure() {
        let temp_dir = TempDir::new().unwrap();
        let project_name = temp_dir.path().join("test-project");

        let args = InitArgs {
            name: project_name.to_string_lossy().to_string(),
            template: "article".to_string(),
            no_git: true,
        };

        let result = init(args).await;
        assert!(result.is_ok());

        assert!(project_name.join("main.typ").exists());
        assert!(project_name.join("nova.toml").exists());
        assert!(project_name.join("references.bib").exists());
        assert!(project_name.join(".gitignore").exists());
    }

    #[tokio::test]
    async fn init_fails_if_exists() {
        let temp_dir = TempDir::new().unwrap();
        let project_name = temp_dir.path().to_string_lossy().to_string();

        let args = InitArgs {
            name: project_name,
            template: "article".to_string(),
            no_git: true,
        };

        let result = init(args).await;
        assert!(result.is_err());
    }

    #[test]
    fn generate_main_document_article() {
        let content = generate_main_document("article");
        assert!(content.contains("title:"));
        assert!(content.contains("Introduction"));
    }

    #[test]
    fn generate_config_contains_name() {
        let config = generate_config("my-project", "ieee-article");
        assert!(config.contains("my-project"));
        assert!(config.contains("ieee-article"));
    }
}
