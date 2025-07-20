use clap::Parser;
use inline_colorization::*;
use serialport::{self, DataBits, FlowControl, Parity, StopBits};
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Read, Write};
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

#[derive(Parser)]
#[command(
    name = "comchan",
    version = "0.1.7",
    author = "Vaishnav-Sabari-Girish",
    about = "Blazingly Fast Minimal Serial Monitor with Plotting"
)]
struct Args {
    #[arg(short = 'p', long = "port")]
    port: String,

    #[arg(short = 'r', long = "baud", default_value = "9600")]
    baud: u32,

    #[arg(short = 'd', long = "data-bits", default_value = "8")]
    data_bits: u8,

    #[arg(short = 's', long = "stop-bits", default_value = "1")]
    stop_bits: u8,

    #[arg(long = "parity", default_value = "none")]
    parity: String,

    #[arg(long = "flow-control", default_value = "none")]
    flow_control: String,

    #[arg(short = 't', long = "timeout", default_value = "500")]
    timeout_ms: u64,

    #[arg(long = "reset-delay", default_value = "1000")]
    reset_delay_ms: u64,

    #[arg(short = 'l', long = "log")]
    log_file: Option<String>,

    #[arg(long = "list-ports", action = clap::ArgAction::SetTrue)]
    list_ports: bool,

    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::SetTrue)]
    verbose: bool,

    #[arg(long = "plot", action = clap::ArgAction::SetTrue)]
    plot: bool,

    #[arg(long = "plot-points", default_value = "100")]
    plot_points: usize,
}

fn list_available_ports() -> Result<(), Box<dyn std::error::Error>> {
    println!("{color_cyan}📋 Available Serial Ports:{color_reset}");
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

fn run_plotter_mode(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal for plotting
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Parse serial port configuration
    let data_bits = parse_data_bits(args.data_bits)?;
    let stop_bits = parse_stop_bits(args.stop_bits)?;
    let parity = parse_parity(&args.parity)?;
    let flow_control = parse_flow_control(&args.flow_control)?;

    // Open serial port
    let mut port = serialport::new(&args.port, args.baud)
        .timeout(Duration::from_millis(args.timeout_ms))
        .data_bits(data_bits)
        .stop_bits(stop_bits)
        .parity(parity)
        .flow_control(flow_control)
        .open()?;

    // Optional log file setup
    let mut log_writer = if let Some(log_path) = &args.log_file {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        Some(BufWriter::new(file))
    } else {
        None
    };

    thread::sleep(Duration::from_millis(args.reset_delay_ms));

    let mut data: Vec<(f64, f64)> = Vec::with_capacity(args.plot_points);
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
                            if data.len() > args.plot_points {
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
                        args.port, args.baud
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

fn run_normal_mode(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Parse serial port configuration
    let data_bits =
        parse_data_bits(args.data_bits).map_err(|e| format!("Configuration error: {}", e))?;
    let stop_bits =
        parse_stop_bits(args.stop_bits).map_err(|e| format!("Configuration error: {}", e))?;
    let parity = parse_parity(&args.parity).map_err(|e| format!("Configuration error: {}", e))?;
    let flow_control = parse_flow_control(&args.flow_control)
        .map_err(|e| format!("Configuration error: {}", e))?;

    // Open serial port with full configuration
    let mut port = serialport::new(&args.port, args.baud)
        .timeout(Duration::from_millis(args.timeout_ms))
        .data_bits(data_bits)
        .stop_bits(stop_bits)
        .parity(parity)
        .flow_control(flow_control)
        .open()
        .map_err(|e| format!("Failed to open port {}: {}", args.port, e))?;

    // Optional log file setup
    let log_writer = if let Some(log_path) = &args.log_file {
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
    thread::sleep(Duration::from_millis(args.reset_delay_ms));

    // Print connection info
    println!(
        "{color_green}📡 ComChan connected to {} at {} baud{color_reset}",
        args.port, args.baud
    );
    if args.verbose {
        println!(
            "{color_blue}⚙️  Configuration: {} data bits, {} stop bits, {} parity, {} flow control{color_reset}",
            args.data_bits, args.stop_bits, args.parity, args.flow_control
        );
        if let Some(log_path) = &args.log_file {
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
                        if args.verbose {
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
                if args.verbose {
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

    // Handle list ports command
    if args.list_ports {
        return list_available_ports();
    }

    // Choose mode based on plot flag
    if args.plot {
        run_plotter_mode(args)
    } else {
        run_normal_mode(args)
    }
}
