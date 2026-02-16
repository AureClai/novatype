//! Unified project configuration loaded from `nova.toml`.
//!
//! This module provides the central [`NovaConfig`] struct that all commands
//! use as their source of truth for project settings.
//!
//! ## Precedence
//!
//! CLI arguments > nova.toml > defaults

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ─── Top-level config ────────────────────────────────────────────────

/// Complete project configuration loaded from `nova.toml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct NovaConfig {
    /// `[project]` — project metadata.
    pub project: Option<ProjectConfig>,
    /// `[document]` — main document settings.
    pub document: Option<DocumentConfig>,
    /// `[output]` — build output settings.
    pub output: Option<OutputConfig>,
    /// `[fonts]` — font paths and font mappings.
    pub fonts: Option<FontsConfig>,
    /// `[bibliography]` — citation style and bibliography files.
    pub bibliography: Option<BibliographyConfig>,
    /// `[python]` — Python figure integration.
    pub python: Option<PythonConfig>,
    /// `[watch]` — watch mode settings.
    pub watch: Option<WatchConfig>,
    /// `[data]` — external data files injected as Typst variables.
    pub data: Option<HashMap<String, PathBuf>>,
}

impl NovaConfig {
    /// Load configuration from a project directory.
    ///
    /// Reads `nova.toml` in the given directory and resolves all relative paths.
    /// Returns an error if the file exists but cannot be parsed.
    pub fn from_project(project_dir: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let project_dir = project_dir.as_ref();
        let config_path = project_dir.join("nova.toml");

        if !config_path.exists() {
            return Err(ConfigError::NotFound { path: config_path });
        }

        let content = std::fs::read_to_string(&config_path).map_err(|e| ConfigError::Io {
            path: config_path.clone(),
            source: e,
        })?;

        let mut config: NovaConfig = toml::from_str(&content).map_err(|e| ConfigError::Parse {
            path: config_path,
            message: e.to_string(),
        })?;

        config.resolve_paths(project_dir);
        Ok(config)
    }

    /// Load configuration, returning defaults if `nova.toml` is missing.
    ///
    /// Only returns an error if the file exists but cannot be parsed.
    pub fn from_project_or_default(project_dir: impl AsRef<Path>) -> Result<Self, ConfigError> {
        match Self::from_project(&project_dir) {
            Ok(config) => Ok(config),
            Err(ConfigError::NotFound { .. }) => Ok(Self::default()),
            Err(e) => Err(e),
        }
    }

    /// Get the main document file path (from `[document].main`).
    pub fn main_file(&self) -> Option<PathBuf> {
        self.document.as_ref().and_then(|d| d.main.clone())
    }

    /// Get the output format string (from `[output].format`).
    pub fn output_format_str(&self) -> Option<&str> {
        self.output.as_ref().map(|o| o.format.as_str())
    }

    /// Get the output directory (from `[output].directory`).
    pub fn output_directory(&self) -> Option<&Path> {
        self.output.as_ref().map(|o| o.directory.as_path())
    }

    /// Resolve all relative paths against the project root.
    fn resolve_paths(&mut self, project_root: &Path) {
        // Document main
        if let Some(ref mut doc) = self.document {
            if let Some(ref mut main) = doc.main {
                if main.is_relative() {
                    *main = project_root.join(&*main);
                }
            }
        }

        // Output directory
        if let Some(ref mut output) = self.output {
            if output.directory.is_relative() {
                output.directory = project_root.join(&output.directory);
            }
        }

        // Font paths
        if let Some(ref mut fonts) = self.fonts {
            fonts.paths = fonts
                .paths
                .iter()
                .map(|p| {
                    if p.is_relative() {
                        project_root.join(p)
                    } else {
                        p.clone()
                    }
                })
                .collect();
        }

        // Bibliography files
        if let Some(ref mut bib) = self.bibliography {
            bib.bibliography = bib
                .bibliography
                .iter()
                .map(|p| {
                    if p.is_relative() {
                        project_root.join(p)
                    } else {
                        p.clone()
                    }
                })
                .collect();
        }

        // Python paths
        if let Some(ref mut py) = self.python {
            py.project_root = project_root.to_path_buf();
            if py.figures_dir.is_relative() {
                py.figures_dir = project_root.join(&py.figures_dir);
            }
            if py.cache_dir.is_relative() {
                py.cache_dir = project_root.join(&py.cache_dir);
            }
            if let Some(ref venv) = py.venv {
                if venv.is_relative() {
                    py.venv = Some(project_root.join(venv));
                }
            }
            py.python_path = py
                .python_path
                .iter()
                .map(|p| {
                    if p.is_relative() {
                        project_root.join(p)
                    } else {
                        p.clone()
                    }
                })
                .collect();
        }

        // Data file paths
        if let Some(ref mut data) = self.data {
            *data = data
                .iter()
                .map(|(k, v)| {
                    let resolved = if v.is_relative() {
                        project_root.join(v)
                    } else {
                        v.clone()
                    };
                    (k.clone(), resolved)
                })
                .collect();
        }
    }
}

// ─── Config error ────────────────────────────────────────────────────

/// Errors that can occur when loading `nova.toml`.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// `nova.toml` not found in project directory.
    #[error("nova.toml not found: {path}")]
    NotFound { path: PathBuf },

    /// TOML parse error.
    #[error("Failed to parse {path}: {message}")]
    Parse { path: PathBuf, message: String },

    /// I/O error reading the file.
    #[error("Failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

// ─── Section structs ─────────────────────────────────────────────────

/// `[project]` — Project metadata.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProjectConfig {
    /// Project name.
    pub name: String,
    /// Project version.
    #[serde(default = "default_version")]
    pub version: String,
    /// Project description.
    #[serde(default)]
    pub description: Option<String>,
    /// List of authors.
    #[serde(default)]
    pub authors: Vec<String>,
    /// License identifier.
    #[serde(default)]
    pub license: Option<String>,
    /// Keywords for the project.
    #[serde(default)]
    pub keywords: Vec<String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// `[document]` — Main document settings.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DocumentConfig {
    /// Main document file (e.g. "main.typ").
    pub main: Option<PathBuf>,
    /// Template name.
    pub template: Option<String>,
}

/// `[output]` — Build output settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputConfig {
    /// Output format: "pdf", "svg", "png".
    #[serde(default = "default_format")]
    pub format: String,
    /// Output directory for build artifacts.
    #[serde(default = "default_output_dir")]
    pub directory: PathBuf,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            directory: default_output_dir(),
        }
    }
}

fn default_format() -> String {
    "pdf".to_string()
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("build")
}

/// `[bibliography]` — Citation style and bibliography files.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BibliographyConfig {
    /// Citation style (e.g. "ieee", "apa", "chicago").
    #[serde(default = "default_citation_style")]
    pub style: String,
    /// Paths to bibliography files.
    #[serde(default)]
    pub bibliography: Vec<PathBuf>,
}

fn default_citation_style() -> String {
    "ieee".to_string()
}

/// `[watch]` — Watch mode settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WatchConfig {
    /// Debounce interval in milliseconds.
    #[serde(default = "default_debounce")]
    pub debounce: u64,
    /// Glob patterns to ignore.
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Clear terminal before each rebuild.
    #[serde(default)]
    pub clear: bool,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            debounce: default_debounce(),
            ignore: Vec::new(),
            clear: false,
        }
    }
}

fn default_debounce() -> u64 {
    300
}

// ─── Python config ───────────────────────────────────────────────────

/// `[python]` — Python figure integration configuration.
///
/// Loaded from the `[python]` section of `nova.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    /// Path to the Python executable. Defaults to "python" (uses PATH).
    #[serde(default = "default_python")]
    pub python: String,

    /// Directory containing Python figure scripts. Defaults to "figures".
    #[serde(default = "default_figures_dir")]
    pub figures_dir: PathBuf,

    /// Directory for cached SVG outputs. Defaults to ".nova/cache".
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,

    /// Virtual environment path (optional).
    pub venv: Option<PathBuf>,

    /// Additional Python paths to add to PYTHONPATH.
    #[serde(default)]
    pub python_path: Vec<PathBuf>,

    /// Environment variables to set for Python execution.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Timeout for figure generation in seconds.
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Project root directory (set automatically, not from TOML).
    #[serde(skip)]
    pub project_root: PathBuf,
}

fn default_python() -> String {
    "python".to_string()
}

fn default_figures_dir() -> PathBuf {
    PathBuf::from("figures")
}

fn default_cache_dir() -> PathBuf {
    PathBuf::from(".nova/cache")
}

fn default_timeout() -> u64 {
    60
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            python: default_python(),
            figures_dir: default_figures_dir(),
            cache_dir: default_cache_dir(),
            venv: None,
            python_path: Vec::new(),
            env: HashMap::new(),
            timeout: default_timeout(),
            project_root: PathBuf::new(),
        }
    }
}

impl PythonConfig {
    /// Get the effective Python executable path.
    ///
    /// If a venv is configured, returns the Python from that venv.
    pub fn python_executable(&self) -> PathBuf {
        if let Some(ref venv) = self.venv {
            #[cfg(windows)]
            let python = venv.join("Scripts").join("python.exe");
            #[cfg(not(windows))]
            let python = venv.join("bin").join("python");
            python
        } else {
            PathBuf::from(&self.python)
        }
    }

    /// Build the PYTHONPATH environment variable.
    pub fn build_python_path(&self) -> String {
        // Include project root so "import figures.xxx" works
        let mut paths: Vec<String> = vec![self.project_root.display().to_string()];

        // Also add figures_dir parent for nested imports
        if let Some(parent) = self.figures_dir.parent() {
            paths.push(parent.display().to_string());
        }

        for p in &self.python_path {
            paths.push(p.display().to_string());
        }

        #[cfg(windows)]
        let separator = ";";
        #[cfg(not(windows))]
        let separator = ":";

        paths.join(separator)
    }
}

// ─── Font types ──────────────────────────────────────────────────────

/// `[fonts]` — Font paths and font mappings.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FontsConfig {
    /// Additional font search paths.
    #[serde(default)]
    pub paths: Vec<PathBuf>,
    /// Main body font.
    pub main: Option<FontSpec>,
    /// Font for headings.
    pub heading: Option<FontSpec>,
    /// Monospace font for code.
    pub mono: Option<FontSpec>,
    /// Font for math expressions.
    pub math: Option<FontSpec>,
}

impl FontsConfig {
    /// Check if any font mapping is configured.
    pub fn has_font_mappings(&self) -> bool {
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

/// Single font specification — can be a simple string or a full spec.
#[derive(Debug, Clone, Deserialize, Serialize)]
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

/// Font configuration (used for frontmatter parsing).
///
/// This is a convenience alias for the fonts section,
/// used when parsing font settings from document frontmatter.
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

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn load_full_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("nova.toml");
        let mut file = std::fs::File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"
[project]
name = "test-project"
version = "1.0.0"
description = "A test project"
authors = ["Test Author"]

[document]
main = "main.typ"
template = "ieee-article"

[output]
format = "pdf"
directory = "build"

[fonts]
paths = ["fonts"]
main = "Inter"

[bibliography]
style = "apa"
bibliography = ["refs.bib"]

[python]
python = "python3"
figures_dir = "figs"
timeout = 120

[watch]
debounce = 500
clear = true
ignore = ["*.tmp"]

[data]
results = "data/results.json"
"#
        )
        .unwrap();

        let config = NovaConfig::from_project(dir.path()).unwrap();

        let project = config.project.unwrap();
        assert_eq!(project.name, "test-project");
        assert_eq!(project.version, "1.0.0");

        let doc = config.document.unwrap();
        assert_eq!(doc.main.unwrap(), dir.path().join("main.typ"));
        assert_eq!(doc.template.unwrap(), "ieee-article");

        let output = config.output.unwrap();
        assert_eq!(output.format, "pdf");

        let fonts = config.fonts.unwrap();
        assert_eq!(fonts.paths.len(), 1);
        assert!(fonts.main.is_some());

        let bib = config.bibliography.unwrap();
        assert_eq!(bib.style, "apa");
        assert_eq!(bib.bibliography.len(), 1);

        let python = config.python.unwrap();
        assert_eq!(python.python, "python3");
        assert_eq!(python.timeout, 120);

        let watch = config.watch.unwrap();
        assert_eq!(watch.debounce, 500);
        assert!(watch.clear);

        let data = config.data.unwrap();
        assert!(data.contains_key("results"));
    }

    #[test]
    fn missing_config_returns_not_found() {
        let dir = TempDir::new().unwrap();
        let result = NovaConfig::from_project(dir.path());
        assert!(matches!(result, Err(ConfigError::NotFound { .. })));
    }

    #[test]
    fn missing_config_or_default_returns_default() {
        let dir = TempDir::new().unwrap();
        let config = NovaConfig::from_project_or_default(dir.path()).unwrap();
        assert!(config.project.is_none());
        assert!(config.python.is_none());
    }

    #[test]
    fn partial_config_works() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("nova.toml"),
            r#"
[project]
name = "minimal"
"#,
        )
        .unwrap();

        let config = NovaConfig::from_project(dir.path()).unwrap();
        assert_eq!(config.project.unwrap().name, "minimal");
        assert!(config.python.is_none());
        assert!(config.output.is_none());
    }

    #[test]
    fn python_config_defaults() {
        let config = PythonConfig::default();
        assert_eq!(config.python, "python");
        assert_eq!(config.figures_dir, PathBuf::from("figures"));
        assert_eq!(config.timeout, 60);
    }

    #[test]
    fn font_spec_simple() {
        let spec = FontSpec::Simple("Inter".to_string());
        assert_eq!(spec.family(), "Inter");
        assert_eq!(spec.to_typst_args(), "font: \"Inter\"");
    }

    #[test]
    fn font_spec_full() {
        let spec = FontSpec::Full {
            family: "Open Sans".to_string(),
            size: Some("14pt".to_string()),
            weight: Some("bold".to_string()),
            style: None,
        };
        assert_eq!(spec.family(), "Open Sans");
        let args = spec.to_typst_args();
        assert!(args.contains("font: \"Open Sans\""));
        assert!(args.contains("size: 14pt"));
        assert!(args.contains("weight: \"bold\""));
    }

    #[test]
    fn fonts_config_to_typst_code() {
        let fonts = FontsConfig {
            paths: vec![],
            main: Some(FontSpec::Simple("Inter".to_string())),
            heading: Some(FontSpec::Simple("Open Sans".to_string())),
            mono: Some(FontSpec::Simple("Fira Code".to_string())),
            math: None,
        };

        let code = fonts.to_typst_code();
        assert!(code.contains("#set text(font: \"Inter\")"));
        assert!(code.contains("#show heading: set text(font: \"Open Sans\")"));
        assert!(code.contains("#show raw: set text(font: \"Fira Code\")"));
    }

    #[test]
    fn empty_fonts_config() {
        let fonts = FontsConfig::default();
        assert!(!fonts.has_font_mappings());
        assert!(fonts.to_typst_code().is_empty());
    }

    #[test]
    fn invalid_toml_returns_parse_error() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("nova.toml"), "invalid = [[[").unwrap();

        let result = NovaConfig::from_project(dir.path());
        assert!(matches!(result, Err(ConfigError::Parse { .. })));
    }

    #[test]
    fn path_resolution() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("nova.toml"),
            r#"
[document]
main = "main.typ"

[output]
directory = "build"

[fonts]
paths = ["custom-fonts"]

[bibliography]
bibliography = ["refs.bib"]

[data]
results = "data/results.json"
"#,
        )
        .unwrap();

        let config = NovaConfig::from_project(dir.path()).unwrap();

        assert_eq!(
            config.document.unwrap().main.unwrap(),
            dir.path().join("main.typ")
        );
        assert_eq!(config.output.unwrap().directory, dir.path().join("build"));
        assert_eq!(
            config.fonts.unwrap().paths[0],
            dir.path().join("custom-fonts")
        );
        assert_eq!(
            config.bibliography.unwrap().bibliography[0],
            dir.path().join("refs.bib")
        );
        assert_eq!(
            config.data.unwrap()["results"],
            dir.path().join("data/results.json")
        );
    }
}
