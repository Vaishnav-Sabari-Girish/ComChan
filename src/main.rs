use clap::Parser;
use inline_colorization::*;
use serde::{Deserialize, Serialize};
use serialport::{self, DataBits, FlowControl, Parity, StopBits};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::borrow::Cow;

// Plotting imports
use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, prelude::*, symbols, widgets::*};

// Add the port finder module
mod port_finder;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    port: Option<String>,
    baud: Option<u32>,
    data_bits: Option<u8>,
    stop_bits: Option<u8>,
    parity: Option<String>,
    flow_control: Option<String>,
    timeout_ms: Option<u64>,
    reset_delay_ms: Option<u64>,
    log_file: Option<String>,
    verbose: Option<bool>,
    plot: Option<bool>,
    plot_points: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: Some("auto".to_string()),
            baud: Some(9600),
            data_bits: Some(8),
            stop_bits: Some(1),
            parity: Some("none".to_string()),
            flow_control: Some("none".to_string()),
            timeout_ms: Some(500),
            reset_delay_ms: Some(1000),
            log_file: None,
            verbose: Some(false),
            plot: Some(false),
            plot_points: Some(100),
        }
    }
}

#[derive(Parser)]
#[command(
    name = "comchan",
    version = "0.2.5",
    author = "Vaishnav-Sabari-Girish",
    about = "Blazingly Fast Minimal Serial Monitor with Plotting"
)]
struct Args {
    #[arg(short = 'p', long = "port", help = "Serial port to connect to")]
    port: Option<String>,

    #[arg(short = 'r', long = "baud", help = "Baud Rate of the Serial Monitor")]
    baud: Option<u32>,

    #[arg(short = 'd', long = "data-bits")]
    data_bits: Option<u8>,

    #[arg(short = 's', long = "stop-bits")]
    stop_bits: Option<u8>,

    #[arg(long = "parity")]
    parity: Option<String>,

    #[arg(long = "flow-control")]
    flow_control: Option<String>,

    #[arg(short = 't', long = "timeout")]
    timeout_ms: Option<u64>,

    #[arg(long = "reset-delay")]
    reset_delay_ms: Option<u64>,

    #[arg(short = 'l', long = "log", help = "Log Serial data into a file")]
    log_file: Option<String>,

    #[arg(long = "list-ports", action = clap::ArgAction::SetTrue, help = "List all available ports")]
    list_ports: bool,

    #[arg(long = "auto", action = clap::ArgAction::SetTrue, help = "Auto-detect USB serial port")]
    auto: Option<bool>,

    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::SetTrue, help = "Verbose mode of Serial monitor")]
    verbose: Option<bool>,

    #[arg(long = "plot", action = clap::ArgAction::SetTrue, help = "Go to Serial Plotter instead of Serial monitor")]
    plot: bool,

    #[arg(long = "plot-points")]
    plot_points: Option<usize>,

    #[arg(
        long = "config",
        short = 'c',
        help = "Path to config file (default: platform-specific config directory)"
    )]
    config_file: Option<PathBuf>,

    #[arg(long = "generate-config", action = clap::ArgAction::SetTrue, help = "Generate a default config file")]
    generate_config: bool,
}

struct MergedConfig {
    port: Option<String>,
    baud: u32,
    data_bits: u8,
    stop_bits: u8,
    parity: String,
    flow_control: String,
    timeout_ms: u64,
    reset_delay_ms: u64,
    log_file: Option<String>,
    list_ports: bool,
    verbose: bool,
    plot: bool,
    plot_points: usize,
}

// Structure to hold multiple sensor data streams
#[derive(Debug, Clone)]
struct SensorData {
    name: String,
    data: Vec<(f64, f64)>,
    color: Color,
    min_value: f64,
    max_value: f64,
}

impl SensorData {
    fn new(name: String, color: Color) -> Self {
        SensorData {
            name,
            data: Vec::new(),
            color,
            min_value: f64::INFINITY,
            max_value: f64::NEG_INFINITY,
        }
    }

    fn add_point(&mut self, x: f64, y: f64, max_points: usize) {
        self.data.push((x, y));

        // Update min/max
        if y < self.min_value {
            self.min_value = y;
        }
        if y > self.max_value {
            self.max_value = y;
        }

        // Maintain rolling window
        if self.data.len() > max_points {
            self.data.remove(0);
        }
    }
}

// Enhanced parsing function that handles multiple sensor formats
fn parse_sensor_data<'a>(line: &'a str) -> Vec<(Cow<'a, str>, f64)> {
    let mut results = Vec::new();
    let line = line.trim();

    // Pattern 1: "SensorName : Value" or "SensorName: Value"
    if line.contains(':') {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() == 2 {
            let sensor_name = parts[0].trim();
            let value_str = parts[1].trim();

            if let Ok(value) = value_str.parse::<f64>() {
                results.push((Cow::Borrowed(sensor_name), value));
                return results;
            }
        }
    }

    // Pattern 2: "SensorName = Value" or "SensorName=Value"
    if line.contains('=') {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 {
            let sensor_name = parts[0].trim();
            let value_str = parts[1].trim();

            if let Ok(value) = value_str.parse::<f64>() {
                results.push((Cow::Borrowed(sensor_name), value));
                return results;
            }
        }
    }

    // Pattern 3: Comma-separated values "value1,value2,value3"
    // These will be named as "Channel 0", "Channel 1", etc.
    if line.contains(',') {
        let values: Vec<&str> = line.split(',').collect();
        for (i, value_str) in values.iter().enumerate() {
            if let Ok(value) = value_str.trim().parse::<f64>() {
                results.push((Cow::Owned(format!("Channel {}", i)), value));
            }
        }
        if !results.is_empty() {
            return results;
        }
    }

    // Pattern 4: Space-separated values "value1 value2 value3"
    let words: Vec<&str> = line.split_whitespace().collect();
    let mut numeric_values = Vec::new();

    for word in &words {
        if let Ok(value) = word.parse::<f64>() {
            numeric_values.push(value);
        }
    }

    // If we found multiple numeric values, treat them as channels
    if numeric_values.len() > 1 {
        for (i, value) in numeric_values.iter().enumerate() {
            results.push((Cow::Owned(format!("Channel {}", i)), *value));
        }
        return results;
    }

    // Pattern 5: Single numeric value (fallback to original behavior)
    if let Ok(value) = line.parse::<f64>() {
        results.push((Cow::Borrowed("Value"), value));
        return results;
    }

    // Pattern 6: Look for numbers in text (like "Temperature: 25.3 C")
    for word in words {
        let cleaned = word.trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-');
        if let Ok(value) = cleaned.parse::<f64>() {
            // Try to find a descriptive name in the line
            let sensor_name = if line.to_lowercase().contains("temp") {
                "Temperature"
            } else if line.to_lowercase().contains("humid") {
                "Humidity"
            } else if line.to_lowercase().contains("pressure") {
                "Pressure"
            } else if line.to_lowercase().contains("mag") {
                "Magnetometer"
            } else if line.to_lowercase().contains("gyro") {
                "Gyroscope"
            } else if line.to_lowercase().contains("accel") {
                "Accelerometer"
            } else {
                "Sensor"
            };

            results.push((Cow::Borrowed(sensor_name), value));
            break; // Only take the first number found in this pattern
        }
    }

    results
}

// Color palette for different sensors
const COLORS: &[Color] = &[
    Color::Cyan,
    Color::Magenta,
    Color::Yellow,
    Color::Green,
    Color::Red,
    Color::Blue,
    Color::White,
    Color::LightCyan,
    Color::LightMagenta,
    Color::LightYellow,
    Color::LightGreen,
    Color::LightRed,
    Color::LightBlue,
];

fn get_color_for_index(index: usize) -> Color {
    COLORS[index % COLORS.len()]
}

fn run_plotter_mode(
    config: MergedConfig,
    port_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal for plotting
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Parse serial port configuration
    let data_bits = parse_data_bits(config.data_bits)?;
    let stop_bits = parse_stop_bits(config.stop_bits)?;
    let parity = parse_parity(&config.parity)?;
    let flow_control = parse_flow_control(&config.flow_control)?;

    // Open serial port
    let mut port = serialport::new(&port_name, config.baud)
        .timeout(Duration::from_millis(config.timeout_ms))
        .data_bits(data_bits)
        .stop_bits(stop_bits)
        .parity(parity)
        .flow_control(flow_control)
        .open()?;

    // Optional log file setup
    let mut log_writer = if let Some(log_path) = &config.log_file {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        Some(BufWriter::new(file))
    } else {
        None
    };

    thread::sleep(Duration::from_millis(config.reset_delay_ms));

    // HashMap to store different sensor streams
    let mut sensors: HashMap<String, SensorData> = HashMap::new();
    let mut x = 0.0;
    let mut buffer = [0u8; 1024];
    let mut received = String::new();
    let mut global_y_min = f64::INFINITY;
    let mut global_y_max = f64::NEG_INFINITY;
    let mut lines_discarded = 0;
    const DISCARD_FIRST_LINES: usize = 3; // Discard first 3 lines to avoid garbled data

    loop {
        // Check for exit condition first
        if event::poll(Duration::from_millis(10))? {
            if let event::Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }

        // Read from serial port
        match port.read(&mut buffer) {
            Ok(n) => {
                if n > 0 {
                    let output = String::from_utf8_lossy(&buffer[..n]);
                    received.push_str(&output);

                    // Process complete lines
                    while let Some(line_end) = received.find('\n') {
                        let line = received.drain(..=line_end).collect::<String>();
                        let clean_line = line.trim();

                        // Skip first few lines to avoid garbled data
                        if lines_discarded < DISCARD_FIRST_LINES {
                            lines_discarded += 1;
                            continue;
                        }

                        // Parse sensor data from the line
                        let sensor_readings = parse_sensor_data(clean_line);
                        let has_data = !sensor_readings.is_empty();

                        // Debug: Print what we're parsing (uncomment for debugging)
                        // if has_data {
                        //     println!("Parsed: {:?} from line: '{}'", sensor_readings, clean_line);
                        // }

                        for (sensor_name, value) in sensor_readings {
                            // Create sensor entry if it doesn't exist
                            if !sensors.contains_key(sensor_name.as_ref()) {
                                let color = get_color_for_index(sensors.len());
                                sensors.insert(
                                    sensor_name.to_string(),
                                    SensorData::new(sensor_name.to_string(), color),
                                );
                            }

                            // Add data point to the appropriate sensor
                            if let Some(sensor) = sensors.get_mut(sensor_name.as_ref()) {
                                sensor.add_point(x, value, config.plot_points);

                                // Update global bounds
                                if value < global_y_min {
                                    global_y_min = value;
                                }
                                if value > global_y_max {
                                    global_y_max = value;
                                }
                            }
                        }

                        // Only increment x if we found some sensor data
                        if has_data {
                            x += 1.0;
                        }

                        // Log to file if enabled
                        if let Some(ref mut writer) = log_writer {
                            writeln!(writer, "RX [{}]: {}", get_timestamp(), clean_line)?;
                            writer.flush()?;
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                // Timeout is normal, continue
            }
            Err(_e) => {
                // Handle error silently in plot mode
            }
        }

        // Calculate dynamic bounds for x-axis
        let (x_min, x_max) = if sensors.is_empty() {
            (0.0, 10.0)
        } else {
            // Find the x-range across all sensors
            let mut min_x = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;

            for sensor in sensors.values() {
                if !sensor.data.is_empty() {
                    let sensor_min = sensor.data[0].0;
                    let sensor_max = sensor.data[sensor.data.len() - 1].0;
                    if sensor_min < min_x {
                        min_x = sensor_min;
                    }
                    if sensor_max > max_x {
                        max_x = sensor_max;
                    }
                }
            }

            if min_x.is_finite() && max_x.is_finite() {
                (min_x, max_x)
            } else {
                (0.0, 10.0)
            }
        };

        // Calculate y-axis bounds with some padding
        let (chart_y_min, chart_y_max) =
            if global_y_min.is_finite() && global_y_max.is_finite() && global_y_min != global_y_max
            {
                let padding = (global_y_max - global_y_min) * 0.1;
                (global_y_min - padding, global_y_max + padding)
            } else if global_y_min.is_finite() && global_y_max.is_finite() {
                (global_y_min - 1.0, global_y_max + 1.0)
            } else {
                (-1.0, 1.0)
            };

        // Create labels outside the closure
        let x_min_label = format!("{:.0}", x_min);
        let x_mid_label = format!("{:.0}", (x_min + x_max) / 2.0);
        let x_max_label = format!("{:.0}", x_max);

        let y_min_label = format!("{:.2}", chart_y_min);
        let y_mid_label = format!("{:.2}", (chart_y_min + chart_y_max) / 2.0);
        let y_max_label = format!("{:.2}", chart_y_max);

        // Create datasets for chart
        let datasets: Vec<Dataset> = sensors
            .values()
            .map(|sensor| {
                Dataset::default()
                    .name(sensor.name.as_str())
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(sensor.color))
                    .data(&sensor.data)
            })
            .collect();

        // Create legend info
        let legend_info = if sensors.len() > 1 {
            format!(" | {} sensors", sensors.len())
        } else {
            String::new()
        };

        // Draw the chart
        terminal.draw(|f| {
            let size = f.area();

            let chart = Chart::new(datasets)
                .block(
                    Block::default()
                        .title(format!(
                            "Live Serial Plotter - {} @ {} baud{} (Press 'q' or ESC to exit)",
                            port_name, config.baud, legend_info
                        ))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::White)),
                )
                .x_axis(
                    Axis::default()
                        .title("Sample")
                        .style(Style::default().fg(Color::Gray))
                        .bounds([x_min, x_max])
                        .labels(vec![
                            x_min_label.as_str(),
                            x_mid_label.as_str(),
                            x_max_label.as_str(),
                        ]),
                )
                .y_axis(
                    Axis::default()
                        .title("Value")
                        .style(Style::default().fg(Color::Gray))
                        .bounds([chart_y_min, chart_y_max])
                        .labels(vec![
                            y_min_label.as_str(),
                            y_mid_label.as_str(),
                            y_max_label.as_str(),
                        ]),
                )
                // FIXED: Removed hidden_legend_constraints to show legends properly
                .legend_position(Some(LegendPosition::TopRight));

            f.render_widget(chart, size);
        })?;
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    // Print summary
    if !sensors.is_empty() {
        println!("\n{color_green}󰄨 Plotting Summary:{color_reset}");
        for (name, sensor) in &sensors {
            println!(
                "   {}: {} data points (min: {:.2}, max: {:.2})",
                name,
                sensor.data.len(),
                sensor.min_value,
                sensor.max_value
            );
        }
    }

    Ok(())
}

// Cross-platform config directory detection
fn get_default_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = if cfg!(target_os = "windows") {
        // Windows: %APPDATA%\comchan\comchan.toml
        if let Some(appdata) = std::env::var_os("APPDATA") {
            PathBuf::from(appdata).join("comchan")
        } else {
            return Err("APPDATA environment variable not found".into());
        }
    } else if cfg!(target_os = "macos") {
        // macOS: ~/Library/Application Support/comchan/comchan.toml
        if let Some(home_dir) = dirs::home_dir() {
            home_dir.join("Library").join("Application Support").join("comchan")
        } else {
            return Err("Could not find home directory".into());
        }
    } else {
        // Linux and other Unix-like systems: ~/.config/comchan/comchan.toml
        if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(".config").join("comchan")
        } else {
            return Err("Could not find home directory".into());
        }
    };

    Ok(config_dir.join("comchan.toml"))
}

fn get_platform_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unix-like"
    }
}

// Updated config file finder with cross-platform support
fn find_config_file(specified_path: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(path) = specified_path {
        if path.exists() {
            return Some(path);
        }
        return None;
    }

    // Check current directory first
    let current_dir_config = PathBuf::from("comchan.toml");
    if current_dir_config.exists() {
        return Some(current_dir_config);
    }

    // Check platform-specific config directories
    if let Ok(default_path) = get_default_config_path() {
        if default_path.exists() {
            return Some(default_path);
        }
    }

    // Fallback: check home directory (for backward compatibility)
    if let Some(home_dir) = dirs::home_dir() {
        let home_config = home_dir.join(".comchan.toml");
        if home_config.exists() {
            return Some(home_config);
        }
    }

    None
}

fn load_config(config_path: Option<PathBuf>) -> Result<Config, Box<dyn std::error::Error>> {
    if let Some(path) = find_config_file(config_path) {
        let content = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file {}: {}", path.display(), e))?;
        println!(
            "{color_blue}󰅍 Loaded config from: {}{color_reset}",
            path.display()
        );
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

fn generate_default_config(path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = if let Some(specified_path) = path {
        specified_path
    } else {
        get_default_config_path()?
    };

    let default_config = Config::default();
    let toml_content = toml::to_string_pretty(&default_config)?;

    let platform_comment = format!(
        "# Platform: {} config directory",
        get_platform_name()
    );

    let location_comment = if cfg!(target_os = "windows") {
        "# Default location: %APPDATA%\\comchan\\comchan.toml"
    } else if cfg!(target_os = "macos") {
        "# Default location: ~/Library/Application Support/comchan/comchan.toml"
    } else {
        "# Default location: ~/.config/comchan/comchan.toml"
    };

    let commented_config = format!(
        r#"# ComChan Configuration File
# 
{}
{}
# 
# This file contains default settings for comchan serial monitor.
# Command line arguments will override these settings.
# 
# To use auto-detection, set port = "auto"
# Available parity options: "none", "odd", "even"
# Available flow control options: "none", "software", "hardware"

{}
"#,
        platform_comment,
        location_comment,
        toml_content
    );

    // Create parent directories if they don't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&config_path, commented_config)?;
    println!(
        "{color_green} Generated default config file for {}: {}{color_reset}",
        get_platform_name(),
        config_path.display()
    );
    println!("{color_blue}󰌵 Edit the file to customize your default settings{color_reset}");

    Ok(())
}

fn merge_config_and_args(config: Config, args: Args) -> MergedConfig {
    MergedConfig {
        port: args.port.or(config.port),
        baud: args.baud.or(config.baud).unwrap_or(9600),
        data_bits: args.data_bits.or(config.data_bits).unwrap_or(8),
        stop_bits: args.stop_bits.or(config.stop_bits).unwrap_or(1),
        parity: args.parity.or(config.parity).unwrap_or("none".to_string()),
        flow_control: args
            .flow_control
            .or(config.flow_control)
            .unwrap_or("none".to_string()),
        timeout_ms: args.timeout_ms.or(config.timeout_ms).unwrap_or(500),
        reset_delay_ms: args
            .reset_delay_ms
            .or(config.reset_delay_ms)
            .unwrap_or(1000),
        log_file: args.log_file.or(config.log_file),
        list_ports: args.list_ports,
        verbose: args.verbose.or(config.verbose).unwrap_or(false),
        plot: args.plot || config.plot.unwrap_or(false),
        plot_points: args.plot_points.or(config.plot_points).unwrap_or(100),
    }
}

fn list_available_ports() -> Result<(), Box<dyn std::error::Error>> {
    println!("{color_cyan}󰅍 All Available Serial Ports:{color_reset}");
    let ports = serialport::available_ports()?;

    if ports.is_empty() {
        println!("  {color_yellow}  No serial ports found{color_reset}");
        return Ok(());
    }

    for port in ports {
        println!(
            "   {} - {}",
            port.port_name,
            match port.port_type {
                serialport::SerialPortType::UsbPort(info) => {
                    format!("USB Device (VID: {:04x}, PID: {:04x})", info.vid, info.pid)
                }
                serialport::SerialPortType::BluetoothPort => "Bluetooth".to_string(),
                serialport::SerialPortType::PciPort => "PCI".to_string(),
                serialport::SerialPortType::Unknown => "Unknown".to_string(),
            }
        );
    }

    println!();
    port_finder::show_usb_ports()?;

    Ok(())
}

fn parse_data_bits(bits: u8) -> Result<DataBits, String> {
    match bits {
        5 => Ok(DataBits::Five),
        6 => Ok(DataBits::Six),
        7 => Ok(DataBits::Seven),
        8 => Ok(DataBits::Eight),
        _ => Err(format!(
            "Invalid data bits: {}. Must be 5, 6, 7, or 8",
            bits
        )),
    }
}

fn parse_stop_bits(bits: u8) -> Result<StopBits, String> {
    match bits {
        1 => Ok(StopBits::One),
        2 => Ok(StopBits::Two),
        _ => Err(format!("Invalid stop bits: {}. Must be 1 or 2", bits)),
    }
}

fn parse_parity(parity: &str) -> Result<Parity, String> {
    match parity.to_lowercase().as_str() {
        "none" | "n" => Ok(Parity::None),
        "odd" | "o" => Ok(Parity::Odd),
        "even" | "e" => Ok(Parity::Even),
        _ => Err(format!(
            "Invalid parity: {}. Must be 'none', 'odd', or 'even'",
            parity
        )),
    }
}

fn parse_flow_control(flow: &str) -> Result<FlowControl, String> {
    match flow.to_lowercase().as_str() {
        "none" | "n" => Ok(FlowControl::None),
        "software" | "s" => Ok(FlowControl::Software),
        "hardware" | "h" => Ok(FlowControl::Hardware),
        _ => Err(format!(
            "Invalid flow control: {}. Must be 'none', 'software', or 'hardware'",
            flow
        )),
    }
}

fn get_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let secs = now / 1000;
    let millis = now % 1000;
    format!("{}.{:03}", secs, millis)
}

fn run_normal_mode(
    config: MergedConfig,
    port_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse serial port configuration
    let data_bits =
        parse_data_bits(config.data_bits).map_err(|e| format!("Configuration error: {}", e))?;
    let stop_bits =
        parse_stop_bits(config.stop_bits).map_err(|e| format!("Configuration error: {}", e))?;
    let parity = parse_parity(&config.parity).map_err(|e| format!("Configuration error: {}", e))?;
    let flow_control = parse_flow_control(&config.flow_control)
        .map_err(|e| format!("Configuration error: {}", e))?;

    let mut port = serialport::new(&port_name, config.baud)
        .timeout(Duration::from_millis(config.timeout_ms))
        .data_bits(data_bits)
        .stop_bits(stop_bits)
        .parity(parity)
        .flow_control(flow_control)
        .open()
        .map_err(|e| format!("Failed to open port {}: {}", port_name, e))?;

    let log_writer = if let Some(log_path) = &config.log_file {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(|e| format!("Failed to open log file {}: {}", log_path, e))?;
        Some(BufWriter::new(file))
    } else {
        None
    };

    thread::sleep(Duration::from_millis(config.reset_delay_ms));

    println!(
        "{color_green} ComChan connected to {} at {} baud{color_reset}",
        port_name, config.baud
    );
    if config.verbose {
        println!(
            "{color_blue}⚙️  Configuration: {} data bits, {} stop bits, {} parity, {} flow control{color_reset}",
            config.data_bits, config.stop_bits, config.parity, config.flow_control
        );
        if let Some(log_path) = &config.log_file {
            println!("{color_blue} Logging to: {}{color_reset}", log_path);
        }
    }
    println!("{color_green} Listening... (Ctrl+C to exit){color_reset}\n");

    let (input_tx, input_rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        let stdin = io::stdin();
        loop {
            let mut input = String::new();
            match stdin.read_line(&mut input) {
                Ok(_) => {
                    if input_tx.send(input).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\n{color_yellow}󰏃 Shutting down ComChan...{color_reset}");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })?;

    let mut buffer = [0u8; 1024];
    let mut received = String::new();
    let mut log_writer = log_writer;

    // Main communication loop
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        // Read from serial port
        match port.read(&mut buffer) {
            Ok(n) => {
                if n > 0 {
                    let output = String::from_utf8_lossy(&buffer[..n]);
                    received.push_str(&output);

                    // Process complete lines
                    while let Some(line_end) = received.find('\n') {
                        let line = received.drain(..=line_end).collect::<String>();

                        // Display received data
                        if config.verbose {
                            print!(" [{}] {}", get_timestamp(), line);
                        } else {
                            print!(" {}", line);
                        }
                        io::stdout().flush()?;

                        // Log to file if enabled
                        if let Some(ref mut writer) = log_writer {
                            writeln!(writer, "RX [{}]: {}", get_timestamp(), line.trim_end())?;
                            writer.flush()?;
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                // Timeout is normal, continue
            }
            Err(e) => {
                eprintln!("{color_red}❌ Serial read error: {e}{color_reset}");
                if let Some(ref mut writer) = log_writer {
                    writeln!(
                        writer,
                        "ERROR [{}]: Serial read error: {}",
                        get_timestamp(),
                        e
                    )?;
                    writer.flush()?;
                }
            }
        }

        // Check for user input (non-blocking)
        if let Ok(input) = input_rx.try_recv() {
            let clean = input.trim_end();
            if !clean.is_empty() {
                // Send to serial port
                let message = format!("{}\n", clean);
                if let Err(e) = port.write_all(message.as_bytes()) {
                    eprintln!("{color_red}❌ Write error: {e}{color_reset}");
                    if let Some(ref mut writer) = log_writer {
                        writeln!(writer, "ERROR [{}]: Write error: {}", get_timestamp(), e)?;
                        writer.flush()?;
                    }
                    continue;
                }

                if let Err(e) = port.flush() {
                    eprintln!("{color_red}❌ Flush error: {e}{color_reset}");
                    if let Some(ref mut writer) = log_writer {
                        writeln!(writer, "ERROR [{}]: Flush error: {}", get_timestamp(), e)?;
                        writer.flush()?;
                    }
                    continue;
                }

                // Log sent data
                if config.verbose {
                    println!(" [{}] Sent: {}", get_timestamp(), clean);
                }

                if let Some(ref mut writer) = log_writer {
                    writeln!(writer, "TX [{}]: {}", get_timestamp(), clean)?;
                    writer.flush()?;
                }

                // Small delay for processing
                thread::sleep(Duration::from_millis(100));
            }
        }

        // Small delay to prevent busy waiting
        thread::sleep(Duration::from_millis(10));
    }

    println!("{color_green} ComChan disconnected cleanly{color_reset}");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Handle config generation
    if args.generate_config {
        return generate_default_config(args.config_file);
    }

    // Load configuration file
    let config = load_config(args.config_file.clone())?;

    // Merge config file settings with command line arguments
    let merged_config = merge_config_and_args(config, args);

    // Handle list ports command
    if merged_config.list_ports {
        return list_available_ports();
    }

    // Handle auto port detection or manual port specification
    let port_name = if let Some(ref port) = merged_config.port {
        if port.to_lowercase() == "auto" {
            match port_finder::find_usb_port()? {
                Some(detected_port) => {
                    println!(
                        "{color_green} Auto-detected USB port: {}{color_reset}",
                        detected_port
                    );
                    detected_port
                }
                None => {
                    eprintln!(
                        "{color_red}❌ No USB serial ports found for auto-detection{color_reset}"
                    );
                    eprintln!(
                        "{color_yellow}󰌵 Try --list-ports to see available ports{color_reset}"
                    );
                    std::process::exit(1);
                }
            }
        } else {
            port.clone()
        }
    } else {
        eprintln!(
            "{color_red}❌ No port specified. Use --port <PORT>, --auto, or set port in config{color_reset}"
        );
        eprintln!(
            "{color_yellow}󰌵 Try --list-ports to see available ports or --generate-config to create a config file{color_reset}"
        );
        std::process::exit(1);
    };

    // Choose mode based on plot flag
    if merged_config.plot {
        run_plotter_mode(merged_config, port_name)
    } else {
        run_normal_mode(merged_config, port_name)
    }
}
