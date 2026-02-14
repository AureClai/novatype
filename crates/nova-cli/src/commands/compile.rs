//! Compile command implementation.

use crate::OutputFormat;
use anyhow::{Context, Result};
use clap::Args;
use nova_font::FontCache;
use nova_python::PythonConfig;
use nova_schema::FrontmatterParser;
use novatype_core::{compile_pdf, compile_svg, set_font_paths, NativeWorld};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Single font specification - can be a simple string or full spec.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum FontSpec {
    /// Simple font family name.
    Simple(String),
    /// Full font specification with size, weight, etc.
    Full {
        family: String,
        #[serde(default)]
        size: Option<String>,
        #[serde(default)]
        weight: Option<String>,
        #[serde(default)]
        style: Option<String>,
    },
}

impl FontSpec {
    /// Get the font family name.
    #[allow(dead_code)]
    pub fn family(&self) -> &str {
        match self {
            FontSpec::Simple(f) => f,
            FontSpec::Full { family, .. } => family,
        }
    }

    /// Generate Typst arguments for this font spec.
    pub fn to_typst_args(&self) -> String {
        match self {
            FontSpec::Simple(family) => format!("font: \"{}\"", family),
            FontSpec::Full {
                family,
                size,
                weight,
                style,
            } => {
                let mut args = vec![format!("font: \"{}\"", family)];
                if let Some(s) = size {
                    args.push(format!("size: {}", s));
                }
                if let Some(w) = weight {
                    args.push(format!("weight: \"{}\"", w));
                }
                if let Some(st) = style {
                    args.push(format!("style: \"{}\"", st));
                }
                args.join(", ")
            }
        }
    }
}

/// Font configuration from frontmatter.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FontConfig {
    /// Main body font.
    pub main: Option<FontSpec>,
    /// Font for headings.
    pub heading: Option<FontSpec>,
    /// Monospace font for code.
    pub mono: Option<FontSpec>,
    /// Font for math expressions.
    pub math: Option<FontSpec>,
}

impl FontConfig {
    /// Check if any font is configured.
    pub fn has_fonts(&self) -> bool {
        self.main.is_some() || self.heading.is_some() || self.mono.is_some() || self.math.is_some()
    }

    /// Generate Typst code to set fonts.
    pub fn to_typst_code(&self) -> String {
        let mut code = String::new();

        if let Some(ref main) = self.main {
            code.push_str(&format!("#set text({})\n", main.to_typst_args()));
        }

        if let Some(ref mono) = self.mono {
            code.push_str(&format!("#show raw: set text({})\n", mono.to_typst_args()));
        }

        if let Some(ref heading) = self.heading {
            code.push_str(&format!(
                "#show heading: set text({})\n",
                heading.to_typst_args()
            ));
        }

        // Math font
        if let Some(ref math) = self.math {
            code.push_str(&format!(
                "#show math.equation: set text({})\n",
                math.to_typst_args()
            ));
        }

        if !code.is_empty() {
            code.push('\n');
        }

        code
    }
}

/// Document metadata from frontmatter (partial, for font extraction).
#[derive(Debug, Clone, Default, Deserialize)]
struct DocumentMetadata {
    #[serde(default)]
    fonts: Option<FontConfig>,
}

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

    /// Skip Python figure generation.
    #[arg(long)]
    pub no_python: bool,
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
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    });

    // Collect font paths: user-provided + nova-font cache
    let mut all_font_paths = args.font_paths.clone();

    // Add fonts from nova-font cache
    match FontCache::new() {
        Ok(cache) => {
            let cached_paths = cache.typst_font_paths();
            if !cached_paths.is_empty() {
                debug!(
                    "Adding {} font paths from nova-font cache",
                    cached_paths.len()
                );
                all_font_paths.extend(cached_paths);
            }
        }
        Err(e) => {
            warn!("Could not load nova-font cache: {}", e);
        }
    }

    // Set font paths if any are available
    if !all_font_paths.is_empty() {
        debug!("Setting font paths: {:?}", all_font_paths);
        set_font_paths(all_font_paths);
    }
    // Generate Python figures if configured
    if !args.no_python {
        // Provision pyplot.typ helper
        if let Err(e) = provision_pyplot(&root) {
            warn!("Failed to provision pyplot.typ: {}", e);
        }

        if let Err(e) = generate_python_figures(&root).await {
            // Don't fail compilation if Python is not configured
            debug!("Python figure generation skipped: {}", e);
        }
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
/// If the content has YAML/TOML frontmatter, strip it and inject font settings.
fn preprocess_content(content: &str) -> Result<String> {
    let parser = FrontmatterParser::new();

    // Try to extract frontmatter
    if let Some((_raw_frontmatter, _format)) = parser.extract_raw(content) {
        debug!("Found frontmatter, preprocessing...");

        // Parse to get metadata and remaining content
        let result: std::result::Result<(Option<DocumentMetadata>, &str), _> =
            parser.parse(content);

        if let Ok((metadata, remaining)) = result {
            let mut output = String::new();

            // Extract and apply font configuration
            if let Some(ref meta) = metadata {
                if let Some(ref fonts) = meta.fonts {
                    if fonts.has_fonts() {
                        debug!("Applying font configuration: {:?}", fonts);
                        output.push_str(&fonts.to_typst_code());
                    }
                }
            }

            output.push_str(remaining);
            return Ok(output);
        }

        // Fallback: parse as generic JSON value if typed parsing fails
        let result: std::result::Result<(Option<serde_json::Value>, &str), _> =
            parser.parse(content);

        if let Ok((metadata, remaining)) = result {
            let mut output = String::new();

            // Try to extract fonts from JSON value
            if let Some(ref meta) = metadata {
                if let Some(title) = meta.get("title") {
                    debug!("Document title: {}", title);
                }

                if let Some(fonts_value) = meta.get("fonts") {
                    if let Ok(fonts) = serde_json::from_value::<FontConfig>(fonts_value.clone()) {
                        if fonts.has_fonts() {
                            debug!("Applying font configuration: {:?}", fonts);
                            output.push_str(&fonts.to_typst_code());
                        }
                    }
                }
            }

            output.push_str(remaining);
            return Ok(output);
        }
    }

    // No frontmatter or parsing failed, use original content
    Ok(content.to_string())
}

/// Provision the pyplot.typ helper to .nova/ directory.
///
/// This ensures the pyplot function is available for importing Python figures.
fn provision_pyplot(project_root: &Path) -> Result<()> {
    let nova_dir = project_root.join(".nova");
    let pyplot_path = nova_dir.join("pyplot.typ");

    // Create .nova directory if it doesn't exist
    if !nova_dir.exists() {
        std::fs::create_dir_all(&nova_dir)
            .with_context(|| format!("Failed to create {:?}", nova_dir))?;
    }

    // Write pyplot.typ (embedded content - single source of truth)
    let pyplot_content = include_str!("../../../../typst-packages/nova/lib.typ");
    std::fs::write(&pyplot_path, pyplot_content)
        .with_context(|| format!("Failed to write {:?}", pyplot_path))?;

    debug!("Provisioned pyplot.typ to {:?}", pyplot_path);
    Ok(())
}

/// Generate Python figures if nova.toml configures Python integration.
async fn generate_python_figures(project_root: &Path) -> Result<()> {
    // Check if nova.toml exists and has Python configuration
    let config = match PythonConfig::from_project(project_root) {
        Ok(config) => config,
        Err(nova_python::Error::ConfigNotFound { .. }) => {
            // No nova.toml, Python not configured
            return Ok(());
        }
        Err(e) => {
            warn!("Failed to load Python config: {}", e);
            return Ok(());
        }
    };

    // Check if figures directory exists
    if !config.figures_dir.exists() {
        debug!("No figures directory found at {:?}", config.figures_dir);
        return Ok(());
    }

    info!("Generating Python figures...");

    // Create Nova Python instance and generate figures
    let nova = nova_python::NovaPython::new(config)?;
    let registry = nova.generate_figures().await?;

    if registry.is_empty() {
        debug!("No Python figures found");
    } else {
        info!("Generated {} Python figure(s)", registry.len());
        for (name, path) in registry.iter() {
            debug!("  {} -> {:?}", name, path);
        }
    }

    Ok(())
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
            no_python: true,
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
            no_python: true,
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

    #[test]
    fn preprocess_with_fonts_simple() {
        let content = r#"---
title: Test
fonts:
  main: "Inter"
  mono: "JetBrains Mono"
---
Hello content"#;

        let processed = preprocess_content(content).unwrap();

        assert!(!processed.contains("---"));
        assert!(processed.contains("#set text(font: \"Inter\")"));
        assert!(processed.contains("#show raw: set text(font: \"JetBrains Mono\")"));
        assert!(processed.contains("Hello content"));
    }

    #[test]
    fn preprocess_with_fonts_full() {
        let content = r#"---
title: Test
fonts:
  main:
    family: "Inter"
    size: "11pt"
  heading:
    family: "Open Sans"
    size: "14pt"
    weight: "bold"
---
Hello content"#;

        let processed = preprocess_content(content).unwrap();

        assert!(!processed.contains("---"));
        assert!(processed.contains("#set text(font: \"Inter\", size: 11pt)"));
        assert!(processed.contains(
            "#show heading: set text(font: \"Open Sans\", size: 14pt, weight: \"bold\")"
        ));
        assert!(processed.contains("Hello content"));
    }

    #[test]
    fn font_config_simple_to_typst() {
        let config = FontConfig {
            main: Some(FontSpec::Simple("Inter".to_string())),
            heading: Some(FontSpec::Simple("Open Sans".to_string())),
            mono: Some(FontSpec::Simple("Fira Code".to_string())),
            math: None,
        };

        let code = config.to_typst_code();

        assert!(code.contains("#set text(font: \"Inter\")"));
        assert!(code.contains("#show heading: set text(font: \"Open Sans\")"));
        assert!(code.contains("#show raw: set text(font: \"Fira Code\")"));
    }

    #[test]
    fn font_config_full_to_typst() {
        let config = FontConfig {
            main: Some(FontSpec::Full {
                family: "Inter".to_string(),
                size: Some("11pt".to_string()),
                weight: None,
                style: None,
            }),
            heading: Some(FontSpec::Full {
                family: "Open Sans".to_string(),
                size: Some("14pt".to_string()),
                weight: Some("bold".to_string()),
                style: None,
            }),
            mono: None,
            math: None,
        };

        let code = config.to_typst_code();

        assert!(code.contains("#set text(font: \"Inter\", size: 11pt)"));
        assert!(code.contains(
            "#show heading: set text(font: \"Open Sans\", size: 14pt, weight: \"bold\")"
        ));
    }

    #[test]
    fn font_config_empty() {
        let config = FontConfig::default();
        assert!(!config.has_fonts());
        assert!(config.to_typst_code().is_empty());
    }

    #[test]
    fn font_spec_family() {
        let simple = FontSpec::Simple("Inter".to_string());
        assert_eq!(simple.family(), "Inter");

        let full = FontSpec::Full {
            family: "Open Sans".to_string(),
            size: Some("12pt".to_string()),
            weight: None,
            style: None,
        };
        assert_eq!(full.family(), "Open Sans");
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
            no_python: true,
        };

        let result = compile(args).await;
        assert!(result.is_ok(), "Compilation failed: {:?}", result);
        assert!(output_path.exists(), "Output file not created");

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("<svg"), "Not a valid SVG");
    }
}
