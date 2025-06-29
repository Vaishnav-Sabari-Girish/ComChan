use clap::Parser;
use serialport;
use std::io::{self, Read, Write};
use std::time::Duration;
use inline_colorization::*;

#[derive(Parser)]
#[command(name = "comchan", version = "0.0.1", author = "Vaishnav-Sabari-Girish", about = "Blazingly Fast Minimal Serial Monitor")]
struct Args {
    #[arg(short = 'p', long = "port")]
    port: String,

    #[arg(short = 'r', long = "baud")]
    baud: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut port = serialport::new(&args.port, args.baud)
        .timeout(Duration::from_millis(500)) // Increased timeout for more reliable reads
        .open()?;

    std::thread::sleep(Duration::from_secs(1)); // Delay for Arduino reset

    println!("{color_green}📡 Comchan connected to {} at {} baud{color_reset}", args.port, args.baud);
    println!("{color_green}🔄 Listening... (Ctrl+C to exit){color_reset}\n");

    let mut buffer = [0u8; 1024];
    let mut input = String::new();
    let mut received = String::new(); // Accumulate serial data
    let stdin = io::stdin();

    loop {
        // Read from serial port
        match port.read(&mut buffer) {
            Ok(n) => {
                if n > 0 {
                    let output = String::from_utf8_lossy(&buffer[..n]);
                    received.push_str(&output); // Accumulate output
                    // Print complete lines
                    while let Some(line_end) = received.find('\n') {
                        let line = received.drain(..=line_end).collect::<String>();
                        print!("📥 {}", line);
                        io::stdout().flush()?;
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}, // Timeout
            Err(e) => eprintln!("{color_red}❌ Serial read error: {e}{color_reset}"),
        }

        // Read from stdin and write to serial port
        input.clear();
        if stdin.read_line(&mut input).is_ok() && !input.trim_end().is_empty() {
            let clean = input.trim_end();
            //println!("SENDING: `{}`", clean);
            if let Err(e) = port.write_all(format!("{}\n", clean).as_bytes()) {
                eprintln!("{color_red}❌ Write error: {e}{color_reset}");
                continue;
            }
            if let Err(e) = port.flush() {
                eprintln!("{color_red}❌ Flush error: {e}{color_reset}");
                continue;
            }
            // Small delay to allow Arduino to process
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}
