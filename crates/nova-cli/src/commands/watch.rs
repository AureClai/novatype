//! Watch command implementation.

use crate::commands::compile::{compile, CompileArgs};
use crate::OutputFormat;
use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

/// Arguments for the watch command.
#[derive(Args, Debug)]
pub struct WatchArgs {
    /// Input file to watch.
    #[arg(required = true)]
    pub input: PathBuf,

    /// Output file path.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output format.
    #[arg(short, long, value_enum, default_value = "pdf")]
    pub format: OutputFormat,

    /// Debounce interval in milliseconds.
    #[arg(long, default_value = "300")]
    pub debounce: u64,

    /// Clear terminal before each build.
    #[arg(long)]
    pub clear: bool,

    /// Font paths to include.
    #[arg(long = "font-path")]
    pub font_paths: Vec<PathBuf>,
}

/// Execute the watch command.
///
/// # Errors
///
/// Returns an error if watching fails.
pub async fn watch(args: WatchArgs) -> Result<()> {
    info!("Watching {:?}", args.input);

    // Validate input file exists
    if !args.input.exists() {
        anyhow::bail!("Input file not found: {:?}", args.input);
    }

    println!("Watching {:?} for changes...", args.input);
    println!("Press Ctrl+C to stop.\n");

    // Initial compile
    run_compile(&args).await?;

    // Watch for changes
    let mut last_modified = std::fs::metadata(&args.input)?.modified()?;

    loop {
        tokio::time::sleep(Duration::from_millis(args.debounce)).await;

        // Check if file was modified
        let current_modified = std::fs::metadata(&args.input)?.modified()?;

        if current_modified != last_modified {
            last_modified = current_modified;

            if args.clear {
                // Clear terminal
                print!("\x1B[2J\x1B[1;1H");
            }

            println!(
                "[{}] Change detected, recompiling...",
                chrono_lite_timestamp()
            );

            match run_compile(&args).await {
                Ok(()) => {
                    println!("[{}] Compilation successful.\n", chrono_lite_timestamp());
                }
                Err(e) => {
                    eprintln!("[{}] Compilation failed: {}\n", chrono_lite_timestamp(), e);
                }
            }
        }
    }
}

/// Run a single compilation.
async fn run_compile(args: &WatchArgs) -> Result<()> {
    let compile_args = CompileArgs {
        input: args.input.clone(),
        output: args.output.clone(),
        format: args.format,
        root: None,
        open: false,
        font_paths: args.font_paths.clone(),
    };

    compile(compile_args).await
}

/// Simple timestamp without pulling in chrono.
fn chrono_lite_timestamp() -> String {
    use std::time::SystemTime;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();

    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;

    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_format() {
        let ts = chrono_lite_timestamp();
        assert_eq!(ts.len(), 8); // HH:MM:SS
        assert!(ts.contains(':'));
    }

    #[tokio::test]
    async fn watch_missing_file_fails() {
        let args = WatchArgs {
            input: PathBuf::from("/nonexistent/file.typ"),
            output: None,
            format: OutputFormat::Pdf,
            debounce: 300,
            clear: false,
            font_paths: vec![],
        };

        let result = watch(args).await;
        assert!(result.is_err());
    }
}
