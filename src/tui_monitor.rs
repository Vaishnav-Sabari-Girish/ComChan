//! Scrollable TUI serial monitor with search.
//!
//! Activated with `--tui`. Keys:
//!   /          search
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
        // Live search: if query is active, record match index before push
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
    let mut last_draw = Instant::now();
    const MIN_DRAW: Duration = Duration::from_millis(33);
    let mut sim_t = 0.0_f64;

    loop {
        // ── Keyboard ──────────────────────────────────────────────────────────
        if event::poll(Duration::from_millis(5))?
            && let Event::Key(key) = event::read()?
        {
            // Ignore key-release events on terminals that emit them
            if key.kind == KeyEventKind::Release {
                // skip
            } else if state.show_help {
                state.show_help = false;
            } else {
                match state.focus {
                    Focus::Search => match key.code {
                        KeyCode::Esc => {
                            state.focus = Focus::Logs;
                        }
                        KeyCode::Enter => {
                            state.recompute_search();
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
                        }
                        KeyCode::Char(c) => {
                            state.search_query.push(c);
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
                                    let payload = if config.zephyr {
                                        format!("{line}\r\n")
                                    } else {
                                        format!("{line}\n")
                                    };
                                    let _ = p.write_all(payload.as_bytes());
                                    let _ = p.flush();
                                }
                                state.push_line(format!("TX [{}]: {line}", get_timestamp()));
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
                            // Forget cleanup so Drop doesn't double-leave
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

        // Reconnect serial if needed
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
                }
            }
        }

        // Simulate
        if config.simulate {
            sim_t += 0.05;
            let pitch = (sim_t * 0.5).sin() * 45.0;
            let roll = (sim_t * 0.8).cos() * 30.0;
            let yaw = (sim_t * 2.0) % 360.0;
            state.push_line(format!(
                "RX [{}]: Pitch: {pitch:.2}, Roll: {roll:.2}, Yaw: {yaw:.2}",
                get_timestamp()
            ));
            std::thread::sleep(Duration::from_millis(50));
        }

        // Replay
        if let Some(ref mut replayer) = session_replayer {
            match replayer.next_payload() {
                crate::replay::ReplayEvent::Payload(payload) => {
                    state.push_line(format!("RX [{}]: {}", get_timestamp(), payload.trim_end()));
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
                        state.push_line(format!("RX [{}]: {}", get_timestamp(), line.trim_end()));
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
                                let trimmed = line.trim_end().to_string();
                                if let Some(ref mut w) = log_writer {
                                    let _ = writeln!(w, "RX [{}]: {}", get_timestamp(), trimmed);
                                    let _ = w.flush();
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
                            let chunk = String::from_utf8_lossy(&serial_buf[..n]);
                            state.receive_buf.push_str(&chunk);
                            while let Some(pos) = state.receive_buf.find('\n') {
                                let line = state.receive_buf.drain(..=pos).collect::<String>();
                                let trimmed = line.trim_end().to_string();
                                if let Some(ref mut w) = log_writer {
                                    let _ = writeln!(w, "RX [{}]: {}", get_timestamp(), trimmed);
                                    let _ = w.flush();
                                }
                                state.push_line(format!("RX [{}]: {}", get_timestamp(), trimmed));
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
                    Line::from(" [Enter]/End] Jump to bottom + auto-scroll"),
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

    // Normal quit path — TerminalCleanup Drop handles leave/raw_mode
    Ok(crate::AppExitState::Quit)
}
