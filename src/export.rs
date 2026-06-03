use crate::serial::get_timestamp;
use plotters::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

// A soft dark grey looks better than blindingly pure black for backgrounds
const DARK_BG: RGBColor = RGBColor(30, 30, 30);
const DARK_GRID: RGBColor = RGBColor(80, 80, 80);

pub fn export_to_svg(
    data: &HashMap<String, Vec<(f64, f64)>>,
    filename: &str,
    sensor_order: &[String],
    plot_title: &str,
    is_dark_mode: bool, // The new toggle!
) -> Result<(), Box<dyn Error>> {
    if data.is_empty() {
        return Err("No data to export".into());
    }

    let root = SVGBackend::new(filename, (1280, 720)).into_drawing_area();

    // 1. Set Background Color
    if is_dark_mode {
        root.fill(&DARK_BG)?;
    } else {
        root.fill(&WHITE)?;
    }

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for series in data.values() {
        for &(x, y) in series {
            if x < min_x {
                min_x = x;
            }
            if x > max_x {
                max_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if y > max_y {
                max_y = y;
            }
        }
    }

    // Padding in Y-axis
    let y_padding = (max_y - min_y).abs() * 0.1;
    let min_y = min_y - y_padding;
    let max_y = max_y + y_padding;

    let x_range = max_x - min_x;
    let (min_x, max_x) = if x_range == 0.0 {
        (min_x - 1.0, max_x + 1.0)
    } else {
        (min_x, max_x)
    };

    // 2. Dynamically assign Text and Grid styles based on mode
    let text_color = if is_dark_mode { &WHITE } else { &BLACK };
    let grid_color = if is_dark_mode { &DARK_GRID } else { &BLACK };
    let legend_bg = if is_dark_mode {
        DARK_BG.mix(0.8)
    } else {
        WHITE.mix(0.8)
    };

    let mut chart = ChartBuilder::on(&root)
        .caption(plot_title, ("sans-serif", 40).into_font().color(text_color))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    chart
        .configure_mesh()
        .label_style(("sans-serif", 18).into_font().color(text_color))
        .axis_style(text_color)
        .bold_line_style(grid_color)
        .light_line_style(grid_color.mix(0.5))
        .draw()?;

    // 3. Setup Trace Colors (Swap dark blue for bright cyan in dark mode for visibility)
    // 3. Setup Trace Colors (Swap dark blue for bright cyan in dark mode for visibility)
    let trace_colors: Vec<RGBColor> = if is_dark_mode {
        vec![CYAN, RED, GREEN, MAGENTA, YELLOW, WHITE]
    } else {
        vec![BLUE, RED, GREEN, MAGENTA, CYAN, BLACK]
    };

    for (i, name) in sensor_order.iter().enumerate() {
        if let Some(series_data) = data.get(name) {
            let color = trace_colors[i % trace_colors.len()];

            chart
                .draw_series(LineSeries::new(series_data.iter().copied(), color))?
                .label(name)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
        }
    }

    // 4. Style the Legend
    chart
        .configure_series_labels()
        .label_font(("sans-serif", 20).into_font().color(text_color))
        .background_style(legend_bg)
        .border_style(text_color)
        .draw()?;

    root.present()?;
    Ok(())
}

pub struct CsvStreamer {
    writer: BufWriter<std::fs::File>,
    headers: Vec<String>,
    header_written: bool,
}

impl CsvStreamer {
    pub fn new(filename: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)?;

        Ok(Self {
            writer: BufWriter::new(file),
            headers: Vec::new(),
            header_written: false,
        })
    }

    pub fn write_row(&mut self, parsed_data: &[(String, f64)]) -> std::io::Result<()> {
        if parsed_data.is_empty() {
            return Ok(());
        }

        if !self.header_written {
            self.headers = parsed_data
                .iter()
                .map(|(name, _)| name.to_string())
                .collect();

            write!(self.writer, "Timestamp")?;

            for header in &self.headers {
                write!(self.writer, ",{}", header)?;
            }
            writeln!(self.writer)?;
            self.header_written = true;
        }

        write!(self.writer, "{}", get_timestamp())?;

        for header in &self.headers {
            if let Some((_, value)) = parsed_data.iter().find(|(name, _)| name == header) {
                write!(self.writer, ",{}", value)?;
            } else {
                write!(self.writer, ",")?;
            }
        }

        writeln!(self.writer)?;
        self.writer.flush()?;

        Ok(())
    }
}
