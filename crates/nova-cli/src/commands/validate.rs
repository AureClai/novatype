//! Validate command implementation.

use anyhow::{Context, Result};
use clap::Args;
use nova_schema::{article_schema, FrontmatterParser, Schema};
use std::path::PathBuf;
use tracing::info;

/// Arguments for the validate command.
#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Input file(s) to validate.
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,

    /// Schema to validate against.
    #[arg(short, long)]
    pub schema: Option<PathBuf>,

    /// Strict validation mode.
    #[arg(long)]
    pub strict: bool,
}

/// Execute the validate command.
///
/// # Errors
///
/// Returns an error if validation fails.
pub async fn validate(args: ValidateArgs) -> Result<()> {
    let mut all_valid = true;
    let mut total_errors = 0;

    for input in &args.inputs {
        info!("Validating {:?}", input);

        match validate_file(input, args.schema.as_ref(), args.strict) {
            Ok(errors) if errors.is_empty() => {
                println!("✓ {:?} - valid", input);
            }
            Ok(errors) => {
                println!("✗ {:?} - {} error(s)", input, errors.len());
                for error in &errors {
                    println!("  - {}", error);
                }
                all_valid = false;
                total_errors += errors.len();
            }
            Err(e) => {
                println!("✗ {:?} - error: {}", input, e);
                all_valid = false;
                total_errors += 1;
            }
        }
    }

    if all_valid {
        println!("\nAll {} file(s) valid.", args.inputs.len());
        Ok(())
    } else {
        println!("\nValidation failed with {} error(s).", total_errors);
        anyhow::bail!("Validation failed");
    }
}

/// Validate a single file.
fn validate_file(
    path: &PathBuf,
    schema_path: Option<&PathBuf>,
    strict: bool,
) -> Result<Vec<String>> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;

    let mut errors = Vec::new();

    // Parse frontmatter
    let parser = FrontmatterParser::new().strict(strict);
    let (metadata, _remaining): (Option<serde_json::Value>, _) = parser
        .parse(&content)
        .map_err(|e| anyhow::anyhow!("Frontmatter parse error: {}", e))?;

    // If no metadata, nothing to validate
    let Some(metadata) = metadata else {
        if strict {
            errors.push("No frontmatter found".to_string());
        }
        return Ok(errors);
    };

    // Load schema
    let schema = if let Some(schema_path) = schema_path {
        Schema::from_file(schema_path)
            .map_err(|e| anyhow::anyhow!("Failed to load schema: {}", e))?
    } else {
        // Use default article schema
        Schema::new("article", article_schema())
            .map_err(|e| anyhow::anyhow!("Failed to create schema: {}", e))?
    };

    // Validate
    let result = schema.validate(&metadata);
    for error in result.errors {
        errors.push(error.to_string());
    }

    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn validate_valid_document() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("valid.typ");

        let content = r#"---
title: "Test Document"
authors:
  - name: Test Author
---
Content here
"#;
        std::fs::write(&file_path, content).unwrap();

        let args = ValidateArgs {
            inputs: vec![file_path],
            schema: None,
            strict: false,
        };

        let result = validate(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_missing_title() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.typ");

        let content = r#"---
authors:
  - name: Test Author
---
Content here
"#;
        std::fs::write(&file_path, content).unwrap();

        let args = ValidateArgs {
            inputs: vec![file_path],
            schema: None,
            strict: false,
        };

        let result = validate(args).await;
        assert!(result.is_err());
    }

    #[test]
    fn validate_file_no_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("no-frontmatter.typ");

        std::fs::write(&file_path, "Just content, no frontmatter").unwrap();

        // Non-strict mode: no errors
        let errors = validate_file(&file_path, None, false).unwrap();
        assert!(errors.is_empty());

        // Strict mode: error
        let errors = validate_file(&file_path, None, true).unwrap();
        assert!(!errors.is_empty());
    }
}
