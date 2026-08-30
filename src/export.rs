use crate::config::ChartType;
use crate::serial::get_timestamp;
use plotters::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

// A soft dark grey looks better than blindingly pure black for backgrounds
const DARK_BG: RGBColor = RGBColor(30, 30, 30);
const DARK_GRID: RGBColor = RGBColor(80, 80, 80);
const HIST_BINS: usize = 24;

pub fn export_to_svg(
    data: &HashMap<String, Vec<(f64, f64)>>,
    filename: &str,
    sensor_order: &[String],
    plot_title: &str,
    is_dark_mode: bool,
    chart_type: ChartType,
) -> Result<(), Box<dyn Error>> {
    if data.is_empty() {
        return Err("No data to export".into());
    }

    let hist_data: HashMap<String, Vec<(f64, f64)>> = if matches!(chart_type, ChartType::Hist) {
        build_histogram_data(data, sensor_order)
    } else {
        HashMap::new()
    };

    let plot_data: &HashMap<String, Vec<(f64, f64)>> = if matches!(chart_type, ChartType::Hist) {
        &hist_data
    } else {
        data
    };

    if plot_data.is_empty() || plot_data.values().all(|s| s.is_empty()) {
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

    for series in plot_data.values() {
        for &(x, y) in series {
            if !x.is_finite() || !y.is_finite() {
                continue;
            }

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

    if !min_x.is_finite() || !max_x.is_finite() || !min_y.is_finite() || !max_y.is_finite() {
        return Err("No finite data points to export".into());
    }

    if matches!(chart_type, ChartType::Bar | ChartType::Hist) {
        min_y = min_y.min(0.0);
        max_y = max_y.max(0.0);
    }

    // Padding in Y-axis
    let y_padding = (max_y - min_y).abs() * 0.1;
    let min_y = if matches!(chart_type, ChartType::Bar | ChartType::Hist) && min_y >= 0.0 {
        0.0
    } else {
        min_y - y_padding
    };
    let max_y = max_y + y_padding;

    let x_range = max_x - min_x;
    let (min_x, max_x) = if x_range == 0.0 {
        (min_x - 1.0, max_x + 1.0)
    } else {
        (min_x, max_x)
    };

    let bar_half = {
        let span = (max_x - min_x).abs().max(1.0);
        let n = plot_data
            .values()
            .map(|s| s.len())
            .max()
            .unwrap_or(1)
            .max(1);
        (span / (n as f64 * 2.5)).max(span * 0.002)
    };

    let (min_x, max_x) = if matches!(chart_type, ChartType::Bar | ChartType::Hist) {
        (min_x - bar_half, max_x + bar_half)
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

    let caption = format!("{} [{}]", plot_title, chart_type);

    let mut chart = ChartBuilder::on(&root)
        .caption(caption, ("sans-serif", 40).into_font().color(text_color))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    chart
        .configure_mesh()
        .x_desc(if matches!(chart_type, ChartType::Hist) {
            "Value"
        } else {
            "Sample"
        })
        .y_desc(if matches!(chart_type, ChartType::Hist) {
            "Count"
        } else {
            "Value"
        })
        .label_style(("sans-serif", 15).into_font().color(text_color))
        .axis_style(text_color)
        .bold_line_style(grid_color)
        .light_line_style(grid_color.mix(0.5))
        .draw()?;

    // 3. Setup Trace Colors (Swap dark blue for bright cyan in dark mode for visibility)
    let trace_colors: Vec<RGBColor> = if is_dark_mode {
        vec![CYAN, RED, GREEN, MAGENTA, YELLOW, WHITE]
    } else {
        vec![BLUE, RED, GREEN, MAGENTA, CYAN, BLACK]
    };

    for (i, name) in sensor_order.iter().enumerate() {
        if let Some(series_data) = plot_data.get(name) {
            let color = trace_colors[i % trace_colors.len()];

            match chart_type {
                ChartType::Line => {
                    chart
                        .draw_series(LineSeries::new(series_data.iter().copied(), color))?
                        .label(name)
                        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
                }
                ChartType::Scatter => {
                    chart
                        .draw_series(PointSeries::of_element(
                            series_data.iter().copied(),
                            3,
                            color,
                            &|c, s, st| EmptyElement::at(c) + Circle::new((0, 0), s, st.filled()),
                        ))?
                        .label(name)
                        .legend(move |(x, y)| Circle::new((x + 10, y), 4, color.filled()));
                }
                ChartType::Bar | ChartType::Hist => {
                    chart
                        .draw_series(series_data.iter().map(|&(x, y)| {
                            Rectangle::new([(x - bar_half, 0.0), (x + bar_half, y)], color.filled())
                        }))?
                        .label(name)
                        .legend(move |(x, y)| {
                            Rectangle::new([(x, y - 5), (x + 20, y + 5)], color.filled())
                        });
                }
            }
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

fn build_histogram_data(
    data: &HashMap<String, Vec<(f64, f64)>>,
    sensor_order: &[String],
) -> HashMap<String, Vec<(f64, f64)>> {
    let mut ymin = f64::INFINITY;
    let mut ymax = f64::NEG_INFINITY;
    for series in data.values() {
        for &(_, y) in series {
            if y < ymin {
                ymin = y;
            }
            if y > ymax {
                ymax = y;
            }
        }
    }

    if !ymin.is_finite() || !ymax.is_finite() {
        return HashMap::new();
    }

    if (ymax - ymin).abs() < f64::EPSILON {
        ymin -= 1.0;
        ymax += 1.0;
    }

    let range = (ymax - ymin).max(f64::EPSILON);
    let bin_width = range / HIST_BINS as f64;

    let mut out = HashMap::new();

    for name in sensor_order {
        let Some(series) = data.get(name) else {
            continue;
        };

        if series.is_empty() {
            continue;
        }

        let mut counts = [0u32; HIST_BINS];
        for &(_, y) in series {
            let idx = ((y - ymin) / bin_width).floor() as isize;
            let idx = idx.clamp(0, (HIST_BINS as isize) - 1) as usize;
            counts[idx] += 1;
        }

        let points: Vec<(f64, f64)> = counts
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                let center = ymin + (i as f64 + 0.5) * bin_width;
                (center, c as f64)
            })
            .collect();

        out.insert(name.clone(), points);
    }

    out
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

        let already_has_data = file.metadata()?.len() > 0;

        Ok(Self {
            writer: BufWriter::new(file),
            headers: Vec::new(),
            header_written: already_has_data,
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
