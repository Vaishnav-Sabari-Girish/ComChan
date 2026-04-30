use plotters::prelude::*;
use std::collections::HashMap;
use std::error::Error;

pub fn export_to_svg(
    data: &HashMap<String, Vec<(f64, f64)>>,
    filename: &str,
    sensor_order: &[String],
    plot_title: &String,
) -> Result<(), Box<dyn Error>> {
    if data.is_empty() {
        return Err("No data to export".into());
    }

    let root = SVGBackend::new(filename, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

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

    let mut chart = ChartBuilder::on(&root)
        .caption(plot_title, ("sans-serif", 40).into_font().color(&BLACK))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    chart
        .configure_mesh()
        .label_style(("sans-serif", 18).into_font().color(&BLACK))
        .axis_style(BLACK)
        .draw()?;

    // Colors
    let colors = [&BLUE, &RED, &GREEN, &MAGENTA, &CYAN, &BLACK];

    for (i, name) in sensor_order.iter().enumerate() {
        if let Some(series_data) = data.get(name) {
            let color = colors[i % colors.len()];

            chart
                .draw_series(LineSeries::new(series_data.iter().copied(), color))?
                .label(name)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
        }
    }

    // Legend Text to BLACK
    chart
        .configure_series_labels()
        .label_font(("sans-serif", 20).into_font().color(&BLACK))
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}
