//! Compile command implementation.

use crate::diagnostics;
use crate::OutputFormat;
use anyhow::{Context, Result};
use clap::Args;
use nova_font::FontCache;
use nova_schema::FrontmatterParser;
use novatype_core::{
    compile_pdf_rich, compile_svg_rich, load_data_files, set_font_paths, FontConfig, NativeWorld,
    NovaConfig,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

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
    /// If omitted, uses [document].main from nova.toml.
    #[arg()]
    pub input: Option<PathBuf>,

    /// Output file path.
    /// If omitted, uses [output] section from nova.toml or defaults to input stem + format.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output format (overrides nova.toml [output].format).
    #[arg(short, long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Root directory for resolving paths.
    #[arg(long)]
    pub root: Option<PathBuf>,

    /// Open the output file after compilation.
    #[arg(long)]
    pub open: bool,

    /// Additional font paths (merged with nova.toml [fonts].paths).
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
    // Determine root directory (for config loading and path resolution)
    let root = determine_root(&args)?;

    // Load nova.toml configuration (defaults if not found)
    let config = NovaConfig::from_project_or_default(&root).unwrap_or_else(|e| {
        warn!("Could not load nova.toml: {}", e);
        NovaConfig::default()
    });

    // Resolve the input file: CLI arg > config [document].main
    let input = resolve_input(&args, &config)?;

    info!("Compiling {:?}", input);

    // Validate input file exists
    if !input.exists() {
        anyhow::bail!("Input file not found: {:?}", input);
    }

    // Resolve output format: CLI arg > config [output].format > pdf
    let format = args.format.unwrap_or_else(|| {
        config
            .output_format_str()
            .and_then(|s| match s {
                "pdf" => Some(OutputFormat::Pdf),
                "svg" => Some(OutputFormat::Svg),
                "png" => Some(OutputFormat::Png),
                _ => None,
            })
            .unwrap_or(OutputFormat::Pdf)
    });

    // Resolve output path: CLI arg > derived from config [output].directory > input stem
    let output_path = resolve_output(&args, &config, &input, format)?;

    // Read input file
    let content =
        std::fs::read_to_string(&input).with_context(|| format!("Failed to read {:?}", input))?;

    // Build processed content: data injection + config fonts + frontmatter processing
    let mut processed = String::new();

    // 1. Inject data files from [data] section
    if let Some(ref data) = config.data {
        if !data.is_empty() {
            match load_data_files(data, &root) {
                Ok(data_code) => {
                    debug!("Injected {} data variable(s)", data.len());
                    processed.push_str(&data_code);
                }
                Err(e) => {
                    warn!("Failed to load data files: {}", e);
                }
            }
        }
    }

    // 2. Apply font config from nova.toml [fonts] section (frontmatter will override)
    if let Some(ref fonts) = config.fonts {
        if fonts.has_font_mappings() {
            debug!("Applying font configuration from nova.toml");
            processed.push_str(&fonts.to_typst_code());
        }
    }

    // 3. Process frontmatter (strips it, applies font overrides from frontmatter)
    let frontmatter_processed = preprocess_content(&content)?;
    processed.push_str(&frontmatter_processed);

    // Collect font paths: CLI args + config [fonts].paths + nova-font cache
    let mut all_font_paths = args.font_paths.clone();

    // Add font paths from nova.toml [fonts].paths
    if let Some(ref fonts) = config.fonts {
        for path in &fonts.paths {
            if !all_font_paths.contains(path) {
                all_font_paths.push(path.clone());
            }
        }
    }

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

        if let Err(e) = generate_python_figures(&root, &config).await {
            // Don't fail compilation if Python is not configured
            debug!("Python figure generation skipped: {}", e);
        }
    }

    // Create the native world
    debug!("Creating native world with root: {:?}", root);
    let world = NativeWorld::from_source(&processed, &root);

    // Compile based on output format
    info!("Compiling document...");
    match format {
        OutputFormat::Pdf => {
            let result = compile_pdf_rich(&world);
            for w in &result.warnings {
                diagnostics::render_warning(w);
            }
            let pdf = match result.output {
                Ok(pdf) => pdf,
                Err(diags) => return diagnostics::render_errors(&diags),
            };

            // Ensure output directory exists
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create output directory {:?}", parent))?;
            }

            std::fs::write(&output_path, &pdf)
                .with_context(|| format!("Failed to write {:?}", output_path))?;
        }
        OutputFormat::Svg => {
            let result = compile_svg_rich(&world);
            for w in &result.warnings {
                diagnostics::render_warning(w);
            }
            let pages = match result.output {
                Ok(pages) => pages,
                Err(diags) => return diagnostics::render_errors(&diags),
            };

            // Ensure output directory exists
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create output directory {:?}", parent))?;
            }

            if pages.len() == 1 {
                std::fs::write(&output_path, &pages[0])
                    .with_context(|| format!("Failed to write {:?}", output_path))?;
            } else {
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
            // PNG: compile to SVG first (PNG not yet implemented)
            let result = compile_svg_rich(&world);
            for w in &result.warnings {
                diagnostics::render_warning(w);
            }
            let pages = match result.output {
                Ok(pages) => pages,
                Err(diags) => return diagnostics::render_errors(&diags),
            };

            let svg_path = output_path.with_extension("svg");
            if let Some(parent) = svg_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
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

    println!("Compiled: {:?} -> {:?}", input, output_path);

    // Open if requested
    if args.open {
        open_file(&output_path)?;
    }

    Ok(())
}

/// Determine the root directory for the compilation.
fn determine_root(args: &CompileArgs) -> Result<PathBuf> {
    if let Some(ref root) = args.root {
        return Ok(root.clone());
    }

    if let Some(ref input) = args.input {
        return Ok(input
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))));
    }

    // No input specified — use current directory (config will provide input)
    Ok(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// Resolve the input file from CLI args or nova.toml config.
fn resolve_input(args: &CompileArgs, config: &NovaConfig) -> Result<PathBuf> {
    if let Some(ref input) = args.input {
        return Ok(input.clone());
    }

    if let Some(main) = config.main_file() {
        info!("Using main file from nova.toml: {:?}", main);
        return Ok(main);
    }

    anyhow::bail!(
        "No input file specified. Either pass a file argument or set [document].main in nova.toml"
    )
}

/// Resolve the output path from CLI args, config, or defaults.
fn resolve_output(
    args: &CompileArgs,
    config: &NovaConfig,
    input: &Path,
    format: OutputFormat,
) -> Result<PathBuf> {
    if let Some(ref output) = args.output {
        return Ok(output.clone());
    }

    let ext = match format {
        OutputFormat::Pdf => "pdf",
        OutputFormat::Svg => "svg",
        OutputFormat::Png => "png",
    };
    let stem = input.file_stem().unwrap_or_default();

    // If config has an output directory, use it
    if let Some(output_dir) = config.output_directory() {
        let filename = format!("{}.{}", stem.to_string_lossy(), ext);
        return Ok(output_dir.join(filename));
    }

    // Default: same directory as input
    Ok(input.with_file_name(format!("{}.{}", stem.to_string_lossy(), ext)))
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
                        debug!("Applying font configuration from frontmatter: {:?}", fonts);
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
fn provision_pyplot(project_root: &Path) -> Result<()> {
    let nova_dir = project_root.join(".nova");
    let pyplot_path = nova_dir.join("pyplot.typ");

    if !nova_dir.exists() {
        std::fs::create_dir_all(&nova_dir)
            .with_context(|| format!("Failed to create {:?}", nova_dir))?;
    }

    let pyplot_content = include_str!("../../../../typst-packages/nova/lib.typ");
    std::fs::write(&pyplot_path, pyplot_content)
        .with_context(|| format!("Failed to write {:?}", pyplot_path))?;

    debug!("Provisioned pyplot.typ to {:?}", pyplot_path);
    Ok(())
}

/// Generate Python figures if nova.toml configures Python integration.
async fn generate_python_figures(project_root: &Path, config: &NovaConfig) -> Result<()> {
    // Use python config from the already-loaded NovaConfig
    let python_config = match config.python {
        Some(ref py) => py.clone(),
        None => {
            debug!("No [python] section in nova.toml");
            return Ok(());
        }
    };

    // Check if figures directory exists
    if !python_config.figures_dir.exists() {
        debug!(
            "No figures directory found at {:?}",
            python_config.figures_dir
        );
        return Ok(());
    }

    info!("Generating Python figures...");

    let nova = nova_python::NovaPython::new(python_config)?;
    let registry = nova.generate_figures().await?;

    if registry.is_empty() {
        debug!("No Python figures found");
    } else {
        info!("Generated {} Python figure(s)", registry.len());
        for (name, path) in registry.iter() {
            debug!("  {} -> {:?}", name, path);
        }
    }

    // Also provision pyplot even if there's no nova.toml [python],
    // the user might have it in their document
    let _unused = project_root;

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
            input: Some(PathBuf::from("/nonexistent/file.typ")),
            output: None,
            format: None,
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

        std::fs::write(&input_path, "Hello, world!").unwrap();

        let args = CompileArgs {
            input: Some(input_path),
            output: Some(output_path.clone()),
            format: None,
            root: None,
            open: false,
            font_paths: vec![],
            no_python: true,
        };

        let result = compile(args).await;
        assert!(result.is_ok(), "Compilation failed: {:?}", result);
        assert!(output_path.exists(), "Output file not created");

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
    fn font_config_empty() {
        let config = FontConfig::default();
        assert!(!config.has_fonts());
        assert!(config.to_typst_code().is_empty());
    }

    #[tokio::test]
    async fn compile_to_svg() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.typ");
        let output_path = temp_dir.path().join("test.svg");

        std::fs::write(&input_path, "Hello SVG!").unwrap();

        let args = CompileArgs {
            input: Some(input_path),
            output: Some(output_path.clone()),
            format: Some(OutputFormat::Svg),
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

    #[tokio::test]
    async fn compile_uses_nova_toml_defaults() {
        let temp_dir = TempDir::new().unwrap();

        // Create nova.toml with document.main
        std::fs::write(
            temp_dir.path().join("nova.toml"),
            r#"
[document]
main = "main.typ"

[output]
format = "pdf"
directory = "build"
"#,
        )
        .unwrap();

        // Create main.typ
        std::fs::write(temp_dir.path().join("main.typ"), "Hello from config!").unwrap();

        // Create build directory
        std::fs::create_dir_all(temp_dir.path().join("build")).unwrap();

        let args = CompileArgs {
            input: None, // Should use nova.toml [document].main
            output: None,
            format: None,
            root: Some(temp_dir.path().to_path_buf()),
            open: false,
            font_paths: vec![],
            no_python: true,
        };

        let result = compile(args).await;
        assert!(result.is_ok(), "Compilation failed: {:?}", result);

        let output = temp_dir.path().join("build/main.pdf");
        assert!(output.exists(), "Output should be in build/ directory");
    }

    #[test]
    fn resolve_input_from_config() {
        let config = NovaConfig {
            document: Some(novatype_core::config::DocumentConfig {
                main: Some(PathBuf::from("/project/main.typ")),
                template: None,
            }),
            ..Default::default()
        };

        let args = CompileArgs {
            input: None,
            output: None,
            format: None,
            root: None,
            open: false,
            font_paths: vec![],
            no_python: true,
        };

        let result = resolve_input(&args, &config).unwrap();
        assert_eq!(result, PathBuf::from("/project/main.typ"));
    }

    #[test]
    fn resolve_input_cli_overrides_config() {
        let config = NovaConfig {
            document: Some(novatype_core::config::DocumentConfig {
                main: Some(PathBuf::from("/project/main.typ")),
                template: None,
            }),
            ..Default::default()
        };

        let args = CompileArgs {
            input: Some(PathBuf::from("/other/file.typ")),
            output: None,
            format: None,
            root: None,
            open: false,
            font_paths: vec![],
            no_python: true,
        };

        let result = resolve_input(&args, &config).unwrap();
        assert_eq!(result, PathBuf::from("/other/file.typ"));
    }

    #[test]
    fn resolve_input_no_config_no_arg_fails() {
        let config = NovaConfig::default();

        let args = CompileArgs {
            input: None,
            output: None,
            format: None,
            root: None,
            open: false,
            font_paths: vec![],
            no_python: true,
        };

        let result = resolve_input(&args, &config);
        assert!(result.is_err());
    }
}
