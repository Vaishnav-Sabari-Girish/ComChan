use crate::config::MergedConfig;
use crate::parser::{SensorData, get_color_for_index, parse_sensor_data};
use crate::serial::{
    get_timestamp, parse_data_bits, parse_flow_control, parse_parity, parse_stop_bits,
};
use crossterm::{
    event::{self, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use inline_colorization::*;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::*,
};
use serialport;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Read, Write};
use std::time::{Duration, Instant};

// ── Plotter state ─────────────────────────────────────────────────────────────

struct PlotterState {
    sensors: HashMap<String, SensorData>,
    /// Ordered list of sensor names (insertion order)
    sensor_order: Vec<String>,
    x: f64,
    global_y_min: f64,
    global_y_max: f64,
    paused: bool,
    /// Total samples received (never paused)
    total_samples: u64,
    samples_this_second: u32,
    sample_rate: u32,
    last_rate_update: Instant,
    start_time: Instant,
    lines_discarded: usize,
    /// Buffer for incomplete serial lines
    receive_buf: String,
    /// Errors reported to the status bar
    last_error: Option<String>,
}

const DISCARD_FIRST_LINES: usize = 3;

impl PlotterState {
    fn new() -> Self {
        let now = Instant::now();
        PlotterState {
            sensors: HashMap::new(),
            sensor_order: Vec::new(),
            x: 0.0,
            global_y_min: f64::INFINITY,
            global_y_max: f64::NEG_INFINITY,
            paused: false,
            total_samples: 0,
            samples_this_second: 0,
            sample_rate: 0,
            last_rate_update: now,
            start_time: now,
            lines_discarded: 0,
            receive_buf: String::new(),
            last_error: None,
        }
    }

    fn get_or_create_sensor(&mut self, name: &str) -> &mut SensorData {
        if !self.sensors.contains_key(name) {
            let color = get_color_for_index(self.sensor_order.len());
            self.sensors
                .insert(name.to_string(), SensorData::new(name.to_string(), color));
            self.sensor_order.push(name.to_string());
        }
        self.sensors.get_mut(name).unwrap()
    }

    fn ingest_line(&mut self, line: &str, max_points: usize) {
        let clean = line.trim();

        if self.lines_discarded < DISCARD_FIRST_LINES {
            self.lines_discarded += 1;
            return;
        }

        if self.paused {
            return;
        }

        let readings = parse_sensor_data(clean);
        if readings.is_empty() {
            return;
        }

        for (name, value) in readings {
            let sensor = self.get_or_create_sensor(name.as_ref());
            sensor.add_point(self.x, value, max_points);

            if value < self.global_y_min {
                self.global_y_min = value;
            }
            if value > self.global_y_max {
                self.global_y_max = value;
            }
        }

        self.x += 1.0;
        self.total_samples += 1;
        self.samples_this_second += 1;

        // Update sample rate every second
        let elapsed = self.last_rate_update.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.sample_rate = self.samples_this_second;
            self.samples_this_second = 0;
            self.last_rate_update = Instant::now();
        }
    }

    fn x_bounds(&self) -> [f64; 2] {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;

        for sensor in self.sensors.values() {
            if let (Some(first), Some(last)) = (sensor.data.first(), sensor.data.last()) {
                if first.0 < min_x {
                    min_x = first.0;
                }
                if last.0 > max_x {
                    max_x = last.0;
                }
            }
        }

        if min_x.is_finite() && max_x.is_finite() {
            [min_x, max_x]
        } else {
            [0.0, 10.0]
        }
    }

    fn y_bounds(&self) -> [f64; 2] {
        if self.global_y_min.is_finite() && self.global_y_max.is_finite() {
            if self.global_y_min == self.global_y_max {
                [self.global_y_min - 1.0, self.global_y_max + 1.0]
            } else {
                let padding = (self.global_y_max - self.global_y_min) * 0.1;
                [self.global_y_min - padding, self.global_y_max + padding]
            }
        } else {
            [-1.0, 1.0]
        }
    }

    fn uptime_str(&self) -> String {
        let secs = self.start_time.elapsed().as_secs();
        format!(
            "{:02}:{:02}:{:02}",
            secs / 3600,
            (secs % 3600) / 60,
            secs % 60
        )
    }
}

// ── Main entry point ──────────────────────────────────────────────────────────

pub fn run_plotter_mode(
    config: MergedConfig,
    port_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let data_bits = parse_data_bits(config.data_bits)?;
    let stop_bits = parse_stop_bits(config.stop_bits)?;
    let parity = parse_parity(&config.parity)?;
    let flow_control = parse_flow_control(&config.flow_control)?;

    let mut port = serialport::new(&port_name, config.baud)
        .timeout(Duration::from_millis(config.timeout_ms))
        .data_bits(data_bits)
        .stop_bits(stop_bits)
        .parity(parity)
        .flow_control(flow_control)
        .open()?;

    let mut log_writer: Option<BufWriter<std::fs::File>> =
        if let Some(ref log_path) = config.log_file {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)?;
            Some(BufWriter::new(file))
        } else {
            None
        };

    std::thread::sleep(Duration::from_millis(config.reset_delay_ms));

    let mut state = PlotterState::new();
    let mut serial_buf = [0u8; 1024];

    loop {
        // ── Input handling ────────────────────────────────────────────────────
        if event::poll(Duration::from_millis(5))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,

                    // Space: pause / resume
                    KeyCode::Char(' ') => {
                        state.paused = !state.paused;
                    }

                    // 'c': clear all data
                    KeyCode::Char('c') => {
                        state.sensors.clear();
                        state.sensor_order.clear();
                        state.x = 0.0;
                        state.global_y_min = f64::INFINITY;
                        state.global_y_max = f64::NEG_INFINITY;
                        state.total_samples = 0;
                    }

                    _ => {}
                }
            }
        }

        // ── Serial read ───────────────────────────────────────────────────────
        match port.read(&mut serial_buf) {
            Ok(n) if n > 0 => {
                let chunk = String::from_utf8_lossy(&serial_buf[..n]);
                state.receive_buf.push_str(&chunk);

                while let Some(pos) = state.receive_buf.find('\n') {
                    let line = state.receive_buf.drain(..=pos).collect::<String>();

                    if let Some(ref mut writer) = log_writer {
                        let _ = writeln!(writer, "RX [{}]: {}", get_timestamp(), line.trim_end());
                        let _ = writer.flush();
                    }

                    state.ingest_line(&line, config.plot_points);
                }
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
            Err(e) => {
                state.last_error = Some(format!("Read error: {}", e));
            }
        }

        // ── Render ────────────────────────────────────────────────────────────
        let x_bounds = state.x_bounds();
        let y_bounds = state.y_bounds();

        // Pre-compute labels before the closure borrows state
        let x_labels = [
            format!("{:.0}", x_bounds[0]),
            format!("{:.0}", (x_bounds[0] + x_bounds[1]) / 2.0),
            format!("{:.0}", x_bounds[1]),
        ];
        let y_labels = [
            format!("{:.2}", y_bounds[0]),
            format!("{:.2}", (y_bounds[0] + y_bounds[1]) / 2.0),
            format!("{:.2}", y_bounds[1]),
        ];

        let pause_indicator = if state.paused { " ⏸ PAUSED" } else { "" };
        let uptime = state.uptime_str();
        let sample_rate = state.sample_rate;
        let total_samples = state.total_samples;
        let sensor_count = state.sensors.len();
        let last_error = state.last_error.clone();
        let baud = config.baud;
        let port_name_disp = port_name.clone();

        // Build sidebar rows before the draw closure
        let sidebar_rows: Vec<(String, Color, f64, f64, f64)> = state
            .sensor_order
            .iter()
            .filter_map(|name| {
                state.sensors.get(name).map(|s| {
                    (
                        s.name.clone(),
                        s.color,
                        s.current_value,
                        s.min_value,
                        s.max_value,
                    )
                })
            })
            .collect();

        // Build datasets
        let datasets: Vec<Dataset> = state
            .sensor_order
            .iter()
            .filter_map(|name| state.sensors.get(name))
            .filter(|s| s.has_data())
            .map(|sensor| {
                Dataset::default()
                    .name(format!("{} ({:.2})", sensor.name, sensor.current_value))
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(sensor.color))
                    .data(&sensor.data)
            })
            .collect();

        terminal.draw(|f| {
            // ── Layout ────────────────────────────────────────────────────────
            //
            //  ┌───────────────── chart ──────────────┬── sidebar ──┐
            //  │                                       │             │
            //  │                                       │  sensor     │
            //  │                                       │  stats      │
            //  │                                       │             │
            //  └───────────────────────────────────────┴─────────────┘
            //  ┌─────────────────── status bar ───────────────────────┐

            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(f.area());

            let main_row = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(10), Constraint::Length(28)])
                .split(outer[0]);

            // ── Chart ─────────────────────────────────────────────────────────
            let chart_title = format!(
                " 󰕾 ComChan Plotter  {}  {}  {} sensors{}",
                port_name_disp, baud, sensor_count, pause_indicator
            );

            let chart = Chart::new(datasets)
                .block(
                    Block::default()
                        .title(Span::styled(
                            chart_title,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                )
                .x_axis(
                    Axis::default()
                        .title(Span::styled("Sample", Style::default().fg(Color::Gray)))
                        .style(Style::default().fg(Color::DarkGray))
                        .bounds(x_bounds)
                        .labels(vec![
                            x_labels[0].as_str(),
                            x_labels[1].as_str(),
                            x_labels[2].as_str(),
                        ]),
                )
                .y_axis(
                    Axis::default()
                        .title(Span::styled("Value", Style::default().fg(Color::Gray)))
                        .style(Style::default().fg(Color::DarkGray))
                        .bounds(y_bounds)
                        .labels(vec![
                            y_labels[0].as_str(),
                            y_labels[1].as_str(),
                            y_labels[2].as_str(),
                        ]),
                )
                .legend_position(Some(LegendPosition::TopLeft));

            f.render_widget(chart, main_row[0]);

            // ── Sidebar (sensor stats) ────────────────────────────────────────
            let sidebar_block = Block::default()
                .title(Span::styled(
                    " Sensors ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));

            let inner = sidebar_block.inner(main_row[1]);
            f.render_widget(sidebar_block, main_row[1]);

            if sidebar_rows.is_empty() {
                let waiting = Paragraph::new("Waiting for\ndata…")
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center);
                f.render_widget(waiting, inner);
            } else {
                // Each sensor gets 4 lines: name + cur/min/max + separator
                let mut lines: Vec<Line> = Vec::new();
                for (name, color, cur, min, max) in &sidebar_rows {
                    lines.push(Line::from(vec![Span::styled(
                        format!("● {}", name),
                        Style::default().fg(*color).add_modifier(Modifier::BOLD),
                    )]));
                    lines.push(Line::from(vec![Span::styled(
                        format!("  now: {:.3}", cur),
                        Style::default().fg(Color::White),
                    )]));
                    lines.push(Line::from(vec![Span::styled(
                        format!("  min: {:.3}", min),
                        Style::default().fg(Color::Blue),
                    )]));
                    lines.push(Line::from(vec![Span::styled(
                        format!("  max: {:.3}", max),
                        Style::default().fg(Color::Red),
                    )]));
                    lines.push(Line::from(vec![Span::styled(
                        "  ─────────────",
                        Style::default().fg(Color::DarkGray),
                    )]));
                }
                let para = Paragraph::new(lines).wrap(Wrap { trim: false });
                f.render_widget(para, inner);
            }

            // ── Status bar ────────────────────────────────────────────────────
            let status_bg = if state.paused {
                Color::DarkGray
            } else {
                Color::Reset
            };

            let error_span = if let Some(ref err) = last_error {
                Span::styled(format!(" ⚠ {} ", err), Style::default().fg(Color::Red))
            } else {
                Span::raw("")
            };

            let status_line = Line::from(vec![
                Span::styled(format!(" ⏱ {}", uptime), Style::default().fg(Color::Green)),
                Span::raw("  "),
                Span::styled(
                    format!("󰩙 {} sps", sample_rate),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("󰆼 {} total", total_samples),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  "),
                error_span,
                Span::styled(
                    "  [Space] pause  [c] clear  [q/Esc] quit",
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            let status_bar = Paragraph::new(status_line)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                )
                .style(Style::default().bg(status_bg));

            f.render_widget(status_bar, outer[1]);
        })?;
    }

    // ── Cleanup ───────────────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if !state.sensors.is_empty() {
        println!("\n{color_green}󰄨 Plotting Summary:{color_reset}");
        for name in &state.sensor_order {
            if let Some(s) = state.sensors.get(name) {
                println!(
                    "   {}: {} pts  min={:.3}  max={:.3}  last={:.3}",
                    s.name,
                    s.data.len(),
                    s.min_value,
                    s.max_value,
                    s.current_value,
                );
            }
        }
        println!(
            "   Total samples: {}  Uptime: {}",
            state.total_samples,
            state.uptime_str()
        );
    }

    Ok(())
}
