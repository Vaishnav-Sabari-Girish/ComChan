use clap::Parser;
use serialport;
use std::io::{self, Read, Write};
use std::time::Duration;

#[derive(Parser)]
struct Args {
    port: String,
    baud: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::parse();

    let mut port = serialport::new(&args.port, args.baud)
        .timeout(Duration::from_millis(100))
        .open()?;

    println!("📡 Comchan connected to {} at {} baud", args.port, args.baud);
    println!("🔄 Listening... (Ctrl+C to exit)\n");

    let mut buffer = [0u8; 1024];

    loop {
        match port.read(&mut buffer) {
            Ok(n) if n > 0 => {
                let output  = String::from_utf8_lossy(&buffer[..n]);
                print!("{output}");
                io::stdout().flush().ok();
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                eprintln!("❌ Error: {e}");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
