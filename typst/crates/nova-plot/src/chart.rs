//! Chart configuration and building.

use crate::data::DataSource;
use crate::error::{Error, Result};
use crate::render::Renderer;
use crate::style::ChartStyle;

/// Type of chart to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartType {
    /// Line chart.
    #[default]
    Line,
    /// Bar chart.
    Bar,
    /// Scatter plot.
    Scatter,
    /// Pie chart.
    Pie,
    /// Area chart.
    Area,
}

impl ChartType {
    /// Parse from string.
    ///
    /// # Errors
    ///
    /// Returns an error if the type is not recognized.
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "line" => Ok(Self::Line),
            "bar" => Ok(Self::Bar),
            "scatter" => Ok(Self::Scatter),
            "pie" => Ok(Self::Pie),
            "area" => Ok(Self::Area),
            _ => Err(Error::InvalidConfig {
                message: format!("unknown chart type: {s}"),
            }),
        }
    }
}

/// A chart configuration.
#[derive(Debug, Clone)]
pub struct Chart {
    /// Chart type.
    pub chart_type: ChartType,

    /// Chart title.
    pub title: Option<String>,

    /// X-axis label.
    pub x_label: Option<String>,

    /// Y-axis label.
    pub y_label: Option<String>,

    /// X-axis column name.
    pub x_column: Option<String>,

    /// Y-axis column name.
    pub y_column: Option<String>,

    /// Data source.
    pub data: Option<DataSource>,

    /// Chart dimensions.
    pub width: f64,
    pub height: f64,

    /// Visual style.
    pub style: ChartStyle,
}

impl Chart {
    /// Create a new chart with default settings.
    #[must_use]
    pub fn new(chart_type: ChartType) -> Self {
        Self {
            chart_type,
            title: None,
            x_label: None,
            y_label: None,
            x_column: None,
            y_column: None,
            data: None,
            width: 600.0,
            height: 400.0,
            style: ChartStyle::default(),
        }
    }

    /// Set the chart title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the X-axis label.
    #[must_use]
    pub fn with_x_label(mut self, label: impl Into<String>) -> Self {
        self.x_label = Some(label.into());
        self
    }

    /// Set the Y-axis label.
    #[must_use]
    pub fn with_y_label(mut self, label: impl Into<String>) -> Self {
        self.y_label = Some(label.into());
        self
    }

    /// Set the data source.
    #[must_use]
    pub fn with_data(mut self, data: DataSource) -> Self {
        self.data = Some(data);
        self
    }

    /// Set the X column.
    #[must_use]
    pub fn with_x_column(mut self, column: impl Into<String>) -> Self {
        self.x_column = Some(column.into());
        self
    }

    /// Set the Y column.
    #[must_use]
    pub fn with_y_column(mut self, column: impl Into<String>) -> Self {
        self.y_column = Some(column.into());
        self
    }

    /// Set the chart dimensions.
    #[must_use]
    pub fn with_size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the visual style.
    #[must_use]
    pub fn with_style(mut self, style: ChartStyle) -> Self {
        self.style = style;
        self
    }

    /// Render the chart to SVG.
    ///
    /// # Errors
    ///
    /// Returns an error if no data is provided or rendering fails.
    pub fn render(&self) -> Result<String> {
        let data = self.data.as_ref().ok_or(Error::NoData)?;
        let renderer = Renderer::new(self);
        renderer.render(data)
    }

    /// Render to SVG and save to file.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering or writing fails.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let svg = self.render()?;
        std::fs::write(path, svg)?;
        Ok(())
    }
}

impl Default for Chart {
    fn default() -> Self {
        Self::new(ChartType::Line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_builder_pattern() {
        let chart = Chart::new(ChartType::Line)
            .with_title("Test Chart")
            .with_x_label("X Axis")
            .with_y_label("Y Axis")
            .with_size(800.0, 600.0);

        assert_eq!(chart.chart_type, ChartType::Line);
        assert_eq!(chart.title, Some("Test Chart".to_string()));
        assert_eq!(chart.width, 800.0);
        assert_eq!(chart.height, 600.0);
    }

    #[test]
    fn chart_type_from_str() {
        assert_eq!(ChartType::from_str("line").unwrap(), ChartType::Line);
        assert_eq!(ChartType::from_str("BAR").unwrap(), ChartType::Bar);
        assert_eq!(ChartType::from_str("Scatter").unwrap(), ChartType::Scatter);
        assert!(ChartType::from_str("invalid").is_err());
    }

    #[test]
    fn render_without_data_fails() {
        let chart = Chart::new(ChartType::Line);
        assert!(matches!(chart.render(), Err(Error::NoData)));
    }

    #[test]
    fn render_with_data() {
        let data = DataSource::from_points(vec![(1.0, 10.0), (2.0, 20.0), (3.0, 15.0)]);
        let chart = Chart::new(ChartType::Line)
            .with_title("Test")
            .with_data(data);

        let svg = chart.render().unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("Test")); // Title should be in SVG
    }
}
