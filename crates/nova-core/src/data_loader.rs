//! Load external data files and convert them to Typst variable definitions.
//!
//! Supports JSON, CSV, TOML, and YAML formats. Each data file is mapped
//! to a Typst `#let` variable.
//!
//! ## Example
//!
//! Given `nova.toml`:
//! ```toml
//! [data]
//! results = "data/results.json"
//! ```
//!
//! And `data/results.json`:
//! ```json
//! {"score": 42, "name": "test"}
//! ```
//!
//! The loader generates:
//! ```typst
//! #let results = (name: "test", score: 42)
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Load all data files and generate Typst `#let` definitions.
///
/// Returns a string of Typst code to prepend to the document source.
pub fn load_data_files(
    data: &HashMap<String, PathBuf>,
    _project_root: &Path,
) -> std::result::Result<String, DataError> {
    let mut typst_code = String::new();

    for (var_name, file_path) in data {
        if !is_valid_typst_identifier(var_name) {
            return Err(DataError::InvalidIdentifier {
                name: var_name.clone(),
            });
        }

        let content = std::fs::read_to_string(file_path).map_err(|e| DataError::ReadFailed {
            path: file_path.clone(),
            source: e,
        })?;

        let typst_value = match file_path.extension().and_then(|s| s.to_str()) {
            Some("json") => json_to_typst(&content).map_err(|e| DataError::ParseFailed {
                path: file_path.clone(),
                message: e,
            })?,
            Some("csv") => csv_to_typst(&content).map_err(|e| DataError::ParseFailed {
                path: file_path.clone(),
                message: e,
            })?,
            Some("toml") => toml_to_typst(&content).map_err(|e| DataError::ParseFailed {
                path: file_path.clone(),
                message: e,
            })?,
            Some("yaml" | "yml") => {
                yaml_to_typst(&content).map_err(|e| DataError::ParseFailed {
                    path: file_path.clone(),
                    message: e,
                })?
            }
            _ => {
                return Err(DataError::UnsupportedFormat {
                    path: file_path.clone(),
                });
            }
        };

        typst_code.push_str(&format!("#let {} = {}\n", var_name, typst_value));
    }

    if !typst_code.is_empty() {
        typst_code.push('\n');
    }

    Ok(typst_code)
}

/// Errors from data loading.
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("Invalid Typst identifier in [data]: '{name}'")]
    InvalidIdentifier { name: String },

    #[error("Failed to read data file {path}: {source}")]
    ReadFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to parse data file {path}: {message}")]
    ParseFailed { path: PathBuf, message: String },

    #[error("Unsupported data file format: {path}")]
    UnsupportedFormat { path: PathBuf },
}

fn is_valid_typst_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.starts_with(|c: char| c.is_alphabetic() || c == '_')
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

// ─── JSON ────────────────────────────────────────────────────────────

fn json_to_typst(json: &str) -> std::result::Result<String, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("JSON: {}", e))?;
    Ok(json_value_to_typst(&value))
}

fn json_value_to_typst(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "none".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => {
            format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
        }
        serde_json::Value::Array(arr) => {
            let items: Vec<_> = arr.iter().map(json_value_to_typst).collect();
            format!("({})", items.join(", "))
        }
        serde_json::Value::Object(obj) => {
            if obj.is_empty() {
                return "(:)".to_string();
            }
            let items: Vec<_> = obj
                .iter()
                .map(|(k, v)| {
                    let key = sanitize_typst_key(k);
                    format!("{}: {}", key, json_value_to_typst(v))
                })
                .collect();
            format!("({})", items.join(", "))
        }
    }
}

// ─── CSV ─────────────────────────────────────────────────────────────

fn csv_to_typst(csv_str: &str) -> std::result::Result<String, String> {
    let mut reader = csv::Reader::from_reader(csv_str.as_bytes());
    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| format!("CSV headers: {}", e))?
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| format!("CSV row: {}", e))?;
        let fields: Vec<_> = headers
            .iter()
            .zip(record.iter())
            .map(|(header, field)| {
                let key = sanitize_typst_key(header);
                let value = infer_typst_value(field);
                format!("{}: {}", key, value)
            })
            .collect();
        rows.push(format!("({})", fields.join(", ")));
    }

    Ok(format!("({})", rows.join(", ")))
}

// ─── TOML ────────────────────────────────────────────────────────────

fn toml_to_typst(toml_str: &str) -> std::result::Result<String, String> {
    let value: toml::Value = toml::from_str(toml_str).map_err(|e| format!("TOML: {}", e))?;
    Ok(toml_value_to_typst(&value))
}

fn toml_value_to_typst(value: &toml::Value) -> String {
    match value {
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        toml::Value::Array(arr) => {
            let items: Vec<_> = arr.iter().map(toml_value_to_typst).collect();
            format!("({})", items.join(", "))
        }
        toml::Value::Table(table) => {
            if table.is_empty() {
                return "(:)".to_string();
            }
            let items: Vec<_> = table
                .iter()
                .map(|(k, v)| {
                    let key = sanitize_typst_key(k);
                    format!("{}: {}", key, toml_value_to_typst(v))
                })
                .collect();
            format!("({})", items.join(", "))
        }
        toml::Value::Datetime(dt) => format!("\"{}\"", dt),
    }
}

// ─── YAML ────────────────────────────────────────────────────────────

fn yaml_to_typst(yaml: &str) -> std::result::Result<String, String> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(yaml).map_err(|e| format!("YAML: {}", e))?;
    Ok(yaml_value_to_typst(&value))
}

fn yaml_value_to_typst(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::Null => "none".to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64() {
                f.to_string()
            } else {
                n.to_string()
            }
        }
        serde_yaml::Value::String(s) => {
            format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
        }
        serde_yaml::Value::Sequence(seq) => {
            let items: Vec<_> = seq.iter().map(yaml_value_to_typst).collect();
            format!("({})", items.join(", "))
        }
        serde_yaml::Value::Mapping(map) => {
            if map.is_empty() {
                return "(:)".to_string();
            }
            let items: Vec<_> = map
                .iter()
                .filter_map(|(k, v)| {
                    let key_str = match k {
                        serde_yaml::Value::String(s) => s.clone(),
                        _ => return None,
                    };
                    let key = sanitize_typst_key(&key_str);
                    Some(format!("{}: {}", key, yaml_value_to_typst(v)))
                })
                .collect();
            format!("({})", items.join(", "))
        }
        _ => "none".to_string(),
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────

/// Infer the Typst type for a CSV field value.
///
/// Attempts to parse as integer, float, or boolean before falling back to string.
fn infer_typst_value(field: &str) -> String {
    let trimmed = field.trim();

    if trimmed.eq_ignore_ascii_case("true") {
        return "true".to_string();
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return "false".to_string();
    }
    if let Ok(_n) = trimmed.parse::<i64>() {
        return trimmed.to_string();
    }
    if let Ok(_n) = trimmed.parse::<f64>() {
        return trimmed.to_string();
    }

    format!(
        "\"{}\"",
        trimmed.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

/// Sanitize a key for use in Typst dict syntax.
///
/// Typst dictionary keys with special characters need quoting.
fn sanitize_typst_key(key: &str) -> String {
    if key
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        && !key.is_empty()
    {
        key.to_string()
    } else {
        format!("\"{}\"", key.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_simple_object() {
        let result = json_to_typst(r#"{"name": "test", "count": 42}"#).unwrap();
        assert!(result.contains("name: \"test\""));
        assert!(result.contains("count: 42"));
    }

    #[test]
    fn json_nested() {
        let result = json_to_typst(r#"{"a": {"b": 1}, "c": [1, 2]}"#).unwrap();
        assert!(result.contains("a: (b: 1)"));
        assert!(result.contains("c: (1, 2)"));
    }

    #[test]
    fn json_null_and_bool() {
        let result = json_to_typst(r#"{"x": null, "y": true}"#).unwrap();
        assert!(result.contains("x: none"));
        assert!(result.contains("y: true"));
    }

    #[test]
    fn csv_basic() {
        let csv = "name,age\nAlice,30\nBob,25\n";
        let result = csv_to_typst(csv).unwrap();
        // Should produce array of dicts with type inference
        assert!(result.contains("name: \"Alice\""));
        assert!(result.contains("age: 30")); // numeric inference
        assert!(result.contains("name: \"Bob\""));
        assert!(result.contains("age: 25"));
        assert!(!result.contains("headers:"));
    }

    #[test]
    fn toml_basic() {
        let toml_str = r#"
name = "test"
count = 42
active = true
"#;
        let result = toml_to_typst(toml_str).unwrap();
        assert!(result.contains("name: \"test\""));
        assert!(result.contains("count: 42"));
        assert!(result.contains("active: true"));
    }

    #[test]
    fn yaml_basic() {
        let yaml = "name: test\ncount: 42\n";
        let result = yaml_to_typst(yaml).unwrap();
        assert!(result.contains("name: \"test\""));
        assert!(result.contains("count: 42"));
    }

    #[test]
    fn valid_identifiers() {
        assert!(is_valid_typst_identifier("myvar"));
        assert!(is_valid_typst_identifier("my_var"));
        assert!(is_valid_typst_identifier("my-var"));
        assert!(is_valid_typst_identifier("_private"));
        assert!(!is_valid_typst_identifier("2var"));
        assert!(!is_valid_typst_identifier(""));
    }

    #[test]
    fn load_json_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let json_path = dir.path().join("data.json");
        std::fs::write(&json_path, r#"{"score": 100}"#).unwrap();

        let mut data = HashMap::new();
        data.insert("scores".to_string(), json_path);

        let result = load_data_files(&data, dir.path()).unwrap();
        assert!(result.contains("#let scores = (score: 100)"));
    }

    #[test]
    fn load_csv_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let csv_path = dir.path().join("data.csv");
        std::fs::write(&csv_path, "x,y\n1,2\n3,4\n").unwrap();

        let mut data = HashMap::new();
        data.insert("points".to_string(), csv_path);

        let result = load_data_files(&data, dir.path()).unwrap();
        assert!(result.contains("#let points ="));
        assert!(result.contains("x: 1"));
        assert!(result.contains("y: 2"));
    }

    #[test]
    fn csv_type_inference() {
        let csv = "name,score,rate,active\nAlice,100,3.14,true\n";
        let result = csv_to_typst(csv).unwrap();
        assert!(result.contains("name: \"Alice\""));
        assert!(result.contains("score: 100"));
        assert!(result.contains("rate: 3.14"));
        assert!(result.contains("active: true"));
    }

    #[test]
    fn unsupported_format_errors() {
        let dir = tempfile::TempDir::new().unwrap();
        let txt_path = dir.path().join("data.txt");
        std::fs::write(&txt_path, "hello").unwrap();

        let mut data = HashMap::new();
        data.insert("stuff".to_string(), txt_path);

        let result = load_data_files(&data, dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_identifier_errors() {
        let dir = tempfile::TempDir::new().unwrap();
        let json_path = dir.path().join("data.json");
        std::fs::write(&json_path, "{}").unwrap();

        let mut data = HashMap::new();
        data.insert("2bad".to_string(), json_path);

        let result = load_data_files(&data, dir.path());
        assert!(result.is_err());
    }
}
