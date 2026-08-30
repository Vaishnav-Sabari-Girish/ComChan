//! Scrollable TUI serial monitor with search.
//!
//! Activated with `--tui`. Keys:
//!   /          search (incremental)
//!   n / N      next / prev match
//!   i          TX input mode
//!   ↑↓ PgUp/Dn scroll (disables auto-scroll)
//!   Enter      jump to bottom + auto-scroll
//!   c          clear buffer
//!   ?          help
//!   Ctrl+P     switch to plotter
//!   Ctrl+G     detach
//!   q / Esc    quit

use crate::config::MergedConfig;
use crate::rtt_reader::RttDefmtReader;
use crate::serial::{
    get_timestamp, parse_data_bits, parse_flow_control, parse_parity, parse_stop_bits,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use pretty_hex::*;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Read, Write};
use std::time::{Duration, Instant};

// ── Terminal cleanup guard ────────────────────────────────────────────────────

struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '\x1b' {
            out.push(c);
            continue;
        }

        match chars.peek().copied() {
            // CSI: ESC [ ... final (0x40..=0x7E)
            Some('[') => {
                chars.next();
                for ch in chars.by_ref() {
                    if ('\x40'..='\x7e').contains(&ch) {
                        break;
                    }
                }
            }
            // OSC: ESC ] ... BEL  or  ESC ] ... ESC \
            Some(']') => {
                chars.next();
                while let Some(ch) = chars.next() {
                    if ch == '\x07' {
                        break;
                    }
                    if ch == '\x1b' {
                        if chars.peek() == Some(&'\\') {
                            chars.next();
                        }
                        break;
                    }
                }
            }
            // SS2 / SS3: ESC N / ESC O + one character
            Some('N') | Some('O') => {
                chars.next();
                let _ = chars.next();
            }
            // DCS / PM / APC string: ESC P / ^ / _ ... ST
            Some('P') | Some('X') | Some('^') | Some('_') => {
                chars.next();
                while let Some(ch) = chars.next() {
                    if ch == '\x07' {
                        break;
                    }
                    if ch == '\x1b' {
                        if chars.peek() == Some(&'\\') {
                            chars.next();
                        }
                        break;
                    }
                }
            }
            // 2-char ESC sequence: ESC + final only (e.g. ESC c, ESC 7, ESC 8)
            // ESC + intermediate bytes + final
            Some(ch) if ('\x20'..='\x2f').contains(&ch) => {
                chars.next();
                while let Some(ch) = chars.peek().copied() {
                    if ('\x20'..='\x2f').contains(&ch) {
                        chars.next();
                    } else if ('\x30'..='\x7e').contains(&ch) {
                        chars.next();
                        break;
                    } else {
                        break;
                    }
                }
            }
            // 2-char ESC sequence: ESC + final only (e.g. ESC c, ESC 7, ESC 8)
            Some(ch) if ('\x30'..='\x7e').contains(&ch) => {
                chars.next();
            }
            _ => {}
        }
    }

    out
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// ── App state ─────────────────────────────────────────────────────────────────

enum Focus {
    Logs,
    Search,
    Input,
}

struct TuiState {
    logs: Vec<String>,
    scroll: usize,
    auto_scroll: bool,
    focus: Focus,
    search_query: String,
    search_matches: Vec<usize>,
    search_cursor: usize,
    input_buf: String,
    show_help: bool,
    status_msg: Option<(String, Instant)>,
    max_lines: usize,
    receive_buf: String,
}

impl TuiState {
    fn new() -> Self {
        Self {
            logs: Vec::new(),
            scroll: 0,
            auto_scroll: true,
            focus: Focus::Logs,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_cursor: 0,
            input_buf: String::new(),
            show_help: false,
            status_msg: None,
            max_lines: 50_000,
            receive_buf: String::new(),
        }
    }

    fn push_line(&mut self, line: String) {
        let will_match = if !self.search_query.is_empty() {
            line.to_lowercase()
                .contains(&self.search_query.to_lowercase())
        } else {
            false
        };

        self.logs.push(line);

        if will_match {
            self.search_matches.push(self.logs.len() - 1);
        }

        if self.logs.len() > self.max_lines {
            let drop_n = self.logs.len() - self.max_lines;
            self.logs.drain(0..drop_n);
            self.scroll = self.scroll.saturating_sub(drop_n);
            self.search_matches.retain_mut(|i| {
                if *i >= drop_n {
                    *i -= drop_n;
                    true
                } else {
                    false
                }
            });
            if self.search_cursor >= self.search_matches.len() {
                self.search_cursor = self.search_matches.len().saturating_sub(1);
            }
        }
    }

    fn recompute_search(&mut self) {
        self.search_matches.clear();
        self.search_cursor = 0;
        if self.search_query.is_empty() {
            return;
        }
        let q = self.search_query.to_lowercase();
        for (i, line) in self.logs.iter().enumerate() {
            if line.to_lowercase().contains(&q) {
                self.search_matches.push(i);
            }
        }
    }

    fn jump_to_match(&mut self, forward: bool) {
        if self.search_matches.is_empty() {
            self.set_status("No matches");
            return;
        }
        if forward {
            self.search_cursor = (self.search_cursor + 1) % self.search_matches.len();
        } else if self.search_cursor == 0 {
            self.search_cursor = self.search_matches.len() - 1;
        } else {
            self.search_cursor -= 1;
        }
        let line_idx = self.search_matches[self.search_cursor];
        self.scroll = line_idx;
        self.auto_scroll = false;
        self.set_status(&format!(
            "Match {}/{}",
            self.search_cursor + 1,
            self.search_matches.len()
        ));
    }

    fn set_status(&mut self, msg: &str) {
        self.status_msg = Some((msg.to_string(), Instant::now()));
    }

    fn clear_logs(&mut self) {
        self.logs.clear();
        self.scroll = 0;
        self.search_matches.clear();
        self.search_cursor = 0;
        self.auto_scroll = true;
        self.set_status("Cleared");
    }
}

// ── Public entry (all I/O on this thread — RttDefmtReader is !Send) ───────────

pub fn run_tui_mode(
    config: MergedConfig,
    port_name: String,
    passed_port: Option<Box<dyn serialport::SerialPort>>,
    passed_rtt: Option<RttDefmtReader>,
    #[cfg(feature = "ble")] active_ble_rx: Option<std::sync::mpsc::Receiver<crate::ble::BleEvent>>,
) -> Result<crate::AppExitState, Box<dyn std::error::Error>> {
    let skip_serial =
        config.simulate || config.replay_file.is_some() || config.rtt || port_name == "BLE_STREAM";

    let serial_settings = if skip_serial {
        None
    } else {
        Some((
            parse_data_bits(config.data_bits).map_err(|e| format!("Configuration error: {e}"))?,
            parse_stop_bits(config.stop_bits).map_err(|e| format!("Configuration error: {e}"))?,
            parse_parity(&config.parity).map_err(|e| format!("Configuration error: {e}"))?,
            parse_flow_control(&config.flow_control)
                .map_err(|e| format!("Configuration error: {e}"))?,
        ))
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let _cleanup = TerminalCleanup;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut log_writer: Option<BufWriter<std::fs::File>> = if let Some(ref path) = config.log_file {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Some(BufWriter::new(file))
    } else {
        None
    };

    // Propagate open errors — do not silently disable a requested --csv export
    let mut csv_streamer = if let Some(ref path) = config.csv_file {
        Some(
            crate::export::CsvStreamer::new(path)
                .map_err(|e| format!("Failed to open CSV file {path}: {e}"))?,
        )
    } else {
        None
    };

    let mut session_replayer = if let Some(ref path) = config.replay_file {
        Some(
            crate::replay::SessionReplayer::new(path)
                .map_err(|e| format!("Failed to open replay file '{path}': {e}"))?,
        )
    } else {
        None
    };

    let mut port = passed_port;
    let mut rtt_reader = if let Some(r) = passed_rtt {
        Some(r)
    } else if config.rtt {
        let elf = config.elf.as_deref().unwrap_or("");
        if elf.is_empty() {
            return Err("RTT mode requires an ELF file. Use --elf <path>".into());
        }
        Some(RttDefmtReader::new(elf, config.chip.clone())?)
    } else {
        None
    };

    if !skip_serial && let Some(p) = port.as_mut() {
        let _ = p.clear(serialport::ClearBuffer::Input);
    }

    let mut state = TuiState::new();
    let mut serial_buf = [0u8; 1024];
    let mut hex_buf: Vec<u8> = Vec::new();
    let mut last_draw = Instant::now();
    const MIN_DRAW: Duration = Duration::from_millis(33);
    let mut sim_t = 0.0_f64;

    #[cfg(feature = "ble")]
    let active_ble_rx = active_ble_rx;

    loop {
        // ── Keyboard ──────────────────────────────────────────────────────────
        if event::poll(Duration::from_millis(5))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind == KeyEventKind::Release {
                // ignore key-release on terminals that emit them
            } else if state.show_help {
                state.show_help = false;
            } else {
                match state.focus {
                    Focus::Search => match key.code {
                        KeyCode::Esc => {
                            state.focus = Focus::Logs;
                        }
                        KeyCode::Enter => {
                            state.focus = Focus::Logs;
                            if state.search_matches.is_empty() {
                                state.set_status("No matches");
                            } else {
                                state.search_cursor = 0;
                                state.scroll = state.search_matches[0];
                                state.auto_scroll = false;
                                state
                                    .set_status(&format!("Match 1/{}", state.search_matches.len()));
                            }
                        }
                        KeyCode::Backspace => {
                            state.search_query.pop();
                            state.recompute_search();
                        }
                        KeyCode::Char(c) => {
                            state.search_query.push(c);
                            state.recompute_search();
                        }
                        _ => {}
                    },
                    Focus::Input => match key.code {
                        KeyCode::Esc => {
                            state.focus = Focus::Logs;
                            state.input_buf.clear();
                        }
                        KeyCode::Enter => {
                            let line = std::mem::take(&mut state.input_buf);
                            if !line.is_empty() {
                                if let Some(ref mut p) = port {
                                    // Match classic monitor: CR terminator (Zephyr wants CRLF)
                                    let payload = if config.zephyr {
                                        format!("{line}\r\n")
                                    } else {
                                        format!("{line}\r")
                                    };
                                    match p.write_all(payload.as_bytes()).and_then(|_| p.flush()) {
                                        Ok(()) => {
                                            let tx_line =
                                                format!("TX [{}]: {line}", get_timestamp());
                                            if let Some(ref mut w) = log_writer {
                                                let _ = writeln!(w, "{tx_line}");
                                                let _ = w.flush();
                                            }
                                            state.push_line(tx_line);
                                        }
                                        Err(e) => {
                                            state.set_status(&format!("Write error: {e}"));
                                            port = None;
                                        }
                                    }
                                } else {
                                    state.set_status("No port — TX not sent");
                                }
                            }
                            state.focus = Focus::Logs;
                        }
                        KeyCode::Backspace => {
                            state.input_buf.pop();
                        }
                        KeyCode::Char(c) => {
                            state.input_buf.push(c);
                        }
                        _ => {}
                    },
                    Focus::Logs => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break;
                        }
                        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            disable_raw_mode().ok();
                            execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
                            std::mem::forget(_cleanup);
                            return Ok(crate::AppExitState::SwitchToPlotter {
                                port,
                                rtt_reader,
                                #[cfg(feature = "ble")]
                                ble_rx: active_ble_rx,
                            });
                        }
                        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            disable_raw_mode().ok();
                            execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
                            std::mem::forget(_cleanup);
                            return Ok(crate::AppExitState::Detach {
                                port: None,
                                rtt_reader: None,
                                #[cfg(feature = "ble")]
                                ble_rx: active_ble_rx,
                                resume_plotter: false,
                                port_name: port_name.clone(),
                            });
                        }
                        KeyCode::Char('?') => state.show_help = true,
                        KeyCode::Char('/') => {
                            state.focus = Focus::Search;
                            state.search_query.clear();
                            state.recompute_search();
                        }
                        KeyCode::Char('i') => {
                            state.focus = Focus::Input;
                            state.input_buf.clear();
                        }
                        KeyCode::Char('c') => state.clear_logs(),
                        KeyCode::Char('n') => state.jump_to_match(true),
                        KeyCode::Char('N') => state.jump_to_match(false),
                        KeyCode::Up => {
                            state.scroll = state.scroll.saturating_sub(1);
                            state.auto_scroll = false;
                        }
                        KeyCode::Down => {
                            state.scroll = state.scroll.saturating_add(1);
                            state.auto_scroll = false;
                        }
                        KeyCode::PageUp => {
                            state.scroll = state.scroll.saturating_sub(20);
                            state.auto_scroll = false;
                        }
                        KeyCode::PageDown => {
                            state.scroll = state.scroll.saturating_add(20);
                            state.auto_scroll = false;
                        }
                        KeyCode::Home => {
                            state.scroll = 0;
                            state.auto_scroll = false;
                        }
                        KeyCode::Enter | KeyCode::End => {
                            state.auto_scroll = true;
                        }
                        _ => {}
                    },
                }
            }
        }

        // ── Data sources ──────────────────────────────────────────────────────

        // Reconnect serial if needed (throttled on failure)
        if port.is_none()
            && let Some((data_bits, stop_bits, parity, flow_control)) = serial_settings
        {
            match serialport::new(&port_name, config.baud)
                .timeout(Duration::from_millis(config.timeout_ms))
                .data_bits(data_bits)
                .stop_bits(stop_bits)
                .parity(parity)
                .flow_control(flow_control)
                .open()
            {
                Ok(p) => {
                    if config.reset_delay_ms > 0 {
                        std::thread::sleep(Duration::from_millis(config.reset_delay_ms));
                    }
                    let _ = p.clear(serialport::ClearBuffer::Input);
                    port = Some(p);
                    state.set_status(&format!("Reconnected to {port_name}"));
                }
                Err(_) => {
                    state.set_status(&format!("Waiting for {port_name}…"));
                    std::thread::sleep(Duration::from_millis(200));
                }
            }
        }

        // Simulate — always feed CSV from the parseable payload, then choose display
        if config.simulate {
            sim_t += 0.05;
            let pitch = (sim_t * 0.5).sin() * 45.0;
            let roll = (sim_t * 0.8).cos() * 30.0;
            let yaw = (sim_t * 2.0) % 360.0;
            let payload = format!("Pitch: {pitch:.2}, Roll: {roll:.2}, Yaw: {yaw:.2}");

            if let Some(ref mut streamer) = csv_streamer {
                let readings = crate::parser::parse_sensor_data(&payload);
                let _ = streamer.write_row(&readings);
            }

            if config.hex_mode || config.hex_pretty {
                let hex_out = format!("{:?}", payload.as_bytes().hex_dump());
                for line in hex_out.lines() {
                    state.push_line(line.to_string());
                }
            } else {
                state.push_line(format!("RX [{}]: {payload}", get_timestamp()));
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        // Replay
        if let Some(ref mut replayer) = session_replayer {
            match replayer.next_payload() {
                crate::replay::ReplayEvent::Payload(payload) => {
                    let trimmed = payload.trim_end().to_string();
                    if let Some(ref mut streamer) = csv_streamer {
                        let readings = crate::parser::parse_sensor_data(&trimmed);
                        let _ = streamer.write_row(&readings);
                    }
                    state.push_line(format!("RX [{}]: {trimmed}", get_timestamp()));
                }
                crate::replay::ReplayEvent::Waiting => {}
                crate::replay::ReplayEvent::Eof => {
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }

        // RTT (stays on this thread — !Send)
        if let Some(ref mut reader) = rtt_reader {
            match reader.poll_logs() {
                Ok(logs) => {
                    for line in logs {
                        if let Some(ref mut w) = log_writer {
                            let _ = writeln!(w, "RX [{}]: {}", get_timestamp(), line.trim_end());
                            let _ = w.flush();
                        }
                        let clean = strip_ansi(line.trim_end());
                        if let Some(ref mut streamer) = csv_streamer {
                            let readings = crate::parser::parse_sensor_data(&clean);
                            let _ = streamer.write_row(&readings);
                        }
                        state.push_line(format!("RX [{}]: {clean}", get_timestamp()));
                    }
                }
                Err(e) => {
                    state.set_status(&format!("RTT lost: {e}. Reconnecting…"));
                    let elf = config.elf.as_deref().unwrap_or("");
                    match RttDefmtReader::new(elf, config.chip.clone()) {
                        Ok(new_reader) => {
                            *reader = new_reader;
                            state.set_status("RTT re-connected!");
                        }
                        Err(err) => {
                            state.set_status(&format!("RTT re-connect failed: {err}"));
                            std::thread::sleep(Duration::from_millis(500));
                        }
                    }
                }
            }
        }

        // BLE
        #[cfg(feature = "ble")]
        {
            if let Some(rx) = active_ble_rx.as_ref() {
                let mut n = 0;
                const MAX: usize = 32;
                while n < MAX {
                    match rx.try_recv() {
                        Ok(crate::ble::BleEvent::Disconnected) => {
                            state.set_status("BLE Connection Lost.");
                            break;
                        }
                        Ok(crate::ble::BleEvent::Payload(text)) => {
                            state.receive_buf.push_str(&text);
                            n += 1;
                            while let Some(pos) = state.receive_buf.find('\n') {
                                let line = state.receive_buf.drain(..=pos).collect::<String>();
                                let trimmed = strip_ansi(line.trim_end());
                                if trimmed.is_empty() {
                                    continue;
                                }
                                if let Some(ref mut w) = log_writer {
                                    let _ = writeln!(w, "RX [{}]: {}", get_timestamp(), trimmed);
                                    let _ = w.flush();
                                }
                                if let Some(ref mut streamer) = csv_streamer {
                                    let readings = crate::parser::parse_sensor_data(&trimmed);
                                    let _ = streamer.write_row(&readings);
                                }
                                state.push_line(format!("RX [{}]: {}", get_timestamp(), trimmed));
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
        }

        // Serial drain
        if let Some(ref mut p) = port {
            let mut drain_iters = 0;
            const MAX_DRAIN: usize = 10;
            loop {
                if drain_iters >= MAX_DRAIN {
                    break;
                }
                drain_iters += 1;
                match p.bytes_to_read() {
                    Ok(avail) if avail > 0 => match p.read(&mut serial_buf) {
                        Ok(n) if n > 0 => {
                            let raw = &serial_buf[..n];

                            if config.hex_mode || config.hex_pretty {
                                let (should_print, data_to_print) = if config.hex_pretty {
                                    hex_buf.extend_from_slice(raw);
                                    if hex_buf.contains(&b'\n') || hex_buf.len() >= 64 {
                                        let data = std::mem::take(&mut hex_buf);
                                        (true, data)
                                    } else {
                                        (false, Vec::new())
                                    }
                                } else {
                                    (true, raw.to_vec())
                                };

                                if should_print {
                                    let hex_out = format!("{:?}", data_to_print.hex_dump());
                                    if let Some(ref mut w) = log_writer {
                                        let _ = writeln!(
                                            w,
                                            "RX HEX [{}]:\n{}",
                                            get_timestamp(),
                                            hex_out
                                        );
                                        let _ = w.flush();
                                    }
                                    for line in hex_out.lines() {
                                        state.push_line(line.to_string());
                                    }
                                }
                            } else {
                                let chunk = String::from_utf8_lossy(raw);
                                state.receive_buf.push_str(&chunk);
                                while let Some(pos) = state.receive_buf.find('\n') {
                                    let line = state.receive_buf.drain(..=pos).collect::<String>();
                                    let trimmed = strip_ansi(line.trim_end());
                                    if trimmed.is_empty() {
                                        continue;
                                    }
                                    if let Some(ref mut w) = log_writer {
                                        let _ =
                                            writeln!(w, "RX [{}]: {}", get_timestamp(), trimmed);
                                        let _ = w.flush();
                                    }
                                    if let Some(ref mut streamer) = csv_streamer {
                                        let readings = crate::parser::parse_sensor_data(&trimmed);
                                        let _ = streamer.write_row(&readings);
                                    }
                                    state.push_line(format!(
                                        "RX [{}]: {}",
                                        get_timestamp(),
                                        trimmed
                                    ));
                                }
                            }
                        }
                        Ok(_) => break,
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => break,
                        Err(e) => {
                            state.set_status(&format!("Read error: {e}"));
                            break;
                        }
                    },
                    Ok(_) => break,
                    Err(e) => {
                        state.set_status(&format!("Read error: {e}"));
                        port = None;
                        break;
                    }
                }
            }
        }

        // Expire status
        if let Some((_, t)) = &state.status_msg
            && t.elapsed() > Duration::from_secs(3)
        {
            state.status_msg = None;
        }

        if last_draw.elapsed() < MIN_DRAW {
            continue;
        }
        last_draw = Instant::now();

        let baud = config.baud;
        let port_disp = port_name.clone();
        let log_count = state.logs.len();
        let match_info = if state.search_query.is_empty() {
            String::new()
        } else if state.search_matches.is_empty() {
            format!("  |  /{}  0 matches", state.search_query)
        } else {
            format!(
                "  |  /{}  {}/{}",
                state.search_query,
                state.search_cursor + 1,
                state.search_matches.len()
            )
        };

        terminal.draw(|f| {
            let root = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ])
                .split(f.area());

            let view_h = root[0].height.saturating_sub(2) as usize;
            let max_scroll = log_count.saturating_sub(view_h);
            if state.auto_scroll {
                state.scroll = max_scroll;
            } else {
                state.scroll = state.scroll.min(max_scroll);
            }

            let title =
                format!(" ComChan TUI  [{port_disp} @ {baud}]  {log_count} lines{match_info} ");
            let border_color = if matches!(state.focus, Focus::Logs) {
                Color::Cyan
            } else {
                Color::DarkGray
            };

            let start = state.scroll;
            let end = (start + view_h).min(log_count);
            let q_lower = state.search_query.to_lowercase();
            let current_match_line = state.search_matches.get(state.search_cursor).copied();

            let lines: Vec<Line> = state.logs[start..end]
                .iter()
                .enumerate()
                .map(|(offset, raw)| {
                    let idx = start + offset;
                    let is_current = current_match_line == Some(idx);
                    if !q_lower.is_empty() && raw.to_lowercase().contains(&q_lower) {
                        let style = if is_current {
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Yellow)
                        };
                        Line::from(Span::styled(raw.clone(), style))
                    } else {
                        Line::from(raw.as_str())
                    }
                })
                .collect();

            let para = Paragraph::new(lines).block(
                Block::default()
                    .title(Span::styled(
                        title,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            );
            f.render_widget(para, root[0]);

            let mut sb = ScrollbarState::default()
                .content_length(max_scroll)
                .position(state.scroll);
            f.render_stateful_widget(
                Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight),
                root[0],
                &mut sb,
            );

            let bottom = match state.focus {
                Focus::Search => {
                    let text = format!("/{}", state.search_query);
                    Paragraph::new(text).block(
                        Block::default()
                            .title(" Search (Enter confirm · Esc cancel) ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Yellow)),
                    )
                }
                Focus::Input => {
                    let text = format!("> {}", state.input_buf);
                    Paragraph::new(text).block(
                        Block::default()
                            .title(" TX (Enter send · Esc cancel) ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green)),
                    )
                }
                Focus::Logs => Paragraph::new(" Press '/' search · 'i' TX · '?' help · 'q' quit ")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::DarkGray)),
                    ),
            };
            f.render_widget(bottom, root[1]);

            let status = if let Some((ref msg, _)) = state.status_msg {
                Span::styled(format!(" {msg} "), Style::default().fg(Color::Red))
            } else if state.auto_scroll {
                Span::styled(" AUTO ", Style::default().fg(Color::Green))
            } else {
                Span::styled(" SCROLL LOCK ", Style::default().fg(Color::Yellow))
            };
            f.render_widget(
                Paragraph::new(Line::from(status)).alignment(Alignment::Left),
                root[2],
            );

            if state.show_help {
                let area = centered_rect(60, 60, f.area());
                let help = vec![
                    Line::from(""),
                    Line::from(" [/]           Start search"),
                    Line::from(" [n] / [N]     Next / previous match"),
                    Line::from(" [i]           TX input mode"),
                    Line::from(" [↑][↓][PgUp]  Scroll (locks auto-scroll)"),
                    Line::from(" [Enter]/[End] Jump to bottom + auto-scroll"),
                    Line::from(" [c]           Clear log buffer"),
                    Line::from(" [Ctrl+P]      Switch to plotter"),
                    Line::from(" [Ctrl+G]      Detach"),
                    Line::from(" [q] / [Esc]   Quit"),
                    Line::from(""),
                    Line::from(" Press any key to close…")
                        .style(Style::default().fg(Color::DarkGray)),
                ];
                f.render_widget(Clear, area);
                f.render_widget(
                    Paragraph::new(help).block(
                        Block::default()
                            .title(" TUI Monitor Shortcuts ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Cyan)),
                    ),
                    area,
                );
            }
        })?;
    }

    Ok(crate::AppExitState::Quit)
}
