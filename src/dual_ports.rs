use crate::config::MergedConfig;
use crate::serial::{parse_data_bits, parse_flow_control, parse_parity, parse_stop_bits};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Span;
use ratatui::widgets::Clear;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

pub enum DualEvent {
    Port1(String),
    Port2(String),
    Error1(String),
    Error2(String),
}

struct DualMonitorState {
    port1_logs: Vec<String>,
    port2_logs: Vec<String>,
    scroll1: usize,
    scroll2: usize,
    active_pane: u8, // 0 for left and 1 for right
    auto_scroll1: bool,
    auto_scroll2: bool,
    show_help: bool,
    input_mode: bool,
    input1: String,
    input2: String,
}

impl DualMonitorState {
    fn new() -> Self {
        Self {
            port1_logs: Vec::new(),
            port2_logs: Vec::new(),
            scroll1: 0,
            scroll2: 0,
            active_pane: 0,
            auto_scroll1: true,
            auto_scroll2: true,
            show_help: false,
            input_mode: false,
            input1: String::new(),
            input2: String::new(),
        }
    }
}

fn split_filename(base_path: &Option<String>, suffix: &str) -> Option<String> {
    base_path.as_ref().map(|path_str| {
        let path = Path::new(path_str);
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = path.extension().unwrap_or_default().to_string_lossy();

        if ext.is_empty() {
            format!("{}_{}", stem, suffix)
        } else {
            format!("{}_{}.{}", stem, suffix, ext)
        }
    })
}

fn spawn_serial_thread(
    port_name: String,
    cfg: MergedConfig,
    tx: mpsc::Sender<DualEvent>,
    rx_cmd: mpsc::Receiver<String>,
    is_port1: bool,
) {
    thread::spawn(move || {
        let wrap_event = |text: String| {
            if is_port1 {
                DualEvent::Port1(text)
            } else {
                DualEvent::Port2(text)
            }
        };
        let wrap_error = |err: String| {
            if is_port1 {
                DualEvent::Error1(err)
            } else {
                DualEvent::Error2(err)
            }
        };

        if cfg.simulate {
            let mut counter = 0;
            loop {
                while let Ok(cmd) = rx_cmd.try_recv() {
                    let _ = tx.send(wrap_event(format!("TX: {}\n", cmd)));
                }
                let text = format!(
                    "SIM [Port {}]: Packet {}\n",
                    if is_port1 { 1 } else { 2 },
                    counter
                );
                let _ = tx.send(wrap_event(text));
                counter += 1;
                thread::sleep(Duration::from_millis(500));
            }
        } else {
            let data_bits = match parse_data_bits(cfg.data_bits) {
                Ok(v) => v,
                Err(e) => {
                    let _ = tx.send(wrap_error(format!("Config error: {}", e)));
                    return;
                }
            };
            let stop_bits = match parse_stop_bits(cfg.stop_bits) {
                Ok(v) => v,
                Err(e) => {
                    let _ = tx.send(wrap_error(format!("Config error: {}", e)));
                    return;
                }
            };
            let parity = match parse_parity(&cfg.parity) {
                Ok(v) => v,
                Err(e) => {
                    let _ = tx.send(wrap_error(format!("Config error: {}", e)));
                    return;
                }
            };
            let flow_control = match parse_flow_control(&cfg.flow_control) {
                Ok(v) => v,
                Err(e) => {
                    let _ = tx.send(wrap_error(format!("Config error: {}", e)));
                    return;
                }
            };

            match serialport::new(&port_name, cfg.baud)
                .timeout(Duration::from_millis(cfg.timeout_ms))
                .data_bits(data_bits)
                .stop_bits(stop_bits)
                .parity(parity)
                .flow_control(flow_control)
                .open()
            {
                Ok(mut port) => {
                    let _ = port.write_data_terminal_ready(true);
                    let mut buffer = [0; 1024];
                    loop {
                        // Drain commands
                        while let Ok(cmd) = rx_cmd.try_recv() {
                            let payload = format!("{}\r\n", cmd);
                            if let Err(e) = port
                                .write_all(payload.as_bytes())
                                .and_then(|_| port.flush())
                            {
                                let _ = tx.send(wrap_event(format!("Write Error: {}", e)));
                            } else {
                                let _ = tx.send(wrap_event(format!("TX: {}\n", cmd)));
                            }
                        }

                        match port.read(&mut buffer) {
                            Ok(n) if n > 0 => {
                                let _ = tx.send(wrap_event(
                                    String::from_utf8_lossy(&buffer[..n]).into_owned(),
                                ));
                            }
                            Err(e) if e.kind() != io::ErrorKind::TimedOut => {
                                let _ = tx.send(wrap_error(format!("Read Error: {}", e)));
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(wrap_error(format!("Failed to open {}: {}", port_name, e)));
                }
            }
        }
    });
}

pub fn run_dual_mode(
    config: MergedConfig,
    ports: Vec<String>,
) -> Result<crate::AppExitState, Box<dyn std::error::Error>> {
    let (tx_cmd1, rx_cmd1) = mpsc::channel::<String>();
    let (tx_cmd2, rx_cmd2) = mpsc::channel::<String>();

    let port1_name = ports[0].clone();
    let port2_name = ports[1].clone();

    // Split logging and CSV
    let log1_path = split_filename(&config.log_file, "port1");
    let log2_path = split_filename(&config.log_file, "port2");

    let csv1_path = split_filename(&config.csv_file, "port1");
    let csv2_path = split_filename(&config.csv_file, "port2");

    let mut log1_writer = log1_path.and_then(|p| {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .ok()
            .map(BufWriter::new)
    });
    let mut log2_writer = log2_path.and_then(|p| {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .ok()
            .map(BufWriter::new)
    });

    let mut csv1_streamer = csv1_path.and_then(|p| crate::export::CsvStreamer::new(&p).ok());
    let mut csv2_streamer = csv2_path.and_then(|p| crate::export::CsvStreamer::new(&p).ok());

    let (tx, rx) = mpsc::channel::<DualEvent>();

    // Serial port 1 thread
    let tx1 = tx.clone();
    let p1_name = port1_name.clone();
    let cfg1 = config.clone();

    spawn_serial_thread(p1_name, cfg1, tx1, rx_cmd1, true);

    // Serial port 2 thread
    let tx2 = tx.clone();
    let p2_name = port2_name.clone();
    let cfg2 = config.clone();

    spawn_serial_thread(p2_name, cfg2, tx2, rx_cmd2, false);

    // TUI
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let _cleanup = TerminalCleanup;
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = DualMonitorState::new();
    let mut buf1 = String::new();
    let mut buf2 = String::new();

    loop {
        while let Ok(event) = rx.try_recv() {
            match event {
                DualEvent::Port1(text) => {
                    buf1.push_str(&text);
                    while let Some(pos) = buf1.find('\n') {
                        let line = buf1.drain(..=pos).collect::<String>();
                        let clean = line.trim_end().to_string();
                        if !clean.is_empty() {
                            app_state.port1_logs.push(clean.clone());
                            if let Some(ref mut w) = log1_writer {
                                let _ = writeln!(w, "{}", clean);
                                let _ = w.flush();
                            }
                            if let Some(ref mut csv) = csv1_streamer {
                                let readings = crate::parser::parse_sensor_data(&clean);
                                let _ = csv.write_row(&readings);
                            }
                        }
                    }
                }
                DualEvent::Port2(text) => {
                    buf2.push_str(&text);
                    while let Some(pos) = buf2.find('\n') {
                        let line = buf2.drain(..=pos).collect::<String>();
                        let clean = line.trim_end().to_string();
                        if !clean.is_empty() {
                            app_state.port2_logs.push(clean.clone());
                            if let Some(ref mut w) = log2_writer {
                                let _ = writeln!(w, "{}", clean);
                                let _ = w.flush();
                            }
                            if let Some(ref mut csv) = csv2_streamer {
                                let readings = crate::parser::parse_sensor_data(&clean);
                                let _ = csv.write_row(&readings);
                            }
                        }
                    }
                }

                DualEvent::Error1(err) => app_state.port1_logs.push(format!("ERROR: {}", err)),
                DualEvent::Error2(err) => app_state.port2_logs.push(format!("ERROR: {}", err)),
            }
        }

        // Limit memory to prevent RAM exhaustion
        if app_state.port1_logs.len() > 2000 {
            app_state.port1_logs.drain(0..500);
            app_state.scroll1 = app_state.scroll1.saturating_sub(500);
        }
        if app_state.port2_logs.len() > 2000 {
            app_state.port2_logs.drain(0..500);
            app_state.scroll2 = app_state.scroll2.saturating_sub(500);
        }

        // Handle Input
        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            if app_state.show_help {
                app_state.show_help = false;
                continue;
            }

            if app_state.input_mode {
                match key.code {
                    KeyCode::Esc => app_state.input_mode = false, // Visual Mode (No typing)
                    KeyCode::Enter => {
                        let cmd = if app_state.active_pane == 0 {
                            app_state.input1.drain(..).collect::<String>()
                        } else {
                            app_state.input2.drain(..).collect::<String>()
                        };

                        if !cmd.is_empty() {
                            if app_state.active_pane == 0 {
                                let _ = tx_cmd1.send(cmd);
                            } else {
                                let _ = tx_cmd2.send(cmd);
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        if app_state.active_pane == 0 {
                            app_state.input1.push(c);
                        } else {
                            app_state.input2.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if app_state.active_pane == 0 {
                            app_state.input1.pop();
                        } else {
                            app_state.input2.pop();
                        }
                    }
                    _ => {}
                }
                continue;
            }

            match key.code {
                KeyCode::Char('i') => app_state.input_mode = true,
                KeyCode::Char('?') => app_state.show_help = true,
                KeyCode::Char('q') => break,
                KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => break,
                KeyCode::Left => app_state.active_pane = 0,
                KeyCode::Right => app_state.active_pane = 1,
                KeyCode::Tab => {
                    if app_state.active_pane == 0 {
                        app_state.active_pane = 1;
                    } else {
                        app_state.active_pane = 0;
                    }
                }
                KeyCode::Up => {
                    if app_state.active_pane == 0 {
                        app_state.scroll1 = app_state.scroll1.saturating_sub(1);
                        app_state.auto_scroll1 = false;
                    } else {
                        app_state.scroll2 = app_state.scroll2.saturating_sub(1);
                        app_state.auto_scroll2 = false;
                    }
                }
                KeyCode::Down => {
                    if app_state.active_pane == 0 {
                        app_state.scroll1 = app_state.scroll1.saturating_add(1);
                    } else {
                        app_state.scroll2 = app_state.scroll2.saturating_add(1);
                    }
                }

                // Snap to bottom
                KeyCode::Enter => {
                    if app_state.active_pane == 0 {
                        app_state.scroll1 = app_state.port1_logs.len();
                        app_state.auto_scroll1 = true;
                    } else {
                        app_state.scroll2 = app_state.port2_logs.len();
                        app_state.auto_scroll2 = true;
                    }
                }
                _ => {}
            }
        }

        terminal.draw(|f| {
            let root_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(f.area());

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(root_layout[0]);

            let pane1_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(chunks[0]);

            let pane2_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(chunks[1]);

            let active_style = Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD);
            let inactive_style = Style::default().fg(Color::DarkGray);

            let p1_style = if app_state.active_pane == 0 {
                active_style
            } else {
                inactive_style
            };
            let p2_style = if app_state.active_pane == 1 {
                active_style
            } else {
                inactive_style
            };

            // Pane 1 render
            let lines1 = app_state.port1_logs.len();

            // Auto-scroll
            let max_scroll1 = lines1.saturating_sub(chunks[0].height.saturating_sub(2) as usize);
            if app_state.auto_scroll1 || app_state.scroll1 >= max_scroll1 {
                app_state.scroll1 = max_scroll1;
                app_state.auto_scroll1 = true;
            }

            let mut scrollbar_state1 = ScrollbarState::default()
                .content_length(max_scroll1)
                .position(app_state.scroll1);

            let block1 = Block::default()
                .title(format!(" Port 1 {} ", port1_name))
                .borders(Borders::ALL)
                .border_style(p1_style);

            let lines1: Vec<Line> = app_state
                .port1_logs
                .iter()
                .map(|log| {
                    if log.starts_with("TX:") {
                        Line::from(Span::styled(log, Style::default().fg(Color::Cyan)))
                    } else if log.starts_with("ERROR:") {
                        Line::from(Span::styled(log, Style::default().fg(Color::Red)))
                    } else {
                        Line::from(log.as_str())
                    }
                })
                .collect();

            let paragraph1 = Paragraph::new(lines1)
                .block(block1)
                .scroll((app_state.scroll1 as u16, 0));

            f.render_widget(paragraph1, pane1_layout[0]);
            f.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                pane1_layout[0],
                &mut scrollbar_state1,
            );

            // Input box
            let mut disp1 = app_state.input1.clone();
            let input_style1 = if app_state.active_pane == 0 && app_state.input_mode {
                disp1.push('█');
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            f.render_widget(
                Paragraph::new(disp1).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" TX (Press 'i' to type) ")
                        .border_style(input_style1),
                ),
                pane1_layout[1],
            );

            // Pane 2 render
            let lines2 = app_state.port2_logs.len();

            // Auto-scroll
            let max_scroll2 = lines2.saturating_sub(chunks[1].height.saturating_sub(2) as usize);
            if app_state.auto_scroll2 || app_state.scroll2 >= max_scroll2 {
                app_state.scroll2 = max_scroll2;
                app_state.auto_scroll2 = true;
            }

            let mut scrollbar_state2 = ScrollbarState::default()
                .content_length(max_scroll2)
                .position(app_state.scroll2);

            let block2 = Block::default()
                .title(format!(" Port 2 {} ", port2_name))
                .borders(Borders::ALL)
                .border_style(p2_style);

            let lines2: Vec<Line> = app_state
                .port2_logs
                .iter()
                .map(|log| {
                    if log.starts_with("TX:") {
                        Line::from(Span::styled(log, Style::default().fg(Color::Cyan)))
                    } else if log.starts_with("ERROR:") {
                        Line::from(Span::styled(log, Style::default().fg(Color::Red)))
                    } else {
                        Line::from(log.as_str())
                    }
                })
                .collect();

            let paragraph2 = Paragraph::new(lines2)
                .block(block2)
                .scroll((app_state.scroll2 as u16, 0));

            f.render_widget(paragraph2, pane2_layout[0]);
            f.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                pane2_layout[0],
                &mut scrollbar_state2,
            );

            // Input box
            let mut disp2 = app_state.input2.clone();
            let input_style2 = if app_state.active_pane == 1 && app_state.input_mode {
                disp2.push('█');
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            f.render_widget(
                Paragraph::new(disp2).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" TX (Press 'i' to type) ")
                        .border_style(input_style2),
                ),
                pane2_layout[1],
            );

            let hint_text = Line::from(" Press '?' for help ")
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center);

            f.render_widget(Paragraph::new(hint_text), root_layout[1]);

            if app_state.show_help {
                let area = centered_rect(50, 50, f.area());

                let help_text = vec![
                    Line::from(""),
                    Line::from(" [?]          : Show / Hide this menu"),
                    Line::from(" [Tab]        : Toggle active pane (Left/Right)"),
                    Line::from(" [←] / [→]    : Select active pane"),
                    Line::from(" [↑] / [↓]    : Scroll active pane & pause auto-scroll"),
                    Line::from(" [i]          : Enter typing mode"),
                    Line::from(" [Esc]        : Exit typing mode"),
                    Line::from(" [Enter]      : Jump to bottom & resume auto-scroll"),
                    Line::from(" [q]          : Quit Dual Monitor"),
                    Line::from(" [Ctrl+C]     : Force Quit"),
                    Line::from(""),
                    Line::from(" Press any key to close... ")
                        .style(Style::default().fg(Color::DarkGray)),
                ];

                f.render_widget(Clear, area);

                let help_block = Paragraph::new(help_text).block(
                    Block::default()
                        .title(" Dual Monitor Shortcuts ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                );
                f.render_widget(help_block, area);
            }
        })?;
    }

    // Cleanup
    Ok(crate::AppExitState::Quit)
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
