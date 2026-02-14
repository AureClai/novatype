//! Init command implementation.

use anyhow::{Context, Result};
use clap::Args;
use std::path::{Path, PathBuf};
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

    /// Include Python figure support (creates figures/ directory with examples).
    #[arg(long)]
    pub python: bool,
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
    let main_content = generate_main_document(&args.template, args.python);
    let main_path = project_dir.join("main.typ");
    std::fs::write(&main_path, main_content)
        .with_context(|| format!("Failed to write {:?}", main_path))?;

    // Create nova.toml configuration
    let config_content = generate_config(&args.name, &args.template, args.python);
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

    // Create Python figure support if requested
    if args.python {
        create_python_support(&project_dir)?;
    }

    // Initialize git repository
    if !args.no_git {
        init_git_repo(&project_dir)?;
    }

    println!("Created project: {}", args.name);
    println!("  - main.typ");
    println!("  - nova.toml");
    println!("  - references.bib");
    println!("  - .gitignore");
    if args.python {
        println!("  - figures/");
        println!("    - __init__.py");
        println!("    - example.py");
    }
    println!("\nGet started:");
    println!("  cd {}", args.name);
    if args.python {
        println!("  pip install nova-typst  # Install Python package");
    }
    println!("  nova compile main.typ");

    Ok(())
}

/// Generate the main document content based on template.
fn generate_main_document(template: &str, with_python: bool) -> String {
    let python_import = if with_python {
        "\n#import \".nova/pyplot.typ\": pyplot\n"
    } else {
        ""
    };

    let python_example = if with_python {
        r#"

= Results

Here's an example Python-generated figure:

#figure(
  pyplot("example-plot", width: 80%),
  caption: [Example plot generated from Python]
) <fig:example>

Add more figures by creating functions in `figures/` with the `@nova.figure` decorator.
"#
    } else {
        r#"

= Results

Present your results.
"#
    };

    match template {
        "article" | "ieee-article" => format!(
            r#"---
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
{python_import}
= Introduction

Your introduction here.

= Methods

Describe your methods.
{python_example}
= Discussion

Discuss your findings.

= Conclusion

Summarize your conclusions.
"#
        ),
        "report" => format!(
            r#"---
title: "My Report"
authors:
  - name: Author Name
date: auto
template: report
---
{python_import}
= Executive Summary

Brief summary of the report.

= Introduction

Background and context.

= Analysis

Main content of the report.
{python_example}
= Recommendations

Key recommendations.

= Appendix

Supporting materials.
"#
        ),
        "book" => format!(
            r#"---
title: "My Book"
authors:
  - name: Author Name
template: book
---
{python_import}
= Preface

Welcome to this book.

= Chapter 1: Introduction

#include "chapters/01-introduction.typ"

= Chapter 2: Background

#include "chapters/02-background.typ"
"#
        ),
        _ => {
            format!(
                r#"---
title: "Untitled Document"
template: {template}
---
{python_import}
= Introduction

Start writing here.
{python_example}"#
            )
        }
    }
}

/// Generate nova.toml configuration.
fn generate_config(name: &str, template: &str, with_python: bool) -> String {
    let mut config = format!(
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
    );

    if with_python {
        config.push_str(
            r#"
[python]
# Python executable (use "python3" on Unix)
python = "python"
# Directory containing figure scripts
figures_dir = "figures"
# Cache directory for generated SVGs
cache_dir = ".nova/cache"
# Timeout for figure generation (seconds)
timeout = 60
"#,
        );
    }

    config
}

/// Generate .gitignore content.
fn generate_gitignore() -> String {
    r#"# Build output
/build/
*.pdf

# NovaType cache
.nova/

# Python
__pycache__/
*.pyc
*.pyo
.venv/

# Editor files
.vscode/
.idea/
*.swp
*~

# OS files
.DS_Store
Thumbs.db
"#
    .to_string()
}

/// Create Python figure support files.
fn create_python_support(project_dir: &Path) -> Result<()> {
    // Create figures directory
    let figures_dir = project_dir.join("figures");
    std::fs::create_dir_all(&figures_dir)
        .with_context(|| format!("Failed to create {:?}", figures_dir))?;

    // Create __init__.py
    let init_path = figures_dir.join("__init__.py");
    std::fs::write(&init_path, "# Python figures for NovaType\n")
        .with_context(|| format!("Failed to write {:?}", init_path))?;

    // Create example.py with sample figure
    let example_content = r#""""
Example Python figures for NovaType.

Run: nova compile main.typ

Only requires matplotlib (no numpy needed for this example).
For more complex figures, install numpy: pip install numpy
"""

import math

import nova
import matplotlib.pyplot as plt


@nova.figure("example-plot")
def plot_example():
    """Generate an example sine wave plot."""
    x = [i * 0.05 for i in range(126)]  # 0 to 2*pi
    y = [math.sin(v) for v in x]

    fig, ax = plt.subplots(figsize=(8, 4))
    ax.plot(x, y, "b-", linewidth=2, label=r"$\sin(x)$")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.set_title("Example Plot")
    ax.legend()
    ax.grid(True, alpha=0.3)
    fig.tight_layout()
"#;
    let example_path = figures_dir.join("example.py");
    std::fs::write(&example_path, example_content)
        .with_context(|| format!("Failed to write {:?}", example_path))?;

    // Note: pyplot.typ is automatically provisioned by nova compile
    // No need to create it here - it's managed by the build system

    Ok(())
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
            python: false,
        };

        let result = init(args).await;
        assert!(result.is_ok());

        assert!(project_name.join("main.typ").exists());
        assert!(project_name.join("nova.toml").exists());
        assert!(project_name.join("references.bib").exists());
        assert!(project_name.join(".gitignore").exists());
    }

    #[tokio::test]
    async fn init_with_python_creates_figures() {
        let temp_dir = TempDir::new().unwrap();
        let project_name = temp_dir.path().join("python-project");

        let args = InitArgs {
            name: project_name.to_string_lossy().to_string(),
            template: "article".to_string(),
            no_git: true,
            python: true,
        };

        let result = init(args).await;
        assert!(result.is_ok());

        // pyplot.typ is now provisioned by nova compile, not init
        assert!(project_name.join("figures/__init__.py").exists());
        assert!(project_name.join("figures/example.py").exists());
    }

    #[tokio::test]
    async fn init_fails_if_exists() {
        let temp_dir = TempDir::new().unwrap();
        let project_name = temp_dir.path().to_string_lossy().to_string();

        let args = InitArgs {
            name: project_name,
            template: "article".to_string(),
            no_git: true,
            python: false,
        };

        let result = init(args).await;
        assert!(result.is_err());
    }

    #[test]
    fn generate_main_document_article() {
        let content = generate_main_document("article", false);
        assert!(content.contains("title:"));
        assert!(content.contains("Introduction"));
    }

    #[test]
    fn generate_main_document_with_python() {
        let content = generate_main_document("article", true);
        assert!(content.contains(".nova/pyplot.typ"));
        assert!(content.contains("pyplot(\"example-plot\""));
    }

    #[test]
    fn generate_config_contains_name() {
        let config = generate_config("my-project", "ieee-article", false);
        assert!(config.contains("my-project"));
        assert!(config.contains("ieee-article"));
    }

    #[test]
    fn generate_config_with_python() {
        let config = generate_config("my-project", "article", true);
        assert!(config.contains("[python]"));
        assert!(config.contains("figures_dir"));
    }

    #[test]
    fn gitignore_includes_python_patterns() {
        let gitignore = generate_gitignore();
        assert!(gitignore.contains("__pycache__/"));
        assert!(gitignore.contains("*.pyc"));
        assert!(gitignore.contains(".venv/"));
        assert!(gitignore.contains(".nova/"));
    }
}
