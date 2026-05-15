use chrono::NaiveTime;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};

pub enum ReplayEvent {
    Payload(String),
    Waiting,
    Eof,
}

pub struct SessionReplayer {
    reader: BufReader<File>,
    last_time: Option<NaiveTime>,
    is_csv: bool,
    queued_event: Option<(String, Instant)>,
}

impl SessionReplayer {
    pub fn new(filepath: &str) -> std::io::Result<Self> {
        let file = File::open(filepath)?;
        let is_csv: bool = filepath.to_lowercase().ends_with(".csv");
        let mut reader = BufReader::new(file);

        if is_csv {
            let mut header = String::new();
            let _ = reader.read_line(&mut header);
        }

        Ok(Self {
            reader,
            last_time: None,
            is_csv,
            queued_event: None,
        })
    }

    pub fn next_payload(&mut self) -> ReplayEvent {
        // Check if line is waiting to be printed
        if let Some((payload, emit_at)) = &self.queued_event {
            if Instant::now() >= *emit_at {
                let p = payload.clone();
                self.queued_event = None;
                return ReplayEvent::Payload(p);
            } else {
                return ReplayEvent::Waiting;
            }
        }

        let mut line = String::new();

        while self.reader.read_line(&mut line).unwrap_or(0) > 0 {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                line.clear();
                continue;
            }

            let (time_str, payload) = if self.is_csv {
                let parts: Vec<&str> = trimmed.splitn(2, ',').collect();
                if parts.len() < 2 {
                    line.clear();
                    continue;
                }
                (parts[0].trim(), parts[1].trim())
            } else {
                if !trimmed.starts_with("RX [") {
                    line.clear();
                    continue;
                }

                let end_bracket = match trimmed.find("]: ") {
                    Some(idx) => idx,
                    None => {
                        line.clear();
                        continue;
                    }
                };
                (&trimmed[4..end_bracket], &trimmed[end_bracket + 3..])
            };

            let mut delay_ms = 0;
            if let Ok(current_time) = NaiveTime::parse_from_str(time_str, "%H:%M:%S%.3f") {
                if let Some(last) = self.last_time {
                    let mut ms = current_time.signed_duration_since(last).num_milliseconds();
                    if ms < 0 {
                        ms += 24 * 60 * 60 * 1000;
                    }
                    if ms > 0 {
                        delay_ms = ms.min(5000) as u64;
                    }
                }
                self.last_time = Some(current_time);
            }

            if delay_ms > 0 {
                let emit_time = Instant::now() + Duration::from_millis(delay_ms);
                self.queued_event = Some((payload.to_string(), emit_time));
                return ReplayEvent::Waiting;
            } else {
                return ReplayEvent::Payload(payload.to_string());
            }
        }

        ReplayEvent::Eof
    }
}
