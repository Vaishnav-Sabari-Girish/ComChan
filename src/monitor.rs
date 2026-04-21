use crate::config::MergedConfig;
use crate::serial::{
    get_timestamp, parse_data_bits, parse_flow_control, parse_parity, parse_stop_bits,
};
use inline_colorization::*;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());

    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next();

                    for ch in chars.by_ref() {
                        if ch.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
                _ => {
                    chars.next();
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

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

    // Disable DTR
    let _ = port.write_data_terminal_ready(false);

    thread::sleep(Duration::from_millis(config.reset_delay_ms));
    let _ = port.write_all(b"\r");
    let _ = port.flush();

    thread::sleep(Duration::from_millis(100));
    let _ = port.write_all(b"shell echo off\r");
    let _ = port.flush();
    thread::sleep(Duration::from_millis(100));

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
    println!("{color_green} Listening… (Ctrl+C to exit, Ctrl+L to clear screen){color_reset}\n");

    // Spawn a thread to read stdin without blocking the serial loop
    let (input_tx, input_rx) = mpsc::channel::<String>();

    // Control bytes channel
    let (ctrl_tx, ctrl_rx) = mpsc::channel::<u8>();
    thread::spawn(move || {
        terminal::enable_raw_mode().ok();
        let mut line_buf = String::new();

        loop {
            if event::poll(Duration::from_millis(10)).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(KeyEvent {
                        code, modifiers, ..
                    })) => match (code, modifiers) {
                        // Ctrl+L → clear screen, nudge device to redraw prompt
                        (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                            print!("\x1bc\x1b[5 q");
                            io::stdout().flush().ok();
                            ctrl_tx.send(b'\r').ok();
                        }
                        // Ctrl+C → break (let the main ctrlc handler take over)
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                        // Enter → send buffered line
                        (KeyCode::Enter, _) => {
                            let _ = input_tx.send(line_buf.clone());
                            line_buf.clear();
                        }
                        // Backspace
                        (KeyCode::Backspace, _) => {
                            line_buf.pop();
                            print!("\x08 \x08");
                            io::stdout().flush().ok();
                        }
                        // Regular character
                        (KeyCode::Char(c), _) => {
                            line_buf.push(c);
                            print!("{}", c);
                            io::stdout().flush().ok();
                        }
                        _ => {}
                    },
                    Err(_) => break,
                    _ => {}
                }
            }
        }

        terminal::disable_raw_mode().ok();
    });

    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\n{color_yellow}󰏃 Shutting down ComChan…{color_reset}");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })?;

    let mut buffer = [0u8; 1024];
    let mut log_writer = log_writer;
    let mut last_sent: Option<String> = None;
    // Accumulates bytes for echo-suppression and verbose timestamp logging only
    let mut line_acc = String::new();

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        // ── Read from serial ─────────────────────────────────────────────────
        match port.read(&mut buffer) {
            Ok(n) if n > 0 => {
                let raw = &buffer[..n];
                let text = String::from_utf8_lossy(raw);

                // ── Echo suppression ─────────────────────────────────────────
                // Accumulate into line_acc and check if this chunk is the
                // echo of the last sent command; if so, swallow it silently.
                line_acc.push_str(&text);
                let mut suppress = false;
                if let Some(ref sent) = last_sent {
                    let clean_acc = strip_ansi(&line_acc).trim().to_string();
                    if clean_acc == *sent {
                        suppress = true;
                        last_sent = None;
                        line_acc.clear();
                    }
                }
                if suppress {
                    continue;
                }

                // ── Verbose timestamp prefix ─────────────────────────────────
                // In verbose mode, prepend a timestamp before each line of
                // output. We still pass the original bytes through for colour.
                if config.verbose {
                    // Walk the incoming text line by line, printing a timestamp
                    // before each newline-terminated chunk.
                    let mut remaining = text.as_ref();
                    while let Some(pos) = remaining.find('\n') {
                        let chunk = &remaining[..=pos];
                        let clean = strip_ansi(chunk);
                        // Only print timestamp for non-empty lines
                        if !clean.trim().is_empty() {
                            print!("[{}] {}", get_timestamp(), chunk);
                        } else {
                            print!("{}", chunk);
                        }
                        remaining = &remaining[pos + 1..];
                    }
                    // Print any trailing partial line (e.g. the prompt)
                    if !remaining.is_empty() {
                        print!("{}", remaining);
                    }
                } else {
                    // Non-verbose: pass raw bytes straight through so Zephyr's
                    // own ANSI colours and \r\n handling reach the terminal
                    // untouched.
                    io::stdout().write_all(raw)?;
                }
                io::stdout().flush()?;

                // ── Logging ───────────────────────────────────────────────────
                if let Some(ref mut writer) = log_writer {
                    // Log each complete line
                    let mut remaining = text.as_ref();
                    while let Some(pos) = remaining.find('\n') {
                        let chunk = &remaining[..=pos];
                        let clean = strip_ansi(chunk);
                        writeln!(writer, "RX [{}]: {}", get_timestamp(), clean.trim_end())?;
                        remaining = &remaining[pos + 1..];
                    }
                    writer.flush()?;
                }

                // Clear line accumulator once we've seen a full line
                if line_acc.contains('\n') {
                    line_acc.clear();
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
                let message = format!("{}\r", clean);
                if let Err(e) = port.write_all(message.as_bytes()) {
                    eprintln!("{color_red}❌ Write error: {e}{color_reset}");
                    if let Some(ref mut writer) = log_writer {
                        writeln!(writer, "ERROR [{}]: Write error: {}", get_timestamp(), e)?;
                        writer.flush()?;
                    }
                    continue;
                }
                port.flush()?;

                last_sent = Some(clean.to_string());
                line_acc.clear();

                if config.verbose {
                    print!("\r\n[{}] Sent: {}\r\n", get_timestamp(), clean);
                    io::stdout().flush()?;
                }
                if let Some(ref mut writer) = log_writer {
                    writeln!(writer, "TX [{}]: {}", get_timestamp(), clean)?;
                    writer.flush()?;
                }

                thread::sleep(Duration::from_millis(100));
            }
        }

        // ── Control bytes (e.g. Ctrl+L repaint) ─────────────────────────────
        if let Ok(byte) = ctrl_rx.try_recv() {
            let _ = port.write_all(&[byte]);
            let _ = port.flush();
        }

        thread::sleep(Duration::from_millis(10));
    }

    println!("\r\n{color_green} ComChan disconnected cleanly{color_reset}");
    terminal::disable_raw_mode().ok();
    Ok(())
}
