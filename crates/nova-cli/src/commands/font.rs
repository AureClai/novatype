//! Font management commands.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use nova_font::{FontCategory, FontManager};
use tracing::info;

/// Font management arguments.
#[derive(Args, Debug)]
pub struct FontArgs {
    /// Font subcommand.
    #[command(subcommand)]
    pub command: FontCommand,
}

/// Font subcommands.
#[derive(Subcommand, Debug)]
pub enum FontCommand {
    /// Search for fonts on Google Fonts.
    Search(SearchArgs),

    /// Install a font from Google Fonts.
    Install(InstallArgs),

    /// Uninstall a font.
    Uninstall(UninstallArgs),

    /// List installed fonts.
    List,

    /// Get information about a font.
    Info(InfoArgs),

    /// Install a predefined font bundle.
    Bundle(BundleArgs),

    /// Show cache information.
    Cache(CacheArgs),
}

/// Search command arguments.
#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search query (font name).
    pub query: String,

    /// Filter by category.
    #[arg(short, long)]
    pub category: Option<String>,

    /// Maximum number of results.
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
}

/// Install command arguments.
#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Font family name.
    pub name: String,

    /// Specific variants to install (e.g., "regular", "700", "700italic").
    #[arg(short, long)]
    pub variants: Option<Vec<String>>,
}

/// Uninstall command arguments.
#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// Font family name.
    pub name: String,
}

/// Info command arguments.
#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Font family name.
    pub name: String,
}

/// Bundle command arguments.
#[derive(Args, Debug)]
pub struct BundleArgs {
    /// Bundle name.
    #[command(subcommand)]
    pub bundle: BundleName,
}

/// Available font bundles.
#[derive(Subcommand, Debug, Clone)]
pub enum BundleName {
    /// Academic fonts (EB Garamond, Libre Baskerville, etc.).
    Academic,
    /// Modern fonts (Inter, Roboto, Open Sans, Lato).
    Modern,
    /// Monospace fonts (JetBrains Mono, Fira Code, etc.).
    Monospace,
    /// Classic fonts (Playfair Display, Merriweather, etc.).
    Classic,
    /// Minimal bundle (Inter, Source Serif Pro, JetBrains Mono).
    Minimal,
    /// List available bundles.
    List,
}

/// Cache command arguments.
#[derive(Args, Debug)]
pub struct CacheArgs {
    /// Cache subcommand.
    #[command(subcommand)]
    pub command: CacheCommand,
}

/// Cache subcommands.
#[derive(Subcommand, Debug)]
pub enum CacheCommand {
    /// Show cache location and size.
    Info,
    /// Clear the font cache.
    Clear,
}

/// Execute font command.
pub async fn font(args: FontArgs) -> Result<()> {
    match args.command {
        FontCommand::Search(search_args) => search(search_args).await,
        FontCommand::Install(install_args) => install(install_args).await,
        FontCommand::Uninstall(uninstall_args) => uninstall(uninstall_args).await,
        FontCommand::List => list().await,
        FontCommand::Info(info_args) => font_info(info_args).await,
        FontCommand::Bundle(bundle_args) => bundle(bundle_args).await,
        FontCommand::Cache(cache_args) => cache(cache_args).await,
    }
}

/// Search for fonts.
async fn search(args: SearchArgs) -> Result<()> {
    let manager = FontManager::new().context("Failed to initialize font manager")?;

    let results = if let Some(category_str) = &args.category {
        let category = parse_category(category_str)?;
        manager.search_by_category(category).await?
    } else {
        manager.search(&args.query).await?
    };

    if results.is_empty() {
        println!("No fonts found matching '{}'", args.query);
        return Ok(());
    }

    println!("Found {} fonts:\n", results.len().min(args.limit));

    for result in results.iter().take(args.limit) {
        let installed = if result.installed { " [installed]" } else { "" };
        let category = result
            .family
            .category
            .as_deref()
            .unwrap_or("unknown");

        println!(
            "  {} ({}){}",
            result.family.family, category, installed
        );

        // Show variants
        let variants: Vec<&str> = result.family.variants.iter().take(5).map(String::as_str).collect();
        let more = if result.family.variants.len() > 5 {
            format!(" +{} more", result.family.variants.len() - 5)
        } else {
            String::new()
        };
        println!("    Variants: {}{}", variants.join(", "), more);
    }

    if results.len() > args.limit {
        println!("\n  ... and {} more", results.len() - args.limit);
    }

    Ok(())
}

/// Install a font.
async fn install(args: InstallArgs) -> Result<()> {
    let mut manager = FontManager::new().context("Failed to initialize font manager")?;

    println!("Installing font: {}", args.name);

    let paths = if let Some(variants) = &args.variants {
        let variant_refs: Vec<&str> = variants.iter().map(String::as_str).collect();
        manager.install_variants(&args.name, &variant_refs).await?
    } else {
        manager.install(&args.name).await?
    };

    println!("Installed {} font files:", paths.len());
    for path in &paths {
        println!("  {}", path.display());
    }

    info!(font = %args.name, files = paths.len(), "Font installed");
    Ok(())
}

/// Uninstall a font.
async fn uninstall(args: UninstallArgs) -> Result<()> {
    let mut manager = FontManager::new().context("Failed to initialize font manager")?;

    manager.uninstall(&args.name)?;
    println!("Uninstalled font: {}", args.name);

    info!(font = %args.name, "Font uninstalled");
    Ok(())
}

/// List installed fonts.
async fn list() -> Result<()> {
    let manager = FontManager::new().context("Failed to initialize font manager")?;

    let installed = manager.list_installed();

    if installed.is_empty() {
        println!("No fonts installed.");
        println!("\nUse 'nova font search <query>' to find fonts.");
        println!("Use 'nova font install <name>' to install a font.");
        return Ok(());
    }

    println!("Installed fonts ({}):\n", installed.len());

    for font in installed {
        let category = font.category.as_deref().unwrap_or("unknown");
        let variants: Vec<_> = font.files.iter().map(|f| f.variant.as_str()).collect();

        println!("  {} ({})", font.family, category);
        println!("    Variants: {}", variants.join(", "));
    }

    // Show font paths for Typst
    println!("\nFont directories for Typst:");
    for path in manager.typst_font_paths() {
        println!("  {}", path.display());
    }

    Ok(())
}

/// Get font information.
async fn font_info(args: InfoArgs) -> Result<()> {
    let manager = FontManager::new().context("Failed to initialize font manager")?;

    let info = manager.get_font_info(&args.name).await?;

    println!("Font: {}", info.family.family);
    println!();

    if let Some(category) = &info.family.category {
        println!("  Category: {}", category);
    }

    if let Some(version) = &info.family.version {
        println!("  Version: {}", version);
    }

    if let Some(modified) = &info.family.last_modified {
        println!("  Last modified: {}", modified);
    }

    println!("  Subsets: {}", info.family.subsets.join(", "));
    println!("  Variants: {}", info.family.variants.join(", "));

    println!();
    println!("  Status: {}", if info.installed { "Installed" } else { "Not installed" });

    if let Some(files) = &info.local_files {
        println!();
        println!("  Local files:");
        for file in files {
            println!("    {}", file.display());
        }
    }

    Ok(())
}

/// Install a font bundle.
async fn bundle(args: BundleArgs) -> Result<()> {
    use nova_font::manager::bundles;

    match args.bundle {
        BundleName::List => {
            println!("Available font bundles:\n");
            println!("  academic   - Academic fonts: {}", bundles::ACADEMIC.join(", "));
            println!("  modern     - Modern fonts: {}", bundles::MODERN.join(", "));
            println!("  monospace  - Monospace fonts: {}", bundles::MONOSPACE.join(", "));
            println!("  classic    - Classic fonts: {}", bundles::CLASSIC.join(", "));
            println!("  minimal    - Minimal bundle: {}", bundles::MINIMAL.join(", "));
            println!("\nUse 'nova font bundle <name>' to install a bundle.");
            Ok(())
        }
        bundle_name => {
            let fonts = match bundle_name {
                BundleName::Academic => bundles::ACADEMIC,
                BundleName::Modern => bundles::MODERN,
                BundleName::Monospace => bundles::MONOSPACE,
                BundleName::Classic => bundles::CLASSIC,
                BundleName::Minimal => bundles::MINIMAL,
                BundleName::List => unreachable!(),
            };

            let bundle_display = format!("{:?}", bundle_name).to_lowercase();
            println!("Installing {} bundle ({} fonts)...\n", bundle_display, fonts.len());

            let mut manager = FontManager::new().context("Failed to initialize font manager")?;
            let mut total_files = 0;

            for font_name in fonts {
                print!("  Installing {}... ", font_name);

                match manager.install(font_name).await {
                    Ok(paths) => {
                        println!("OK ({} files)", paths.len());
                        total_files += paths.len();
                    }
                    Err(nova_font::Error::AlreadyInstalled { .. }) => {
                        println!("already installed");
                    }
                    Err(e) => {
                        println!("FAILED: {}", e);
                    }
                }
            }

            println!("\nBundle installation complete ({} new files)", total_files);
            Ok(())
        }
    }
}

/// Cache management.
async fn cache(args: CacheArgs) -> Result<()> {
    let mut manager = FontManager::new().context("Failed to initialize font manager")?;

    match args.command {
        CacheCommand::Info => {
            let size = manager.cache_size()?;
            let size_mb = size as f64 / (1024.0 * 1024.0);

            println!("Font cache information:\n");
            println!("  Location: {}", manager.cache_dir().display());
            println!("  Size: {:.2} MB", size_mb);
            println!("  Fonts: {}", manager.list_installed().len());
        }
        CacheCommand::Clear => {
            manager.clear_cache()?;
            println!("Font cache cleared.");
        }
    }

    Ok(())
}

/// Parse a category string.
fn parse_category(s: &str) -> Result<FontCategory> {
    FontCategory::from_str(s).ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid category '{}'. Valid categories: serif, sans-serif, display, handwriting, monospace",
            s
        )
    })
}
