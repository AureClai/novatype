//! Compile command implementation.

use crate::OutputFormat;
use anyhow::{Context, Result};
use clap::Args;
use nova_schema::FrontmatterParser;
use novatype_core::{compile_pdf, compile_svg, set_font_paths, NativeWorld};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Arguments for the compile command.
#[derive(Args, Debug)]
pub struct CompileArgs {
    /// Input file to compile.
    #[arg(required = true)]
    pub input: PathBuf,

    /// Output file path.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output format.
    #[arg(short, long, value_enum, default_value = "pdf")]
    pub format: OutputFormat,

    /// Root directory for resolving paths.
    #[arg(long)]
    pub root: Option<PathBuf>,

    /// Open the output file after compilation.
    #[arg(long)]
    pub open: bool,

    /// Font paths to include.
    #[arg(long = "font-path")]
    pub font_paths: Vec<PathBuf>,
}

/// Execute the compile command.
///
/// # Errors
///
/// Returns an error if compilation fails.
pub async fn compile(args: CompileArgs) -> Result<()> {
    info!("Compiling {:?}", args.input);

    // Validate input file exists
    if !args.input.exists() {
        anyhow::bail!("Input file not found: {:?}", args.input);
    }

    // Determine output path
    let output_path = args.output.unwrap_or_else(|| {
        let stem = args.input.file_stem().unwrap_or_default();
        let ext = match args.format {
            OutputFormat::Pdf => "pdf",
            OutputFormat::Svg => "svg",
            OutputFormat::Png => "png",
        };
        args.input
            .with_file_name(format!("{}.{}", stem.to_string_lossy(), ext))
    });

    // Read input file
    let content = std::fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read {:?}", args.input))?;

    // Check for frontmatter and preprocess if needed
    let processed_content = preprocess_content(&content)?;

    // Determine root directory
    let root = args.root.unwrap_or_else(|| {
        args.input
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    });

    // Set custom font paths if provided
    if !args.font_paths.is_empty() {
        debug!("Setting font paths: {:?}", args.font_paths);
        set_font_paths(args.font_paths.clone());
    }

    // Create the native world
    debug!("Creating native world with root: {:?}", root);
    let world = NativeWorld::from_source(&processed_content, &root);

    // Compile based on output format
    info!("Compiling document...");
    match args.format {
        OutputFormat::Pdf => {
            let pdf = compile_pdf(&world)
                .map_err(|errors| anyhow::anyhow!("Compilation failed:\n{}", errors.join("\n")))?;

            std::fs::write(&output_path, &pdf)
                .with_context(|| format!("Failed to write {:?}", output_path))?;
        }
        OutputFormat::Svg => {
            let pages = compile_svg(&world)
                .map_err(|errors| anyhow::anyhow!("Compilation failed:\n{}", errors.join("\n")))?;

            if pages.len() == 1 {
                // Single page, write directly
                std::fs::write(&output_path, &pages[0])
                    .with_context(|| format!("Failed to write {:?}", output_path))?;
            } else {
                // Multiple pages, write numbered files
                let stem = output_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                let parent = output_path.parent().unwrap_or(Path::new("."));
                for (i, page) in pages.iter().enumerate() {
                    let page_path = parent.join(format!("{}-{}.svg", stem, i + 1));
                    std::fs::write(&page_path, page)
                        .with_context(|| format!("Failed to write {:?}", page_path))?;
                }
            }
        }
        OutputFormat::Png => {
            // PNG output: compile to SVG first, then convert
            // For now, just produce SVG and warn
            let pages = compile_svg(&world)
                .map_err(|errors| anyhow::anyhow!("Compilation failed:\n{}", errors.join("\n")))?;

            let svg_path = output_path.with_extension("svg");
            if pages.len() == 1 {
                std::fs::write(&svg_path, &pages[0])
                    .with_context(|| format!("Failed to write {:?}", svg_path))?;
            } else {
                let stem = svg_path.file_stem().unwrap_or_default().to_string_lossy();
                let parent = svg_path.parent().unwrap_or(Path::new("."));
                for (i, page) in pages.iter().enumerate() {
                    let page_path = parent.join(format!("{}-{}.svg", stem, i + 1));
                    std::fs::write(&page_path, page)
                        .with_context(|| format!("Failed to write {:?}", page_path))?;
                }
            }
            eprintln!("Note: PNG output is not yet implemented. SVG file(s) created instead.");
        }
    }

    println!("Compiled: {:?} -> {:?}", args.input, output_path);

    // Open if requested
    if args.open {
        open_file(&output_path)?;
    }

    Ok(())
}

/// Preprocess content to handle NovaType frontmatter.
///
/// If the content has YAML/TOML frontmatter, strip it.
fn preprocess_content(content: &str) -> Result<String> {
    let parser = FrontmatterParser::new();

    // Try to extract frontmatter
    if let Some((_raw_frontmatter, _format)) = parser.extract_raw(content) {
        debug!("Found frontmatter, preprocessing...");

        // Parse to get remaining content
        let result: std::result::Result<(Option<serde_json::Value>, &str), _> =
            parser.parse(content);

        if let Ok((metadata, remaining)) = result {
            // Log metadata if present
            if let Some(meta) = &metadata {
                if let Some(title) = meta.get("title") {
                    debug!("Document title: {}", title);
                }
            }

            return Ok(remaining.to_string());
        }
    }

    // No frontmatter or parsing failed, use original content
    Ok(content.to_string())
}

/// Open a file with the system's default application.
fn open_file(path: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.display().to_string()])
            .spawn()
            .context("Failed to open file")?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .context("Failed to open file")?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .context("Failed to open file")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn compile_missing_file_fails() {
        let args = CompileArgs {
            input: PathBuf::from("/nonexistent/file.typ"),
            output: None,
            format: OutputFormat::Pdf,
            root: None,
            open: false,
            font_paths: vec![],
        };

        let result = compile(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn compile_creates_output() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.typ");
        let output_path = temp_dir.path().join("test.pdf");

        // Write valid Typst content
        std::fs::write(&input_path, "Hello, world!").unwrap();

        let args = CompileArgs {
            input: input_path,
            output: Some(output_path.clone()),
            format: OutputFormat::Pdf,
            root: None,
            open: false,
            font_paths: vec![],
        };

        let result = compile(args).await;
        assert!(result.is_ok(), "Compilation failed: {:?}", result);
        assert!(output_path.exists(), "Output file not created");

        // Verify PDF magic bytes
        let content = std::fs::read(&output_path).unwrap();
        assert_eq!(&content[0..5], b"%PDF-", "Not a valid PDF");
    }

    #[test]
    fn preprocess_strips_frontmatter() {
        let content = r#"---
title: Test
---
Hello content"#;

        let processed = preprocess_content(content).unwrap();

        assert!(!processed.contains("---"));
        assert!(processed.contains("Hello content"));
    }

    #[test]
    fn preprocess_no_frontmatter() {
        let content = "Just regular content";
        let processed = preprocess_content(content).unwrap();
        assert_eq!(processed, content);
    }

    #[tokio::test]
    async fn compile_to_svg() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.typ");
        let output_path = temp_dir.path().join("test.svg");

        std::fs::write(&input_path, "Hello SVG!").unwrap();

        let args = CompileArgs {
            input: input_path,
            output: Some(output_path.clone()),
            format: OutputFormat::Svg,
            root: None,
            open: false,
            font_paths: vec![],
        };

        let result = compile(args).await;
        assert!(result.is_ok(), "Compilation failed: {:?}", result);
        assert!(output_path.exists(), "Output file not created");

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("<svg"), "Not a valid SVG");
    }
}
