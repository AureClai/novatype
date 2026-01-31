//! Compile command implementation.

use crate::OutputFormat;
use anyhow::{Context, Result};
use clap::Args;
use nova_schema::FrontmatterParser;
use std::path::PathBuf;
use std::process::Command;
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
        args.input.with_file_name(format!("{}.{}", stem.to_string_lossy(), ext))
    });

    // Read input file
    let content = std::fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read {:?}", args.input))?;

    // Check for frontmatter and preprocess if needed
    let (_processed_content, temp_file) = preprocess_content(&content, &args.input)?;
    let compile_input = temp_file.as_ref().unwrap_or(&args.input);

    // Find typst binary (same directory as nova binary, or in PATH)
    let typst_path = find_typst_binary()?;
    debug!("Using typst binary: {:?}", typst_path);

    // Build typst compile command
    let mut cmd = Command::new(&typst_path);
    cmd.arg("compile");
    cmd.arg(compile_input);
    cmd.arg(&output_path);

    // Add format flag
    match args.format {
        OutputFormat::Pdf => {} // Default
        OutputFormat::Svg => {
            cmd.arg("--format").arg("svg");
        }
        OutputFormat::Png => {
            cmd.arg("--format").arg("png");
        }
    }

    // Add root directory if specified
    if let Some(root) = &args.root {
        cmd.arg("--root").arg(root);
    }

    // Add font paths
    for font_path in &args.font_paths {
        cmd.arg("--font-path").arg(font_path);
    }

    // Execute compilation
    info!("Running typst compile...");
    let output = cmd.output().with_context(|| {
        format!("Failed to execute typst binary at {:?}", typst_path)
    })?;

    // Clean up temp file if created
    if let Some(temp) = temp_file {
        let _ = std::fs::remove_file(&temp);
    }

    // Check result
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!(
            "Typst compilation failed:\n{}\n{}",
            stderr,
            stdout
        );
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
/// If the content has YAML/TOML frontmatter, strip it and create a temp file.
/// Returns the processed content and optional temp file path.
fn preprocess_content(content: &str, input: &PathBuf) -> Result<(String, Option<PathBuf>)> {
    let parser = FrontmatterParser::new();

    // Try to extract frontmatter
    if let Some((_raw_frontmatter, _format)) = parser.extract_raw(content) {
        debug!("Found frontmatter, preprocessing...");

        // Parse to get remaining content
        let result: Result<(Option<serde_json::Value>, &str), _> = parser.parse(content);

        if let Ok((metadata, remaining)) = result {
            // Log metadata if present
            if let Some(meta) = &metadata {
                if let Some(title) = meta.get("title") {
                    debug!("Document title: {}", title);
                }
            }

            // Create temp file with processed content
            let temp_path = input.with_extension("nova-temp.typ");
            std::fs::write(&temp_path, remaining)
                .with_context(|| "Failed to write temporary file")?;

            return Ok((remaining.to_string(), Some(temp_path)));
        }
    }

    // No frontmatter or parsing failed, use original content
    Ok((content.to_string(), None))
}

/// Find the typst binary.
fn find_typst_binary() -> Result<PathBuf> {
    // First, try same directory as current executable
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let typst_in_dir = exe_dir.join(if cfg!(windows) { "typst.exe" } else { "typst" });
            if typst_in_dir.exists() {
                return Ok(typst_in_dir);
            }
        }
    }

    // Try PATH
    let typst_name = if cfg!(windows) { "typst.exe" } else { "typst" };
    if let Ok(path) = which::which(typst_name) {
        return Ok(path);
    }

    // Fallback: assume it's in PATH
    Ok(PathBuf::from(typst_name))
}

/// Open a file with the system's default application.
fn open_file(path: &PathBuf) -> Result<()> {
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
        // Skip test if typst binary not available
        if find_typst_binary().is_err() {
            eprintln!("Skipping test: typst binary not found");
            return;
        }

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
    }

    #[test]
    fn preprocess_strips_frontmatter() {
        let content = r#"---
title: Test
---
Hello content"#;

        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.typ");
        std::fs::write(&input, content).unwrap();

        let (processed, temp_file) = preprocess_content(content, &input).unwrap();

        assert!(!processed.contains("---"));
        assert!(processed.contains("Hello content"));
        assert!(temp_file.is_some());

        // Clean up
        if let Some(temp) = temp_file {
            let _ = std::fs::remove_file(temp);
        }
    }

    #[test]
    fn preprocess_no_frontmatter() {
        let content = "Just regular content";
        let input = PathBuf::from("test.typ");

        let (processed, temp_file) = preprocess_content(content, &input).unwrap();

        assert_eq!(processed, content);
        assert!(temp_file.is_none());
    }
}
