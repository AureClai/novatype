//! # Nova CLI
//!
//! Command-line interface for NovaType document compilation.
//!
//! ## Usage
//!
//! ```bash
//! # Compile a document
//! nova compile document.typ
//!
//! # Compile with watch mode
//! nova compile document.typ --watch
//!
//! # Initialize a new project
//! nova init my-paper --template ieee-article
//!
//! # Validate document metadata
//! nova validate document.typ
//! ```

pub mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// NovaType document composition system.
#[derive(Parser, Debug)]
#[command(name = "nova")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors.
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

/// Available commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Compile a document to PDF or other formats.
    Compile(commands::CompileArgs),

    /// Initialize a new NovaType project.
    Init(commands::InitArgs),

    /// Validate document metadata against schema.
    Validate(commands::ValidateArgs),

    /// Watch for changes and recompile.
    Watch(commands::WatchArgs),

    /// Manage templates.
    Template(commands::TemplateArgs),
}

/// Run the CLI with the given arguments.
///
/// # Errors
///
/// Returns an error if the command fails.
pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Compile(args) => commands::compile(args).await,
        Commands::Init(args) => commands::init(args).await,
        Commands::Validate(args) => commands::validate(args).await,
        Commands::Watch(args) => commands::watch(args).await,
        Commands::Template(args) => commands::template(args).await,
    }
}

/// Common output format options.
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum OutputFormat {
    /// PDF document.
    #[default]
    Pdf,
    /// SVG pages.
    Svg,
    /// PNG images.
    Png,
}

impl From<OutputFormat> for nova_core::compiler::OutputFormat {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Pdf => Self::Pdf,
            OutputFormat::Svg => Self::Svg,
            OutputFormat::Png => Self::Png,
        }
    }
}
