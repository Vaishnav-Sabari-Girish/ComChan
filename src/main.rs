use clap::Parser;
use serialport;
use std::io::{self, Read, Write};
use std::time::Duration;

#[derive(Parser)]
struct Args {
    #[arg(short = 'p', long = "port")]
    port: String,

    #[arg(short = 'r', long = "baud")]
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
                continue;
            }
            Err(e) => {
                eprintln!("❌ Error: {e}");
                return Err(Box::new(e)); // graceful exit
            }
            _ => {}
        }
    }
}
