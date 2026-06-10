use defmt_decoder::{DecodeError, StreamDecoder, Table};
use object::{Object, ObjectSymbol};
use probe_rs::{
    Permissions, Session,
    config::TargetSelector,
    probe::list::Lister,
    rtt::{Rtt, ScanRegion},
};
use std::fs;

pub struct RttDefmtReader {
    session: Session,
    rtt: Rtt,
    stream_decoder: Box<dyn StreamDecoder>,
}

impl RttDefmtReader {
    pub fn new(
        elf_path: &str,
        chip_override: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let elf_bytes = fs::read(elf_path)?;
        let elf_bytes_ref: &'static [u8] = Box::leak(elf_bytes.into_boxed_slice());

        let table = Table::parse(elf_bytes_ref)?
            .ok_or("No defmt table found. Is the firmware compiled with defmt?")?;

        let table_ref: &'static Table = Box::leak(Box::new(table));
        let stream_decoder = table_ref.new_stream_decoder();

        // ── INSTANT ATTACH FIX ──
        // Instead of scanning all of RAM, we parse the ELF file to find the exact
        // memory address of the RTT control block. This reduces attach time from 3s to 0.01s.
        let rtt_addr = {
            let obj_file = object::File::parse(elf_bytes_ref)?;
            obj_file
                .symbols()
                .find(|sym| sym.name() == Ok("_SEGGER_RTT"))
                .map(|sym| sym.address() as u32)
        };

        let lister = Lister::new();
        let probes = lister.list_all();

        if probes.is_empty() {
            return Err("No debug probes detected. Check your USB connection.".into());
        }

        if probes.len() > 1 {
            eprintln!("⚠️ Warning: Multiple debug probes detected!");
            eprintln!(
                "⚠️ Silently attaching to the first enumerated probe: {}",
                probes[0].identifier
            );
        }

        let probe = probes[0].open()?;

        let target_selector = match chip_override {
            Some(chip) => TargetSelector::Unspecified(chip),
            None => TargetSelector::Auto,
        };

        let mut session = probe.attach(target_selector, Permissions::default()).map_err(|e| {
            format!(
                "Failed to attach: {}\nHint: Try specifying the chip manually using --chip (e.g., --chip nRF52840_xxAA)",
                e
            )
        })?;

        let rtt = {
            let mut core = session.core(0)?;

            if let Some(addr) = rtt_addr {
                // Fast path: We know exactly where it is!
                Rtt::attach_region(&mut core, &ScanRegion::Exact(addr as u64))?
            } else {
                // Slow path fallback: Scan RAM, taking the highest address to avoid Flash mirrors
                match Rtt::attach_region(&mut core, &ScanRegion::Ram) {
                    Ok(rtt) => rtt,
                    Err(probe_rs::rtt::Error::MultipleControlBlocksFound(addrs)) => {
                        let active_addr = addrs.into_iter().max().expect("Address list empty");
                        Rtt::attach_region(&mut core, &ScanRegion::Exact(active_addr))?
                    }
                    Err(e) => return Err(e.into()),
                }
            }
        };

        Ok(Self {
            session,
            rtt,
            stream_decoder,
        })
    }

    pub fn poll_logs(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut logs = Vec::new();
        let mut core = self.session.core(0)?;

        if let Some(channel) = self.rtt.up_channels().first_mut() {
            let mut buf = [0u8; 1024];
            let bytes_read = channel.read(&mut core, &mut buf)?;

            if bytes_read > 0 {
                self.stream_decoder.received(&buf[..bytes_read]);

                loop {
                    match self.stream_decoder.decode() {
                        Ok(frame) => {
                            let timestamp = frame
                                .display_timestamp()
                                .map(|t| t.to_string())
                                .unwrap_or_default();

                            let raw_level = frame
                                .level()
                                .map(|l| l.as_str().to_uppercase())
                                .unwrap_or_else(|| "UNK".to_string());
                            let padded_level = format!("{:5}", raw_level);

                            // Apply standard terminal ANSI color codes
                            let colored_level = match raw_level.as_str() {
                                "ERROR" => format!("\x1b[31;1m{}\x1b[0m", padded_level), // Bold Red
                                "WARN" => format!("\x1b[33;1m{}\x1b[0m", padded_level), // Bold Yellow
                                "INFO" => format!("\x1b[32m{}\x1b[0m", padded_level),   // Green
                                "DEBUG" => format!("\x1b[36m{}\x1b[0m", padded_level),  // Cyan
                                "TRACE" => format!("\x1b[90m{}\x1b[0m", padded_level),  // Gray/Dim
                                _ => padded_level,
                            };

                            let message = frame.display_message().to_string();

                            let formatted = if timestamp.is_empty() {
                                format!("[{}] {}", colored_level, message)
                            } else {
                                format!("{} [{}] {}", timestamp, colored_level, message)
                            };

                            logs.push(formatted);
                        }
                        // SILENTLY handle partial frames. Do not print "Unexpected EOF" here!
                        Err(DecodeError::UnexpectedEof) => break,
                        Err(DecodeError::Malformed) => {
                            eprintln!("Warning: Malformed defmt frame detected.");
                            break;
                        }
                    }
                }
            }
        }

        Ok(logs)
    }
}
