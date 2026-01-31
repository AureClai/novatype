//! SVG rendering for charts.

use crate::chart::{Chart, ChartType};
use crate::data::{DataSource, Table};
use crate::error::{Error, Result};
use crate::style::Color;

/// Chart renderer.
pub struct Renderer<'a> {
    chart: &'a Chart,
}

impl<'a> Renderer<'a> {
    /// Create a new renderer.
    pub fn new(chart: &'a Chart) -> Self {
        Self { chart }
    }

    /// Render the chart to SVG.
    pub fn render(&self, data: &DataSource) -> Result<String> {
        match self.chart.chart_type {
            ChartType::Line => self.render_line(data),
            ChartType::Bar => self.render_bar(data),
            ChartType::Scatter => self.render_scatter(data),
            ChartType::Pie => self.render_pie(data),
            ChartType::Area => self.render_area(data),
        }
    }

    /// Render a line chart.
    fn render_line(&self, data: &DataSource) -> Result<String> {
        let table = data.as_table().ok_or(Error::InvalidData {
            message: "line chart requires table data".to_string(),
        })?;

        let (x_data, y_data) = self.extract_xy(table)?;
        let (x_min, x_max, y_min, y_max) = self.calculate_bounds(&x_data, &y_data);

        let mut svg = self.svg_header();
        svg.push_str(&self.render_background());

        if self.chart.style.show_grid {
            svg.push_str(&self.render_grid(x_min, x_max, y_min, y_max));
        }

        svg.push_str(&self.render_axes(x_min, x_max, y_min, y_max));
        svg.push_str(&self.render_line_path(&x_data, &y_data, x_min, x_max, y_min, y_max));

        if let Some(ref title) = self.chart.title {
            svg.push_str(&self.render_title(title));
        }

        svg.push_str(&self.render_axis_labels());
        svg.push_str("</svg>");

        Ok(svg)
    }

    /// Render a bar chart.
    fn render_bar(&self, data: &DataSource) -> Result<String> {
        let table = data.as_table().ok_or(Error::InvalidData {
            message: "bar chart requires table data".to_string(),
        })?;

        let (labels, values) = self.extract_labels_values(table)?;
        let y_max = values.iter().cloned().fold(0.0_f64, f64::max);

        let mut svg = self.svg_header();
        svg.push_str(&self.render_background());
        svg.push_str(&self.render_bars(&labels, &values, y_max));

        if let Some(ref title) = self.chart.title {
            svg.push_str(&self.render_title(title));
        }

        svg.push_str("</svg>");

        Ok(svg)
    }

    /// Render a scatter plot.
    fn render_scatter(&self, data: &DataSource) -> Result<String> {
        let table = data.as_table().ok_or(Error::InvalidData {
            message: "scatter chart requires table data".to_string(),
        })?;

        let (x_data, y_data) = self.extract_xy(table)?;
        let (x_min, x_max, y_min, y_max) = self.calculate_bounds(&x_data, &y_data);

        let mut svg = self.svg_header();
        svg.push_str(&self.render_background());

        if self.chart.style.show_grid {
            svg.push_str(&self.render_grid(x_min, x_max, y_min, y_max));
        }

        svg.push_str(&self.render_axes(x_min, x_max, y_min, y_max));
        svg.push_str(&self.render_points(&x_data, &y_data, x_min, x_max, y_min, y_max));

        if let Some(ref title) = self.chart.title {
            svg.push_str(&self.render_title(title));
        }

        svg.push_str("</svg>");

        Ok(svg)
    }

    /// Render a pie chart.
    fn render_pie(&self, data: &DataSource) -> Result<String> {
        let table = data.as_table().ok_or(Error::InvalidData {
            message: "pie chart requires table data".to_string(),
        })?;

        let (labels, values) = self.extract_labels_values(table)?;

        let mut svg = self.svg_header();
        svg.push_str(&self.render_background());
        svg.push_str(&self.render_pie_slices(&labels, &values));

        if let Some(ref title) = self.chart.title {
            svg.push_str(&self.render_title(title));
        }

        svg.push_str("</svg>");

        Ok(svg)
    }

    /// Render an area chart.
    fn render_area(&self, data: &DataSource) -> Result<String> {
        // Area chart is similar to line chart but filled
        let table = data.as_table().ok_or(Error::InvalidData {
            message: "area chart requires table data".to_string(),
        })?;

        let (x_data, y_data) = self.extract_xy(table)?;
        let (x_min, x_max, y_min, y_max) = self.calculate_bounds(&x_data, &y_data);

        let mut svg = self.svg_header();
        svg.push_str(&self.render_background());

        if self.chart.style.show_grid {
            svg.push_str(&self.render_grid(x_min, x_max, y_min, y_max));
        }

        svg.push_str(&self.render_axes(x_min, x_max, y_min, y_max));
        svg.push_str(&self.render_area_path(&x_data, &y_data, x_min, x_max, y_min, y_max));

        if let Some(ref title) = self.chart.title {
            svg.push_str(&self.render_title(title));
        }

        svg.push_str("</svg>");

        Ok(svg)
    }

    // Helper methods

    fn svg_header(&self) -> String {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
            self.chart.width, self.chart.height, self.chart.width, self.chart.height
        )
    }

    fn render_background(&self) -> String {
        format!(
            r#"<rect width="100%" height="100%" fill="{}"/>"#,
            self.chart.style.background.to_hex()
        )
    }

    fn render_title(&self, title: &str) -> String {
        format!(
            r#"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" fill="{}">{}</text>"#,
            self.chart.width / 2.0,
            self.chart.style.padding.top / 2.0 + 5.0,
            self.chart.style.font_family,
            self.chart.style.title_font_size,
            self.chart.style.text_color.to_hex(),
            html_escape(title)
        )
    }

    fn render_axis_labels(&self) -> String {
        let mut svg = String::new();

        if let Some(ref label) = self.chart.x_label {
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" fill="{}">{}</text>"#,
                self.chart.width / 2.0,
                self.chart.height - 10.0,
                self.chart.style.font_family,
                self.chart.style.label_font_size,
                self.chart.style.text_color.to_hex(),
                html_escape(label)
            ));
        }

        if let Some(ref label) = self.chart.y_label {
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" fill="{}" transform="rotate(-90, {}, {})">{}</text>"#,
                15.0,
                self.chart.height / 2.0,
                self.chart.style.font_family,
                self.chart.style.label_font_size,
                self.chart.style.text_color.to_hex(),
                15.0,
                self.chart.height / 2.0,
                html_escape(label)
            ));
        }

        svg
    }

    fn plot_area(&self) -> (f64, f64, f64, f64) {
        let padding = &self.chart.style.padding;
        (
            padding.left,
            padding.top,
            self.chart.width - padding.left - padding.right,
            self.chart.height - padding.top - padding.bottom,
        )
    }

    fn render_grid(&self, _x_min: f64, _x_max: f64, _y_min: f64, _y_max: f64) -> String {
        let (x, y, w, h) = self.plot_area();
        let grid_color = self.chart.style.grid_color.to_hex();
        let mut svg = String::new();

        // Horizontal grid lines
        for i in 0..=5 {
            let y_pos = y + h - (h * f64::from(i) / 5.0);
            svg.push_str(&format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>"#,
                x,
                y_pos,
                x + w,
                y_pos,
                grid_color
            ));
        }

        // Vertical grid lines
        for i in 0..=5 {
            let x_pos = x + (w * f64::from(i) / 5.0);
            svg.push_str(&format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>"#,
                x_pos,
                y,
                x_pos,
                y + h,
                grid_color
            ));
        }

        svg
    }

    fn render_axes(&self, x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> String {
        let (x, y, w, h) = self.plot_area();
        let axis_color = self.chart.style.axis_color.to_hex();
        let text_color = self.chart.style.text_color.to_hex();
        let mut svg = String::new();

        // X axis
        svg.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="2"/>"#,
            x,
            y + h,
            x + w,
            y + h,
            axis_color
        ));

        // Y axis
        svg.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="2"/>"#,
            x,
            y,
            x,
            y + h,
            axis_color
        ));

        // X axis ticks and labels
        for i in 0..=5 {
            let x_pos = x + (w * f64::from(i) / 5.0);
            let value = x_min + (x_max - x_min) * f64::from(i) / 5.0;
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" fill="{}">{:.1}</text>"#,
                x_pos, y + h + 20.0, self.chart.style.font_family, self.chart.style.axis_font_size, text_color, value
            ));
        }

        // Y axis ticks and labels
        for i in 0..=5 {
            let y_pos = y + h - (h * f64::from(i) / 5.0);
            let value = y_min + (y_max - y_min) * f64::from(i) / 5.0;
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" text-anchor="end" font-family="{}" font-size="{}" fill="{}">{:.1}</text>"#,
                x - 5.0, y_pos + 4.0, self.chart.style.font_family, self.chart.style.axis_font_size, text_color, value
            ));
        }

        svg
    }

    fn render_line_path(
        &self,
        x_data: &[f64],
        y_data: &[f64],
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
    ) -> String {
        let (px, py, pw, ph) = self.plot_area();
        let color = self.chart.style.primary.to_hex();

        let points: Vec<String> = x_data
            .iter()
            .zip(y_data.iter())
            .map(|(&x, &y)| {
                let sx = px + (x - x_min) / (x_max - x_min) * pw;
                let sy = py + ph - (y - y_min) / (y_max - y_min) * ph;
                format!("{sx},{sy}")
            })
            .collect();

        format!(
            r#"<polyline points="{}" fill="none" stroke="{}" stroke-width="{}"/>"#,
            points.join(" "),
            color,
            self.chart.style.line_width
        )
    }

    fn render_area_path(
        &self,
        x_data: &[f64],
        y_data: &[f64],
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
    ) -> String {
        let (px, py, pw, ph) = self.plot_area();
        let color = self.chart.style.primary;
        let fill_color = Color::with_alpha(color.r, color.g, color.b, 100);

        let mut path = String::new();

        // Move to first point
        if let (Some(&first_x), Some(&first_y)) = (x_data.first(), y_data.first()) {
            let sx = px + (first_x - x_min) / (x_max - x_min) * pw;
            let sy = py + ph - (first_y - y_min) / (y_max - y_min) * ph;
            path.push_str(&format!("M{sx},{} L{sx},{sy}", py + ph));
        }

        // Line through all points
        for (&x, &y) in x_data.iter().zip(y_data.iter()) {
            let sx = px + (x - x_min) / (x_max - x_min) * pw;
            let sy = py + ph - (y - y_min) / (y_max - y_min) * ph;
            path.push_str(&format!(" L{sx},{sy}"));
        }

        // Close path
        if let Some(&last_x) = x_data.last() {
            let sx = px + (last_x - x_min) / (x_max - x_min) * pw;
            path.push_str(&format!(" L{sx},{} Z", py + ph));
        }

        format!(
            r#"<path d="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
            path,
            fill_color.to_rgba(),
            color.to_hex(),
            self.chart.style.line_width
        )
    }

    fn render_points(
        &self,
        x_data: &[f64],
        y_data: &[f64],
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
    ) -> String {
        let (px, py, pw, ph) = self.plot_area();
        let color = self.chart.style.primary.to_hex();

        x_data
            .iter()
            .zip(y_data.iter())
            .map(|(&x, &y)| {
                let sx = px + (x - x_min) / (x_max - x_min) * pw;
                let sy = py + ph - (y - y_min) / (y_max - y_min) * ph;
                format!(
                    r#"<circle cx="{}" cy="{}" r="{}" fill="{}"/>"#,
                    sx, sy, self.chart.style.point_radius, color
                )
            })
            .collect()
    }

    fn render_bars(&self, labels: &[String], values: &[f64], y_max: f64) -> String {
        let (px, py, pw, ph) = self.plot_area();
        let bar_count = values.len();
        let bar_width = pw / bar_count as f64 * 0.8;
        let gap = pw / bar_count as f64 * 0.2;

        let mut svg = String::new();

        for (i, (label, &value)) in labels.iter().zip(values.iter()).enumerate() {
            let x = px + (i as f64 * (bar_width + gap)) + gap / 2.0;
            let height = (value / y_max) * ph;
            let y = py + ph - height;
            let color = self.chart.style.series_color(i);

            svg.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
                x,
                y,
                bar_width,
                height,
                color.to_hex()
            ));

            // Label
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" fill="{}">{}</text>"#,
                x + bar_width / 2.0,
                py + ph + 20.0,
                self.chart.style.font_family,
                self.chart.style.axis_font_size,
                self.chart.style.text_color.to_hex(),
                html_escape(label)
            ));
        }

        svg
    }

    fn render_pie_slices(&self, labels: &[String], values: &[f64]) -> String {
        let cx = self.chart.width / 2.0;
        let cy = self.chart.height / 2.0;
        let radius = f64::min(cx, cy) - self.chart.style.padding.top;
        let total: f64 = values.iter().sum();

        let mut svg = String::new();
        let mut start_angle = -std::f64::consts::FRAC_PI_2; // Start at top

        for (i, (&value, label)) in values.iter().zip(labels.iter()).enumerate() {
            let angle = (value / total) * 2.0 * std::f64::consts::PI;
            let end_angle = start_angle + angle;
            let color = self.chart.style.series_color(i);

            let x1 = cx + radius * start_angle.cos();
            let y1 = cy + radius * start_angle.sin();
            let x2 = cx + radius * end_angle.cos();
            let y2 = cy + radius * end_angle.sin();

            let large_arc = if angle > std::f64::consts::PI { 1 } else { 0 };

            svg.push_str(&format!(
                r#"<path d="M{cx},{cy} L{x1},{y1} A{radius},{radius} 0 {large_arc},1 {x2},{y2} Z" fill="{}"/>"#,
                color.to_hex()
            ));

            // Label
            let label_angle = start_angle + angle / 2.0;
            let label_radius = radius * 0.7;
            let lx = cx + label_radius * label_angle.cos();
            let ly = cy + label_radius * label_angle.sin();

            svg.push_str(&format!(
                r#"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" fill="white">{}</text>"#,
                lx, ly,
                self.chart.style.font_family,
                self.chart.style.axis_font_size,
                html_escape(label)
            ));

            start_angle = end_angle;
        }

        svg
    }

    fn extract_xy(&self, table: &Table) -> Result<(Vec<f64>, Vec<f64>)> {
        let x_col = self.chart.x_column.as_deref().unwrap_or("x");
        let y_col = self.chart.y_column.as_deref().unwrap_or("y");

        // Try to get columns by name, or use first two columns
        let x_data = table
            .column_as_f64(x_col)
            .or_else(|| table.headers.first().and_then(|h| table.column_as_f64(h)))
            .ok_or_else(|| Error::MissingColumn {
                column: x_col.to_string(),
            })?;

        let y_data = table
            .column_as_f64(y_col)
            .or_else(|| table.headers.get(1).and_then(|h| table.column_as_f64(h)))
            .ok_or_else(|| Error::MissingColumn {
                column: y_col.to_string(),
            })?;

        Ok((x_data, y_data))
    }

    fn extract_labels_values(&self, table: &Table) -> Result<(Vec<String>, Vec<f64>)> {
        if table.headers.len() < 2 {
            return Err(Error::InvalidData {
                message: "need at least 2 columns".to_string(),
            });
        }

        let labels = table.column_as_str(&table.headers[0]).unwrap_or_default();
        let values =
            table
                .column_as_f64(&table.headers[1])
                .ok_or_else(|| Error::MissingColumn {
                    column: table.headers[1].clone(),
                })?;

        Ok((labels, values))
    }

    fn calculate_bounds(&self, x_data: &[f64], y_data: &[f64]) -> (f64, f64, f64, f64) {
        let x_min = x_data.iter().cloned().fold(f64::INFINITY, f64::min);
        let x_max = x_data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let y_min = y_data.iter().cloned().fold(f64::INFINITY, f64::min);
        let y_max = y_data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Add some padding to bounds
        let x_padding = (x_max - x_min) * 0.05;
        let y_padding = (y_max - y_min) * 0.05;

        (
            x_min - x_padding,
            x_max + x_padding,
            (y_min - y_padding).min(0.0),
            y_max + y_padding,
        )
    }
}

/// Escape HTML entities.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::Chart;

    #[test]
    fn render_line_chart() {
        let data =
            DataSource::from_points(vec![(1.0, 10.0), (2.0, 20.0), (3.0, 15.0), (4.0, 25.0)]);

        let chart = Chart::new(ChartType::Line)
            .with_title("Test Line Chart")
            .with_data(data);

        let svg = chart.render().unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("polyline"));
        assert!(svg.contains("Test Line Chart"));
    }

    #[test]
    fn render_bar_chart() {
        let data = DataSource::from_csv_string("label,value\nA,10\nB,20\nC,15").unwrap();

        let chart = Chart::new(ChartType::Bar)
            .with_title("Test Bar Chart")
            .with_data(data);

        let svg = chart.render().unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("rect"));
    }

    #[test]
    fn render_scatter_plot() {
        let data = DataSource::from_points(vec![(1.0, 1.0), (2.0, 4.0), (3.0, 9.0)]);

        let chart = Chart::new(ChartType::Scatter).with_data(data);

        let svg = chart.render().unwrap();
        assert!(svg.contains("circle"));
    }

    #[test]
    fn html_escape_works() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }
}
