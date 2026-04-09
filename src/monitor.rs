use crate::config::MergedConfig;
use crate::serial::{
    get_timestamp, parse_data_bits, parse_flow_control, parse_parity, parse_stop_bits,
};
use inline_colorization::*;
use serialport;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn run_normal_mode(
    config: MergedConfig,
    port_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
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
        "{color_green} ComChan connected to {} at {} baud{color_reset}",
        port_name, config.baud
    );
    if config.verbose {
        println!(
            "{color_blue}⚙️  Config: {} data bits, {} stop bits, {} parity, {} flow control{color_reset}",
            config.data_bits, config.stop_bits, config.parity, config.flow_control
        );
        if let Some(log_path) = &config.log_file {
            println!("{color_blue} Logging to: {}{color_reset}", log_path);
        }
    }
    println!("{color_green} Listening… (Ctrl+C to exit){color_reset}\n");

    // Spawn a thread to read stdin without blocking the serial loop
    let (input_tx, input_rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        let stdin = io::stdin();
        loop {
            let mut input = String::new();
            match stdin.lock().read_line(&mut input) {
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
        println!("\n{color_yellow}󰏃 Shutting down ComChan…{color_reset}");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })?;

    let mut buffer = [0u8; 1024];
    let mut received = String::new();
    let mut log_writer = log_writer;

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        // ── Read from serial ─────────────────────────────────────────────────
        match port.read(&mut buffer) {
            Ok(n) if n > 0 => {
                let output = String::from_utf8_lossy(&buffer[..n]);
                received.push_str(&output);

                while let Some(line_end) = received.find('\n') {
                    let line = received.drain(..=line_end).collect::<String>();

                    if config.verbose {
                        print!(" [{}] {}", get_timestamp(), line);
                    } else {
                        print!(" {}", line);
                    }
                    io::stdout().flush()?;

                    if let Some(ref mut writer) = log_writer {
                        writeln!(writer, "RX [{}]: {}", get_timestamp(), line.trim_end())?;
                        writer.flush()?;
                    }
                }
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
            Err(e) => {
                eprintln!("{color_red}❌ Serial read error: {e}{color_reset}");
                if let Some(ref mut writer) = log_writer {
                    writeln!(writer, "ERROR [{}]: {}", get_timestamp(), e)?;
                    writer.flush()?;
                }
            }
        }

        // ── Write user input ─────────────────────────────────────────────────
        if let Ok(input) = input_rx.try_recv() {
            let clean = input.trim_end();
            if !clean.is_empty() {
                let message = format!("{}\n", clean);
                if let Err(e) = port.write_all(message.as_bytes()) {
                    eprintln!("{color_red}❌ Write error: {e}{color_reset}");
                    if let Some(ref mut writer) = log_writer {
                        writeln!(writer, "ERROR [{}]: Write error: {}", get_timestamp(), e)?;
                        writer.flush()?;
                    }
                    continue;
                }
                port.flush()?;

                if config.verbose {
                    println!(" [{}] Sent: {}", get_timestamp(), clean);
                }
                if let Some(ref mut writer) = log_writer {
                    writeln!(writer, "TX [{}]: {}", get_timestamp(), clean)?;
                    writer.flush()?;
                }

                thread::sleep(Duration::from_millis(100));
            }
        }

        thread::sleep(Duration::from_millis(10));
    }

    println!("{color_green} ComChan disconnected cleanly{color_reset}");
    Ok(())
}
