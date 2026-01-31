//! # Nova Plot
//!
//! Native data visualization engine for NovaType.
//!
//! This crate provides:
//! - SVG-based chart rendering
//! - Multiple chart types (line, bar, scatter, pie)
//! - CSV and JSON data parsing
//! - Configurable styling
//!
//! ## Example
//!
//! ```rust,ignore
//! use nova_plot::{Chart, ChartType, DataSource};
//!
//! let data = DataSource::from_csv("data.csv")?;
//! let chart = Chart::new(ChartType::Line)
//!     .with_title("Sales Over Time")
//!     .with_data(data);
//!
//! let svg = chart.render()?;
//! ```

pub mod chart;
pub mod data;
pub mod error;
pub mod render;
pub mod style;

pub use chart::{Chart, ChartType};
pub use data::DataSource;
pub use error::{Error, Result};
pub use style::ChartStyle;
