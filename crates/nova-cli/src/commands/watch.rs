//! Watch command implementation.

use crate::commands::compile::{compile, CompileArgs};
use crate::OutputFormat;
use anyhow::Result;
use clap::Args;
use novatype_core::NovaConfig;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::info;

/// Arguments for the watch command.
#[derive(Args, Debug)]
pub struct WatchArgs {
    /// Input file to watch.
    /// If omitted, uses [document].main from nova.toml.
    #[arg()]
    pub input: Option<PathBuf>,

    /// Output file path.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output format (overrides nova.toml [output].format).
    #[arg(short, long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Debounce interval in milliseconds (overrides nova.toml [watch].debounce).
    #[arg(long)]
    pub debounce: Option<u64>,

    /// Clear terminal before each build (overrides nova.toml [watch].clear).
    #[arg(long)]
    pub clear: bool,

    /// Additional font paths.
    #[arg(long = "font-path")]
    pub font_paths: Vec<PathBuf>,

    /// Skip Python figure generation.
    #[arg(long)]
    pub no_python: bool,
}

/// Execute the watch command.
///
/// # Errors
///
/// Returns an error if watching fails.
pub async fn watch(args: WatchArgs) -> Result<()> {
    // Determine root directory for config loading
    let root = if let Some(ref input) = args.input {
        input
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    // Load nova.toml to get watch settings and input file
    let config = NovaConfig::from_project_or_default(&root).unwrap_or_default();

    // Resolve input file: CLI arg > config [document].main
    let input = if let Some(ref input) = args.input {
        input.clone()
    } else if let Some(main) = config.main_file() {
        info!("Using main file from nova.toml: {:?}", main);
        main
    } else {
        anyhow::bail!(
            "No input file specified. Either pass a file argument or set [document].main in nova.toml"
        );
    };

    if !input.exists() {
        anyhow::bail!("Input file not found: {:?}", input);
    }

    // Merge watch config: CLI args > nova.toml [watch] > defaults
    let debounce = args
        .debounce
        .unwrap_or_else(|| config.watch.as_ref().map(|w| w.debounce).unwrap_or(300));

    let clear = args.clear || config.watch.as_ref().map(|w| w.clear).unwrap_or(false);

    info!("Watching {:?}", input);
    println!("Watching {:?} for changes...", input);
    println!("Press Ctrl+C to stop.\n");

    // Initial compile
    run_compile(&args, &input).await?;

    // Watch for changes
    let mut last_modified = std::fs::metadata(&input)?.modified()?;

    loop {
        tokio::time::sleep(Duration::from_millis(debounce)).await;

        let current_modified = std::fs::metadata(&input)?.modified()?;

        if current_modified != last_modified {
            last_modified = current_modified;

            if clear {
                print!("\x1B[2J\x1B[1;1H");
            }

            println!(
                "[{}] Change detected, recompiling...",
                chrono_lite_timestamp()
            );

            match run_compile(&args, &input).await {
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
async fn run_compile(args: &WatchArgs, input: &Path) -> Result<()> {
    let compile_args = CompileArgs {
        input: Some(input.to_path_buf()),
        output: args.output.clone(),
        format: args.format,
        root: None,
        open: false,
        font_paths: args.font_paths.clone(),
        no_python: args.no_python,
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
            input: Some(PathBuf::from("/nonexistent/file.typ")),
            output: None,
            format: None,
            debounce: None,
            clear: false,
            font_paths: vec![],
            no_python: true,
        };

        let result = watch(args).await;
        assert!(result.is_err());
    }
}
