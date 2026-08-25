use std::io::{self, Write};
use std::process::Command;

type SerialPortBox = Box<dyn serialport::SerialPort>;
type DetachResult = Result<(Option<SerialPortBox>, Vec<String>), Box<dyn std::error::Error>>;

pub fn run_detached_shell(port: Option<SerialPortBox>) -> DetachResult {
    let _ = crossterm::terminal::disable_raw_mode();
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(
        stdout,
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show,
    );
    let _ = stdout.flush();

    drop(port);

    println!();
    println!("-------------------------------------------------");
    println!("  ComChan detached -- interactive shell");
    println!("  Serial port is released");
    println!("  Exit this shell (exit / CTRL+D) to reattach");
    println!("-------------------------------------------------");
    println!();

    // Shell
    let shell = std::env::var("SHELL").unwrap_or_else(|_| default_shell());
    let status = Command::new(&shell).env("COMCHAN_DETACHED", "1").status();

    match status {
        Ok(s) if !s.success() => {
            eprintln!("[ComChan] shell exited with {s}");
        }
        Err(e) => {
            eprintln!("[ComChan] failed to spawn shell '{shell}' : {e}");
        }
        _ => {}
    }

    println!();

    println!("-------------------------------------------------");
    println!("  Reattached to ComChan");
    println!("-------------------------------------------------");
    println!();

    Ok((None, Vec::new()))
}

fn default_shell() -> String {
    if cfg!(windows) {
        "cmd.exe".into()
    } else {
        "/bin/sh".into()
    }
}
