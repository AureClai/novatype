//! Chart styling configuration.

use serde::{Deserialize, Serialize};

/// Visual style configuration for charts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartStyle {
    /// Background color.
    pub background: Color,

    /// Primary color for data.
    pub primary: Color,

    /// Secondary colors for additional series.
    pub palette: Vec<Color>,

    /// Axis color.
    pub axis_color: Color,

    /// Grid line color.
    pub grid_color: Color,

    /// Text color.
    pub text_color: Color,

    /// Font family.
    pub font_family: String,

    /// Title font size.
    pub title_font_size: f64,

    /// Label font size.
    pub label_font_size: f64,

    /// Axis font size.
    pub axis_font_size: f64,

    /// Line width for line charts.
    pub line_width: f64,

    /// Point radius for scatter plots.
    pub point_radius: f64,

    /// Show grid lines.
    pub show_grid: bool,

    /// Show legend.
    pub show_legend: bool,

    /// Padding around the chart.
    pub padding: Padding,
}

impl Default for ChartStyle {
    fn default() -> Self {
        Self {
            background: Color::WHITE,
            primary: Color::from_hex("#4285f4"),
            palette: vec![
                Color::from_hex("#4285f4"), // Blue
                Color::from_hex("#ea4335"), // Red
                Color::from_hex("#fbbc04"), // Yellow
                Color::from_hex("#34a853"), // Green
                Color::from_hex("#9334a8"), // Purple
                Color::from_hex("#ff6d01"), // Orange
            ],
            axis_color: Color::from_hex("#333333"),
            grid_color: Color::from_hex("#e0e0e0"),
            text_color: Color::from_hex("#333333"),
            font_family: "Arial, sans-serif".to_string(),
            title_font_size: 18.0,
            label_font_size: 14.0,
            axis_font_size: 12.0,
            line_width: 2.0,
            point_radius: 4.0,
            show_grid: true,
            show_legend: true,
            padding: Padding::default(),
        }
    }
}

impl ChartStyle {
    /// Create a dark theme style.
    #[must_use]
    pub fn dark() -> Self {
        Self {
            background: Color::from_hex("#1e1e1e"),
            primary: Color::from_hex("#61afef"),
            palette: vec![
                Color::from_hex("#61afef"),
                Color::from_hex("#e06c75"),
                Color::from_hex("#e5c07b"),
                Color::from_hex("#98c379"),
                Color::from_hex("#c678dd"),
                Color::from_hex("#d19a66"),
            ],
            axis_color: Color::from_hex("#abb2bf"),
            grid_color: Color::from_hex("#3e4451"),
            text_color: Color::from_hex("#abb2bf"),
            ..Default::default()
        }
    }

    /// Create a minimal style.
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            show_grid: false,
            show_legend: false,
            ..Default::default()
        }
    }

    /// Get color for a series index.
    #[must_use]
    pub fn series_color(&self, index: usize) -> &Color {
        if index == 0 {
            &self.primary
        } else {
            &self.palette[(index - 1) % self.palette.len()]
        }
    }
}

/// RGB color.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    /// Red component (0-255).
    pub r: u8,
    /// Green component (0-255).
    pub g: u8,
    /// Blue component (0-255).
    pub b: u8,
    /// Alpha component (0-255).
    pub a: u8,
}

impl Color {
    /// White color.
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    /// Black color.
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    /// Create a new color.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a new color with alpha.
    #[must_use]
    pub const fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create from hex string (e.g., "#ff0000" or "ff0000").
    #[must_use]
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 && hex.len() != 8 {
            return Self::BLACK;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
        } else {
            255
        };

        Self { r, g, b, a }
    }

    /// Convert to hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }

    /// Convert to CSS rgba() string.
    #[must_use]
    pub fn to_rgba(&self) -> String {
        if self.a == 255 {
            format!("rgb({}, {}, {})", self.r, self.g, self.b)
        } else {
            format!(
                "rgba({}, {}, {}, {:.2})",
                self.r,
                self.g,
                self.b,
                f64::from(self.a) / 255.0
            )
        }
    }
}

/// Padding configuration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Padding {
    /// Top padding.
    pub top: f64,
    /// Right padding.
    pub right: f64,
    /// Bottom padding.
    pub bottom: f64,
    /// Left padding.
    pub left: f64,
}

impl Default for Padding {
    fn default() -> Self {
        Self {
            top: 40.0,
            right: 40.0,
            bottom: 60.0,
            left: 60.0,
        }
    }
}

impl Padding {
    /// Create uniform padding.
    #[must_use]
    pub const fn uniform(value: f64) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_from_hex() {
        let red = Color::from_hex("#ff0000");
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);

        let blue = Color::from_hex("0000ff");
        assert_eq!(blue.r, 0);
        assert_eq!(blue.g, 0);
        assert_eq!(blue.b, 255);
    }

    #[test]
    fn color_to_hex() {
        let color = Color::new(255, 128, 0);
        assert_eq!(color.to_hex(), "#ff8000");

        let with_alpha = Color::with_alpha(255, 128, 0, 128);
        assert_eq!(with_alpha.to_hex(), "#ff800080");
    }

    #[test]
    fn color_to_rgba() {
        let color = Color::new(255, 128, 0);
        assert_eq!(color.to_rgba(), "rgb(255, 128, 0)");

        let with_alpha = Color::with_alpha(255, 128, 0, 128);
        assert!(with_alpha.to_rgba().starts_with("rgba(255, 128, 0,"));
    }

    #[test]
    fn style_series_color() {
        let style = ChartStyle::default();

        // First series uses primary
        assert_eq!(style.series_color(0).to_hex(), style.primary.to_hex());

        // Others use palette
        assert_eq!(style.series_color(1).to_hex(), style.palette[0].to_hex());
    }

    #[test]
    fn dark_theme() {
        let dark = ChartStyle::dark();
        assert_eq!(dark.background.to_hex(), "#1e1e1e");
    }
}
