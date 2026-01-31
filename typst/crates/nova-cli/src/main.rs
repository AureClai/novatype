//! NovaType CLI entry point.
//!
//! The `nova` command-line tool for document compilation.

use anyhow::Result;
use clap::Parser;
use nova_cli::{Cli, run};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Run the command
    run(cli).await
}
