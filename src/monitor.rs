use crate::config::MergedConfig;
use crate::rtt_reader::RttDefmtReader;
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

use pretty_hex::*;

enum MonitorCommand {
    Repaint(u8),
    SwitchMode,
    Quit,
}

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

macro_rules! poll_ctrl_rx_while_waiting {
    ($ctrl_rx:expr, $running:expr) => {
        let range = core::range::Range { start: 0, end: 20 };
        for _ in range {
            if let Ok(cmd) = $ctrl_rx.try_recv() {
                match cmd {
                    MonitorCommand::SwitchMode => {
                        terminal::disable_raw_mode().ok();
                        return Ok(crate::AppExitState::SwitchToPlotter {
                            port: None,
                            rtt_reader: None,
                            #[cfg(feature = "ble")]
                            ble_rx: None,
                        });
                    }
                    MonitorCommand::Quit => {
                        println!("\r\n{color_yellow}󰏃 Shutting down ComChan…{color_reset}");
                        $running.store(false, std::sync::atomic::Ordering::SeqCst);
                        return Ok(crate::AppExitState::Quit);
                    }
                    _ => {}
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
    };
}

pub fn run_normal_mode(
    config: MergedConfig,
    port_name: String,
    passed_port: Option<Box<dyn serialport::SerialPort>>,
    passed_rtt: Option<crate::rtt_reader::RttDefmtReader>,
    #[cfg(feature = "ble")] active_ble_rx: Option<std::sync::mpsc::Receiver<String>>,
) -> Result<crate::AppExitState, Box<dyn std::error::Error>> {
    let skip_serial =
        config.simulate || config.replay_file.is_some() || config.rtt || port_name == "BLE_STREAM";

    let serial_config = if skip_serial {
        None
    } else {
        Some((
            parse_data_bits(config.data_bits).map_err(|e| format!("Configuration error: {}", e))?,
            parse_stop_bits(config.stop_bits).map_err(|e| format!("Configuration error: {}", e))?,
            parse_parity(&config.parity).map_err(|e| format!("Configuration error: {}", e))?,
            parse_flow_control(&config.flow_control)
                .map_err(|e| format!("Configuration error: {}", e))?,
        ))
    };

    // 1. Setup logging ONCE
    let mut log_writer = if let Some(log_path) = &config.log_file {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(|e| format!("Failed to open log file {}: {}", log_path, e))?;
        Some(BufWriter::new(file))
    } else {
        None
    };

    let mut csv_streamer = if let Some(csv_path) = &config.csv_file {
        crate::export::CsvStreamer::new(csv_path)
            .map_err(|e| format!("Failed to open CSV file {}: {}", csv_path, e))
            .ok()
    } else {
        None
    };

    let mut session_replayer = if let Some(ref path) = config.replay_file {
        Some(
            crate::replay::SessionReplayer::new(path)
                .map_err(|e| format!("Failed to open replay file '{}': {}", path, e))?,
        )
    } else {
        None
    };

    println!("{color_green} Listening… (Ctrl+C to exit, Ctrl+L to clear screen){color_reset}\n");

    // 2. Setup channels and input thread ONCE
    let (input_tx, input_rx) = mpsc::channel::<String>();
    let (ctrl_tx, ctrl_rx) = mpsc::channel::<MonitorCommand>();
    thread::spawn(move || {
        terminal::enable_raw_mode().ok();
        let mut line_buf = String::new();

        loop {
            if event::poll(Duration::from_millis(10)).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(KeyEvent {
                        code, modifiers, ..
                    })) => match (code, modifiers) {
                        (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                            print!("\x1bc\x1b[5 q");
                            io::stdout().flush().ok();
                            ctrl_tx.send(MonitorCommand::Repaint(b'\r')).ok();
                        }
                        (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                            ctrl_tx.send(MonitorCommand::SwitchMode).ok();
                            break;
                        }
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            ctrl_tx.send(MonitorCommand::Quit).ok();
                            break;
                        }
                        (KeyCode::Enter, _) => {
                            let _ = input_tx.send(line_buf.clone());
                            line_buf.clear();
                        }
                        (KeyCode::Backspace, _) => {
                            line_buf.pop();
                            print!("\x08 \x08");
                            io::stdout().flush().ok();
                        }
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

    let mut buffer = [0u8; 1024];
    let mut last_sent: Option<String> = None;
    let mut line_acc = String::new();
    let mut rx_buf = String::new();
    let mut lines_discarded = 0;
    const DISCARD_COUNT: usize = 5;

    let mut active_port = passed_port;
    let mut active_rtt = passed_rtt;

    // Connection & Reconnection
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        // Initialize RTT reader
        let mut rtt_reader = if let Some(r) = active_rtt.take() {
            Some(r)
        } else if config.rtt {
            let elf = config.elf.as_deref().unwrap_or("");

            match RttDefmtReader::new(elf, config.chip.clone()) {
                Ok(reader) => Some(reader),
                Err(e) => {
                    let err_msg = e.to_string();

                    if err_msg.contains("No such file")
                        || err_msg.contains("No defmt table")
                        || err_msg.contains("ChipNotFound")
                        || err_msg.contains("requires an ELF file")
                    {
                        return Err(e);
                    }
                    // Otherwise, treat as a transient hardware/connection error and wait
                    print!("\r{color_yellow}⏳ Waiting for RTT target...{color_reset}\x1b[K");
                    io::stdout().flush().ok();

                    poll_ctrl_rx_while_waiting!(ctrl_rx, running);

                    continue;
                }
            }
        } else {
            None
        };

        // Serial Port
        let mut port = if let Some(p) = active_port.take() {
            Some(p)
        } else if skip_serial {
            None
        } else {
            // Match handles the error instead of returning it
            let (data_bits, stop_bits, parity, flow_control) = serial_config.unwrap();
            match serialport::new(&port_name, config.baud)
                .timeout(Duration::from_millis(config.timeout_ms))
                .data_bits(data_bits)
                .stop_bits(stop_bits)
                .parity(parity)
                .flow_control(flow_control)
                .open()
            {
                Ok(mut p) => {
                    let _ = p.write_data_terminal_ready(false);
                    thread::sleep(Duration::from_millis(config.reset_delay_ms));
                    let _ = p.write_all(b"\r");
                    let _ = p.flush();

                    if config.zephyr {
                        thread::sleep(Duration::from_millis(100));
                        let _ = p.write_all(b"shell echo off\r");
                        let _ = p.flush();
                        thread::sleep(Duration::from_millis(100));
                    }

                    println!(
                        "\r\n{color_green}🔌 Connected to {} at {} baud{color_reset}",
                        port_name, config.baud
                    );
                    if config.verbose {
                        println!(
                            "\r{color_blue}⚙️  Config: {} data bits, {} stop bits, {} parity, {} flow control{color_reset}",
                            config.data_bits, config.stop_bits, config.parity, config.flow_control
                        );
                        if let Some(log_path) = &config.log_file {
                            println!("\r{color_blue} Logging to: {}{color_reset}", log_path);
                        }
                    }
                    Some(p)
                }
                Err(_) => {
                    // Retry connection every 1 second
                    print!(
                        "\r{color_yellow}⏳ Waiting for device on {}...{color_reset}\x1b[K",
                        port_name
                    );
                    io::stdout().flush().ok();

                    poll_ctrl_rx_while_waiting!(ctrl_rx, running);

                    continue;
                }
            }
        };

        let mut is_connected = true;
        let mut hex_buf: Vec<u8> = Vec::new();

        // Read / Write Data
        while running.load(std::sync::atomic::Ordering::SeqCst) && is_connected {
            if config.simulate {
                let t = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                let sim_text = format!(
                    "Temperature: {:2}, Humidity: {:.2}, Pressure: {:.2}\r\n",
                    (t * 1.0).sin() * 50.0,
                    (t * 0.8).cos() * 50.0,
                    (t * 0.5).sin() * 50.0
                );

                if config.hex_mode || config.hex_pretty {
                    let hex_out = format!("{:?}", sim_text.as_bytes().hex_dump());
                    let raw_mode_safe_hex = hex_out.replace('\n', "\r\n");
                    print!("\r\n{}\r\n", raw_mode_safe_hex);
                    io::stdout().flush().ok();
                } else {
                    io::stdout().write_all(sim_text.as_bytes()).ok();
                    io::stdout().flush().ok();
                }

                if let Some(ref mut streamer) = csv_streamer {
                    let clean = strip_ansi(sim_text.trim());
                    let readings = crate::parser::parse_sensor_data(&clean);
                    let _ = streamer.write_row(&readings);
                }

                thread::sleep(Duration::from_millis(500));
            }

            if let Some(ref mut replayer) = session_replayer {
                match replayer.next_payload() {
                    crate::replay::ReplayEvent::Payload(payload) => {
                        let text = format!("{}\r\n", payload);

                        io::stdout().write_all(text.as_bytes()).ok();
                        io::stdout().flush().ok();
                    }
                    crate::replay::ReplayEvent::Waiting => {}
                    crate::replay::ReplayEvent::Eof => {
                        println!("\n{color_yellow}Replay Finished.{color_reset}");
                        running.store(false, std::sync::atomic::Ordering::SeqCst);
                        break;
                    }
                }
            }

            if let Some(reader) = rtt_reader.as_mut() {
                match reader.poll_logs() {
                    Ok(logs) => {
                        if logs.is_empty() {
                            thread::sleep(Duration::from_millis(10));
                        } else {
                            for line in logs {
                                let trimmed = line.trim_end();

                                if trimmed.is_empty() {
                                    continue;
                                }

                                if config.verbose {
                                    print!("\r[{}] {}\r\n", get_timestamp(), trimmed);
                                } else {
                                    print!("\r{}\r\n", trimmed);
                                }
                                io::stdout().flush().ok();

                                if let Some(ref mut writer) = log_writer {
                                    writeln!(writer, "RX [{}]: {}", get_timestamp(), trimmed).ok();
                                    let _ = writer.flush();
                                }

                                if let Some(ref mut streamer) = csv_streamer {
                                    let readings = crate::parser::parse_sensor_data(trimmed);
                                    let _ = streamer.write_row(&readings);
                                }
                            }
                        }
                    }

                    Err(e) => {
                        eprintln!(
                            "\r\n{color_yellow}⚠️ RTT connection lost ({color_red}{}{color_yellow}). Attempting to reconnect...{color_reset}",
                            e
                        );

                        if let Some(ref mut writer) = log_writer {
                            writeln!(
                                writer,
                                "ERROR [{}]: RTT connection lost: {}",
                                get_timestamp(),
                                e
                            )
                            .ok();
                            let _ = writer.flush();
                        }
                        is_connected = false;
                    }
                }
            }

            #[cfg(feature = "ble")]
            {
                if let Some(rx) = active_ble_rx.as_ref() {
                    while let Ok(text) = rx.try_recv() {
                        rx_buf.push_str(&text);

                        while let Some(pos) = rx_buf.find('\n') {
                            let full_line = rx_buf.drain(..=pos).collect::<String>();
                            let clean = strip_ansi(&full_line);
                            let trimmed = clean.trim_end();

                            if trimmed.is_empty() {
                                continue;
                            }

                            if lines_discarded < DISCARD_COUNT {
                                lines_discarded += 1;
                                continue;
                            }

                            if config.verbose {
                                print!("\r[{}] {}\r\n", get_timestamp(), trimmed);
                            } else {
                                print!("\r{}\r\n", trimmed);
                            }
                            io::stdout().flush().ok();

                            if let Some(ref mut writer) = log_writer {
                                writeln!(writer, "RX [{}]: {}", get_timestamp(), trimmed).ok();
                                let _ = writer.flush();
                            }

                            if let Some(ref mut streamer) = csv_streamer {
                                let readings = crate::parser::parse_sensor_data(trimmed);
                                let _ = streamer.write_row(&readings);
                            }
                        }
                    }
                }
            }

            if let Some(p) = port.as_mut() {
                match p.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        let raw = &buffer[..n];

                        if config.hex_mode || config.hex_pretty {
                            let (should_print, data_to_print) = if config.hex_pretty {
                                hex_buf.extend_from_slice(raw);

                                if hex_buf.contains(&b'\n') || hex_buf.len() >= 64 {
                                    let data = hex_buf.clone();
                                    hex_buf.clear();
                                    (true, data)
                                } else {
                                    (false, Vec::new())
                                }
                            } else {
                                (true, raw.to_vec())
                            };

                            if should_print {
                                let hex_out = format!("{:?}", data_to_print.hex_dump());
                                let raw_mode_safe_hex = hex_out.replace('\n', "\r\n");

                                print!("\r\n{}\r\n", raw_mode_safe_hex);
                                io::stdout().flush().ok();

                                if let Some(ref mut writer) = log_writer {
                                    writeln!(writer, "RX HEX [{}]:\n{}", get_timestamp(), hex_out)
                                        .ok();
                                    let _ = writer.flush();
                                }
                            }
                            continue;
                        }

                        let text = String::from_utf8_lossy(raw);

                        // ── Echo suppression ─────────────────────────────────────────
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
                        if config.verbose {
                            let mut remaining = text.as_ref();
                            while let Some(pos) = remaining.find('\n') {
                                let chunk = &remaining[..=pos];
                                let clean = strip_ansi(chunk);
                                if !clean.trim().is_empty() {
                                    print!("[{}] {}", get_timestamp(), chunk);
                                } else {
                                    print!("{}", chunk);
                                }
                                remaining = &remaining[pos + 1..];
                            }
                            if !remaining.is_empty() {
                                print!("{}", remaining);
                            }
                        } else {
                            io::stdout().write_all(raw).ok();
                        }
                        io::stdout().flush().ok();

                        // ── Logging & CSV streaming ───────────────────────────────────────────────────
                        rx_buf.push_str(&text);

                        while let Some(pos) = rx_buf.find('\n') {
                            let full_line = rx_buf.drain(..=pos).collect::<String>();
                            let clean = strip_ansi(&full_line);
                            let trimmed = clean.trim_end();

                            if trimmed.is_empty() {
                                continue;
                            }

                            if lines_discarded < DISCARD_COUNT {
                                lines_discarded += 1;
                                continue;
                            }

                            if let Some(ref mut writer) = log_writer {
                                writeln!(writer, "RX [{}]: {}", get_timestamp(), trimmed).ok();
                                let _ = writer.flush();
                            }

                            if let Some(ref mut streamer) = csv_streamer {
                                let readings = crate::parser::parse_sensor_data(trimmed);
                                let _ = streamer.write_row(&readings);
                            }
                        }

                        if line_acc.contains('\n') {
                            line_acc.clear();
                        }
                    }
                    Ok(_) => {}
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
                    Err(e) => {
                        // Read Error -> Trigger Reconnection
                        eprintln!(
                            "\r\n{color_yellow}⚠️ Device connection lost ({color_red}{}{color_yellow}). Attempting to reconnect...{color_reset}",
                            e
                        );

                        if let Some(ref mut writer) = log_writer {
                            writeln!(
                                writer,
                                "ERROR [{}]: Connection lost: {}",
                                get_timestamp(),
                                e
                            )
                            .ok();
                            let _ = writer.flush();
                        }

                        is_connected = false;
                    }
                }
            }

            // Write user input
            if let Ok(input) = input_rx.try_recv() {
                let clean = input.trim_end();
                if !clean.is_empty() {
                    let message = format!("{}\r", clean);

                    if let Some(p) = port.as_mut() {
                        if let Err(e) = p.write_all(message.as_bytes()) {
                            // Write Error -> Trigger Reconnection
                            eprintln!("\r\n{color_red}❌ Write error: {e}{color_reset}");
                            if let Some(ref mut writer) = log_writer {
                                writeln!(writer, "ERROR [{}]: Write error: {}", get_timestamp(), e)
                                    .ok();
                                let _ = writer.flush();
                            }
                            is_connected = false;
                            continue;
                        }
                        p.flush().ok();
                    } else if config.rtt {
                        eprintln!(
                            "\r\n{color_yellow}Sending Input over RTT is not supported{color_reset}"
                        );
                        continue;
                    }

                    last_sent = Some(clean.to_string());
                    line_acc.clear();

                    if config.verbose {
                        print!("\r\n[{}] Sent: {}\r\n", get_timestamp(), clean);
                        io::stdout().flush().ok();
                    }
                    if let Some(ref mut writer) = log_writer {
                        writeln!(writer, "TX [{}]: {}", get_timestamp(), clean).ok();
                        let _ = writer.flush();
                    }

                    thread::sleep(Duration::from_millis(100));
                }
            }

            // ── Control bytes (e.g. Ctrl+L repaint) ─────────────────────────────
            if let Ok(cmd) = ctrl_rx.try_recv() {
                match cmd {
                    MonitorCommand::Repaint(byte) => {
                        if let Some(p) = port.as_mut() {
                            let _ = p.write_all(&[byte]);
                            let _ = p.flush();
                        }
                    }
                    MonitorCommand::SwitchMode => {
                        terminal::disable_raw_mode().ok();
                        return Ok(crate::AppExitState::SwitchToPlotter {
                            port,
                            rtt_reader,
                            #[cfg(feature = "ble")]
                            ble_rx: active_ble_rx,
                        });
                    }
                    MonitorCommand::Quit => {
                        println!("\r\n{color_yellow}󰏃 Shutting down ComChan…{color_reset}");
                        running.store(false, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    }

    println!("\r\n{color_green} ComChan disconnected cleanly{color_reset}");
    terminal::disable_raw_mode().ok();
    Ok(crate::AppExitState::Quit)
}
