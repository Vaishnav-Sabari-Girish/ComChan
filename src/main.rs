use clap::Parser;
use inline_colorization::*;
use serialport::{self, DataBits, FlowControl, Parity, StopBits};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
    version = "0.1.8",
    author = "Vaishnav-Sabari-Girish",
    about = "Blazingly Fast Minimal Serial Monitor with Plotting"
)]
struct Args {
    #[arg(short = 'p', long = "port", help = "Serial port to connect to")]
    port: Option<String>,

    #[arg(short = 'r', long = "baud")]
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

    #[arg(short = 'l', long = "log")]
    log_file: Option<String>,

    #[arg(long = "list-ports", action = clap::ArgAction::SetTrue)]
    list_ports: bool,

    #[arg(long = "auto", action = clap::ArgAction::SetTrue, help = "Auto-detect USB serial port")]
    auto: Option<bool>,

    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::SetTrue)]
    verbose: Option<bool>,

    #[arg(long = "plot", action = clap::ArgAction::SetTrue)]
    plot: Option<bool>,

    #[arg(long = "plot-points")]
    plot_points: Option<usize>,

    #[arg(long = "config", short = 'c', help = "Path to config file (default: ~/.config/comchan/comchan.toml)")]
    config_file: Option<PathBuf>,

    #[arg(long = "generate-config", action = clap::ArgAction::SetTrue, help = "Generate a default config file")]
    generate_config: bool,
}

fn find_config_file(specified_path: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(path) = specified_path {
        if path.exists() {
            return Some(path);
        }
        return None;
    }

    // Check for config file in current directory first
    let current_dir_config = PathBuf::from("comchan.toml");
    if current_dir_config.exists() {
        return Some(current_dir_config);
    }

    // Check for config file in ~/.config/comchan/
    if let Some(home_dir) = dirs::home_dir() {
        let config_dir_path = home_dir.join(".config").join("comchan").join("comchan.toml");
        if config_dir_path.exists() {
            return Some(config_dir_path);
        }

        // Fallback to old location for backward compatibility
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
        println!("{color_blue}📋 Loaded config from: {}{color_reset}", path.display());
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

fn generate_default_config(path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = if let Some(specified_path) = path {
        specified_path
    } else {
        // Default to ~/.config/comchan/comchan.toml
        if let Some(home_dir) = dirs::home_dir() {
            let config_dir = home_dir.join(".config").join("comchan");
            
            // Create the directory if it doesn't exist
            fs::create_dir_all(&config_dir)?;
            
            config_dir.join("comchan.toml")
        } else {
            // Fallback to current directory if home directory can't be determined
            PathBuf::from("comchan.toml")
        }
    };
    
    let default_config = Config::default();
    let toml_content = toml::to_string_pretty(&default_config)?;
    
    // Add comments to the generated config
    let commented_config = format!(
        r#"# ComChan Configuration File
# 
# This file contains default settings for comchan serial monitor.
# Command line arguments will override these settings.
# 
# To use auto-detection, set port = "auto"
# Available parity options: "none", "odd", "even"
# Available flow control options: "none", "software", "hardware"

{}
"#,
        toml_content
    );
    
    // Ensure the parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(&config_path, commented_config)?;
    println!("{color_green}✅ Generated default config file: {}{color_reset}", config_path.display());
    println!("{color_blue}💡 Edit the file to customize your default settings{color_reset}");
    
    Ok(())
}

fn merge_config_and_args(config: Config, args: Args) -> MergedConfig {
    MergedConfig {
        port: args.port.or(config.port),
        baud: args.baud.or(config.baud).unwrap_or(9600),
        data_bits: args.data_bits.or(config.data_bits).unwrap_or(8),
        stop_bits: args.stop_bits.or(config.stop_bits).unwrap_or(1),
        parity: args.parity.or(config.parity).unwrap_or("none".to_string()),
        flow_control: args.flow_control.or(config.flow_control).unwrap_or("none".to_string()),
        timeout_ms: args.timeout_ms.or(config.timeout_ms).unwrap_or(500),
        reset_delay_ms: args.reset_delay_ms.or(config.reset_delay_ms).unwrap_or(1000),
        log_file: args.log_file.or(config.log_file),
        list_ports: args.list_ports,
        verbose: args.verbose.or(config.verbose).unwrap_or(false),
        plot: args.plot.or(config.plot).unwrap_or(false),
        plot_points: args.plot_points.or(config.plot_points).unwrap_or(100),
    }
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

fn list_available_ports() -> Result<(), Box<dyn std::error::Error>> {
    println!("{color_cyan}📋 All Available Serial Ports:{color_reset}");
    let ports = serialport::available_ports()?;

    if ports.is_empty() {
        println!("  {color_yellow}⚠️  No serial ports found{color_reset}");
        return Ok(());
    }

    for port in ports {
        println!(
            "  🔌 {} - {}",
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
    // Show detailed USB info
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

fn parse_numeric_value(line: &str) -> Option<f64> {
    // Try to parse the entire line as a number first
    if let Ok(value) = line.trim().parse::<f64>() {
        return Some(value);
    }

    // Look for numbers in the line (handles cases like "Temperature: 25.3")
    let words: Vec<&str> = line.split_whitespace().collect();
    for word in words {
        // Remove common non-numeric characters
        let cleaned = word.trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-');
        if let Ok(value) = cleaned.parse::<f64>() {
            return Some(value);
        }
    }

    None
}

fn run_plotter_mode(config: MergedConfig, port_name: String) -> Result<(), Box<dyn std::error::Error>> {
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

    let mut data: Vec<(f64, f64)> = Vec::with_capacity(config.plot_points);
    let mut x = 0.0;
    let mut buffer = [0u8; 1024];
    let mut received = String::new();
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

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

                        // Try to parse numeric value from the line
                        if let Some(y) = parse_numeric_value(clean_line) {
                            data.push((x, y));

                            // Update y bounds for dynamic scaling
                            if y < y_min {
                                y_min = y;
                            }
                            if y > y_max {
                                y_max = y;
                            }

                            // Maintain rolling window
                            if data.len() > config.plot_points {
                                data.remove(0);
                            }

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
        let (x_min, x_max) = if data.is_empty() {
            (0.0, 10.0)
        } else {
            (data[0].0, data[data.len() - 1].0)
        };

        // Calculate y-axis bounds with some padding
        let (chart_y_min, chart_y_max) = if y_min.is_finite() && y_max.is_finite() && y_min != y_max
        {
            let padding = (y_max - y_min) * 0.1;
            (y_min - padding, y_max + padding)
        } else if y_min.is_finite() && y_max.is_finite() {
            (y_min - 1.0, y_max + 1.0)
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

        // Draw the chart
        terminal.draw(|f| {
            let size = f.area();

            let chart = Chart::new(vec![
                Dataset::default()
                    .name("Serial Data")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&data),
            ])
            .block(
                Block::default()
                    .title(format!(
                        "Live Serial Plotter - {} @ {} baud (Press 'q' or ESC to exit)",
                        port_name, config.baud
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
            );

            f.render_widget(chart, size);
        })?;
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn run_normal_mode(config: MergedConfig, port_name: String) -> Result<(), Box<dyn std::error::Error>> {
    // Parse serial port configuration
    let data_bits =
        parse_data_bits(config.data_bits).map_err(|e| format!("Configuration error: {}", e))?;
    let stop_bits =
        parse_stop_bits(config.stop_bits).map_err(|e| format!("Configuration error: {}", e))?;
    let parity = parse_parity(&config.parity).map_err(|e| format!("Configuration error: {}", e))?;
    let flow_control = parse_flow_control(&config.flow_control)
        .map_err(|e| format!("Configuration error: {}", e))?;

    // Open serial port with full configuration
    let mut port = serialport::new(&port_name, config.baud)
        .timeout(Duration::from_millis(config.timeout_ms))
        .data_bits(data_bits)
        .stop_bits(stop_bits)
        .parity(parity)
        .flow_control(flow_control)
        .open()
        .map_err(|e| format!("Failed to open port {}: {}", port_name, e))?;

    // Optional log file setup
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

    // Reset delay
    thread::sleep(Duration::from_millis(config.reset_delay_ms));

    // Print connection info
    println!(
        "{color_green}📡 ComChan connected to {} at {} baud{color_reset}",
        port_name, config.baud
    );
    if config.verbose {
        println!(
            "{color_blue}⚙️  Configuration: {} data bits, {} stop bits, {} parity, {} flow control{color_reset}",
            config.data_bits, config.stop_bits, config.parity, config.flow_control
        );
        if let Some(log_path) = &config.log_file {
            println!("{color_blue}📝 Logging to: {}{color_reset}", log_path);
        }
    }
    println!("{color_green}🔄 Listening... (Ctrl+C to exit){color_reset}\n");

    // Setup channels for non-blocking input
    let (input_tx, input_rx) = mpsc::channel::<String>();

    // Spawn input handling thread
    thread::spawn(move || {
        let stdin = io::stdin();
        loop {
            let mut input = String::new();
            match stdin.read_line(&mut input) {
                Ok(_) => {
                    if input_tx.send(input).is_err() {
                        break; // Main thread has ended
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Setup Ctrl+C handler
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\n{color_yellow}🛑 Shutting down ComChan...{color_reset}");
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
                            print!("📥 [{}] {}", get_timestamp(), line);
                        } else {
                            print!("📥 {}", line);
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
                    println!("📤 [{}] Sent: {}", get_timestamp(), clean);
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

    println!("{color_green}✅ ComChan disconnected cleanly{color_reset}");
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
                   println!("{color_green}🔍 Auto-detected USB port: {}{color_reset}", detected_port);
                    detected_port
                }
                None => {
                    eprintln!("{color_red}❌ No USB serial ports found for auto-detection{color_reset}");
                    eprintln!("{color_yellow}💡 Try --list-ports to see available ports{color_reset}");
                    std::process::exit(1);
                }
            }
        } else {
            port.clone()
        }
    } else {
        eprintln!("{color_red}❌ No port specified. Use --port <PORT>, --auto, or set port in config{color_reset}");
        eprintln!("{color_yellow}💡 Try --list-ports to see available ports or --generate-config to create a config file{color_reset}");
        std::process::exit(1);
    };

    // Choose mode based on plot flag
    if merged_config.plot {
        run_plotter_mode(merged_config, port_name)
    } else {
        run_normal_mode(merged_config, port_name)
    }
}
