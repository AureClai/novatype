//! # Nova WASM
//!
//! WebAssembly bindings for NovaType, enabling browser-based document compilation.
//!
//! ## Usage in JavaScript
//!
//! ```javascript
//! import init, { NovaCompiler } from 'nova-wasm';
//!
//! await init();
//!
//! const compiler = new NovaCompiler();
//! const result = compiler.compile(source);
//! console.log(result.svg_pages); // Array of SVG strings
//! ```

mod world;

use wasm_bindgen::prelude::*;
use world::WasmWorld;

/// Initialize the WASM module.
///
/// Call this once before using any other functions.
#[wasm_bindgen(start)]
pub fn init_module() {
    // Set up panic hook for better error messages in console
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// NovaType compiler for WebAssembly.
#[wasm_bindgen]
pub struct NovaCompiler {
    /// Configuration options
    strict_validation: bool,
}

#[wasm_bindgen]
impl NovaCompiler {
    /// Create a new compiler instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            strict_validation: true,
        }
    }

    /// Set strict validation mode.
    #[wasm_bindgen(js_name = setStrictValidation)]
    pub fn set_strict_validation(&mut self, strict: bool) {
        self.strict_validation = strict;
    }

    /// Compile a document from source.
    ///
    /// Returns SVG output for display in the browser.
    #[wasm_bindgen]
    pub fn compile(&self, source: &str) -> Result<JsValue, JsValue> {
        // Parse frontmatter
        let parser = nova_schema::FrontmatterParser::new();
        let (metadata, content): (Option<serde_json::Value>, &str) = parser
            .parse(source)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {e}")))?;

        // Compile with Typst
        let svg_pages = match compile_to_svg(content) {
            Ok(pages) => pages,
            Err(errors) => {
                let result = CompileResult {
                    success: false,
                    content: content.to_string(),
                    metadata,
                    svg_pages: vec![],
                    errors,
                };
                return serde_wasm_bindgen::to_value(&result)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")));
            }
        };

        // Build result object
        let result = CompileResult {
            success: true,
            content: content.to_string(),
            metadata,
            svg_pages,
            errors: vec![],
        };

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
    }

    /// Compile a document and return only the first page SVG.
    #[wasm_bindgen(js_name = compileToSvg)]
    pub fn compile_to_svg_simple(&self, source: &str) -> Result<String, JsValue> {
        // Parse frontmatter
        let parser = nova_schema::FrontmatterParser::new();
        let (_, content): (Option<serde_json::Value>, &str) = parser
            .parse(source)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {e}")))?;

        // Compile with Typst
        let svg_pages =
            compile_to_svg(content).map_err(|errors| JsValue::from_str(&errors.join("\n")))?;

        svg_pages
            .into_iter()
            .next()
            .ok_or_else(|| JsValue::from_str("No pages generated"))
    }

    /// Validate document metadata.
    ///
    /// Note: Full JSON Schema validation is not available in the WASM build.
    /// This function only checks for frontmatter presence.
    #[wasm_bindgen]
    pub fn validate(&self, source: &str) -> Result<JsValue, JsValue> {
        let parser = nova_schema::FrontmatterParser::new();
        let (metadata, _): (Option<serde_json::Value>, _) = parser
            .parse(source)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {e}")))?;

        let errors: Vec<String> = if metadata.is_some() {
            // Basic validation - just check if we have a title
            if let Some(ref meta) = metadata {
                if meta.get("title").is_none() {
                    vec!["Missing required field: title".to_string()]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else if self.strict_validation {
            vec!["No frontmatter found".to_string()]
        } else {
            vec![]
        };

        let validation_result = ValidationResult {
            valid: errors.is_empty(),
            errors,
        };

        serde_wasm_bindgen::to_value(&validation_result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
    }

    /// Parse frontmatter from source.
    #[wasm_bindgen(js_name = parseFrontmatter)]
    pub fn parse_frontmatter(&self, source: &str) -> Result<JsValue, JsValue> {
        let parser = nova_schema::FrontmatterParser::new();
        let (metadata, content): (Option<serde_json::Value>, _) = parser
            .parse(source)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {e}")))?;

        let result = ParseResult {
            metadata,
            content: content.to_string(),
        };

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
    }
}

impl Default for NovaCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Compile Typst source to SVG pages.
fn compile_to_svg(source: &str) -> Result<Vec<String>, Vec<String>> {
    // Create the WASM world
    let world = WasmWorld::new(source);

    // Compile the document
    let result = typst::compile::<typst::layout::PagedDocument>(&world);

    // Handle warnings (log to console)
    for warning in &result.warnings {
        log(&format!("Warning: {}", warning.message));
    }

    // Handle result
    match result.output {
        Ok(document) => {
            // Render each page to SVG
            let svg_pages: Vec<String> = document.pages.iter().map(typst_svg::svg).collect();

            Ok(svg_pages)
        }
        Err(errors) => {
            let error_messages: Vec<String> = errors
                .iter()
                .map(|e| {
                    format!(
                        "{}: {}",
                        e.span
                            .id()
                            .map(|id| format!("{:?}", id))
                            .unwrap_or_default(),
                        e.message
                    )
                })
                .collect();
            Err(error_messages)
        }
    }
}

/// Compilation result structure.
#[derive(serde::Serialize)]
struct CompileResult {
    success: bool,
    content: String,
    metadata: Option<serde_json::Value>,
    svg_pages: Vec<String>,
    errors: Vec<String>,
}

/// Validation result structure.
#[derive(serde::Serialize)]
struct ValidationResult {
    valid: bool,
    errors: Vec<String>,
}

/// Parse result structure.
#[derive(serde::Serialize)]
struct ParseResult {
    metadata: Option<serde_json::Value>,
    content: String,
}

/// Log a message to the browser console.
#[wasm_bindgen]
pub fn log(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}

/// Get the library version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn compiler_creation() {
        let compiler = NovaCompiler::new();
        assert!(compiler.strict_validation);
    }

    #[wasm_bindgen_test]
    fn parse_frontmatter_valid() {
        let compiler = NovaCompiler::new();
        let source = r#"---
title: Test
---
Content"#;

        let result = compiler.parse_frontmatter(source);
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn version_returns_string() {
        let v = version();
        assert!(!v.is_empty());
    }

    #[wasm_bindgen_test]
    fn compile_simple_document() {
        let compiler = NovaCompiler::new();
        let source = "Hello, World!";
        let result = compiler.compile(source);
        assert!(result.is_ok());
    }
}
