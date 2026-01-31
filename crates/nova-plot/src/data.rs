//! Data source handling for charts.
//!
//! Supports CSV and JSON data formats.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A data source for charts.
#[derive(Debug, Clone)]
pub enum DataSource {
    /// Tabular data with columns.
    Table(Table),
    /// Series data for multi-line charts.
    Series(Vec<Series>),
}

impl DataSource {
    /// Load data from a CSV file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_csv(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut reader = csv::Reader::from_path(path)?;

        let headers: Vec<String> = reader.headers()?.iter().map(String::from).collect();

        let mut rows = Vec::new();
        for result in reader.records() {
            let record = result?;
            let row: Vec<Value> = record.iter().map(Value::parse).collect();
            rows.push(row);
        }

        Ok(Self::Table(Table { headers, rows }))
    }

    /// Load data from a CSV string.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn from_csv_string(content: &str) -> Result<Self> {
        let mut reader = csv::Reader::from_reader(content.as_bytes());

        let headers: Vec<String> = reader.headers()?.iter().map(String::from).collect();

        let mut rows = Vec::new();
        for result in reader.records() {
            let record = result?;
            let row: Vec<Value> = record.iter().map(Value::parse).collect();
            rows.push(row);
        }

        Ok(Self::Table(Table { headers, rows }))
    }

    /// Load data from JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn from_json(content: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(content)?;
        Self::from_json_value(value)
    }

    /// Load data from a JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if the format is invalid.
    pub fn from_json_value(value: serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    return Err(Error::NoData);
                }

                // Check if it's an array of objects (table format)
                if arr[0].is_object() {
                    let headers: Vec<String> =
                        arr[0].as_object().unwrap().keys().cloned().collect();

                    let rows: Vec<Vec<Value>> = arr
                        .iter()
                        .filter_map(|obj| {
                            obj.as_object().map(|o| {
                                headers
                                    .iter()
                                    .map(|h| Value::from_json(o.get(h).cloned()))
                                    .collect()
                            })
                        })
                        .collect();

                    Ok(Self::Table(Table { headers, rows }))
                } else {
                    // Array of values (single series)
                    let values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

                    Ok(Self::Series(vec![Series {
                        name: "data".to_string(),
                        values,
                    }]))
                }
            }
            _ => Err(Error::InvalidData {
                message: "expected JSON array".to_string(),
            }),
        }
    }

    /// Create data from inline points.
    #[must_use]
    pub fn from_points(points: Vec<(f64, f64)>) -> Self {
        let headers = vec!["x".to_string(), "y".to_string()];
        let rows: Vec<Vec<Value>> = points
            .into_iter()
            .map(|(x, y)| vec![Value::Number(x), Value::Number(y)])
            .collect();

        Self::Table(Table { headers, rows })
    }

    /// Get as table if possible.
    #[must_use]
    pub fn as_table(&self) -> Option<&Table> {
        match self {
            Self::Table(t) => Some(t),
            _ => None,
        }
    }

    /// Get as series if possible.
    #[must_use]
    pub fn as_series(&self) -> Option<&[Series]> {
        match self {
            Self::Series(s) => Some(s),
            _ => None,
        }
    }
}

/// Tabular data with named columns.
#[derive(Debug, Clone)]
pub struct Table {
    /// Column headers.
    pub headers: Vec<String>,
    /// Row data.
    pub rows: Vec<Vec<Value>>,
}

impl Table {
    /// Get the column index by name.
    #[must_use]
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.headers.iter().position(|h| h == name)
    }

    /// Get a column's values as floats.
    #[must_use]
    pub fn column_as_f64(&self, name: &str) -> Option<Vec<f64>> {
        let idx = self.column_index(name)?;
        Some(
            self.rows
                .iter()
                .filter_map(|row| row.get(idx).and_then(Value::as_f64))
                .collect(),
        )
    }

    /// Get a column's values as strings.
    #[must_use]
    pub fn column_as_str(&self, name: &str) -> Option<Vec<String>> {
        let idx = self.column_index(name)?;
        Some(
            self.rows
                .iter()
                .filter_map(|row| row.get(idx).map(Value::to_string))
                .collect(),
        )
    }

    /// Get the number of rows.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get the number of columns.
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }
}

/// A named data series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    /// Series name.
    pub name: String,
    /// Data values.
    pub values: Vec<f64>,
}

/// A data value (number or string).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    /// Numeric value.
    Number(f64),
    /// String value.
    String(String),
    /// Null/missing value.
    Null,
}

impl Value {
    /// Parse a string into a value.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        if s.is_empty() {
            Self::Null
        } else if let Ok(n) = s.parse::<f64>() {
            Self::Number(n)
        } else {
            Self::String(s.to_string())
        }
    }

    /// Convert from JSON value.
    #[must_use]
    pub fn from_json(value: Option<serde_json::Value>) -> Self {
        match value {
            Some(serde_json::Value::Number(n)) => Self::Number(n.as_f64().unwrap_or(0.0)),
            Some(serde_json::Value::String(s)) => Self::String(s),
            Some(serde_json::Value::Null) | None => Self::Null,
            Some(v) => Self::String(v.to_string()),
        }
    }

    /// Get as f64 if numeric.
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            Self::String(s) => s.parse().ok(),
            Self::Null => None,
        }
    }

    /// Check if null.
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Null => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_string() {
        let csv = "x,y\n1,10\n2,20\n3,30";
        let data = DataSource::from_csv_string(csv).unwrap();

        let table = data.as_table().unwrap();
        assert_eq!(table.headers, vec!["x", "y"]);
        assert_eq!(table.row_count(), 3);
    }

    #[test]
    fn table_column_access() {
        let csv = "name,value\nalpha,100\nbeta,200";
        let data = DataSource::from_csv_string(csv).unwrap();
        let table = data.as_table().unwrap();

        assert_eq!(table.column_index("name"), Some(0));
        assert_eq!(table.column_index("value"), Some(1));
        assert_eq!(table.column_index("missing"), None);

        let values = table.column_as_f64("value").unwrap();
        assert_eq!(values, vec![100.0, 200.0]);
    }

    #[test]
    fn from_json_array_of_objects() {
        let json = r#"[
            {"x": 1, "y": 10},
            {"x": 2, "y": 20}
        ]"#;

        let data = DataSource::from_json(json).unwrap();
        let table = data.as_table().unwrap();
        assert_eq!(table.row_count(), 2);
    }

    #[test]
    fn from_points() {
        let data = DataSource::from_points(vec![(1.0, 10.0), (2.0, 20.0)]);
        let table = data.as_table().unwrap();

        assert_eq!(table.headers, vec!["x", "y"]);
        assert_eq!(table.row_count(), 2);
    }

    #[test]
    fn value_parsing() {
        assert!(matches!(Value::parse("42"), Value::Number(n) if (n - 42.0).abs() < f64::EPSILON));
        assert!(matches!(Value::parse("hello"), Value::String(_)));
        assert!(matches!(Value::parse(""), Value::Null));
    }

    #[test]
    fn value_as_f64() {
        assert_eq!(Value::Number(42.0).as_f64(), Some(42.0));
        assert_eq!(Value::String("42".to_string()).as_f64(), Some(42.0));
        assert_eq!(Value::String("hello".to_string()).as_f64(), None);
        assert_eq!(Value::Null.as_f64(), None);
    }
}
