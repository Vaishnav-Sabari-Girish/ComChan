use serialport::{DataBits, FlowControl, Parity, StopBits};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn parse_data_bits(bits: u8) -> Result<DataBits, String> {
    match bits {
        5 => Ok(DataBits::Five),
        6 => Ok(DataBits::Six),
        7 => Ok(DataBits::Seven),
        8 => Ok(DataBits::Eight),
        _ => Err(format!(
            "Invalid data bits: {}. Must be 5, 6, 7, or 8",
            bits
        )),
    }
}

pub fn parse_stop_bits(bits: u8) -> Result<StopBits, String> {
    match bits {
        1 => Ok(StopBits::One),
        2 => Ok(StopBits::Two),
        _ => Err(format!("Invalid stop bits: {}. Must be 1 or 2", bits)),
    }
}

pub fn parse_parity(parity: &str) -> Result<Parity, String> {
    match parity.to_lowercase().as_str() {
        "none" | "n" => Ok(Parity::None),
        "odd" | "o" => Ok(Parity::Odd),
        "even" | "e" => Ok(Parity::Even),
        _ => Err(format!(
            "Invalid parity: {}. Must be 'none', 'odd', or 'even'",
            parity
        )),
    }
}

pub fn parse_flow_control(flow: &str) -> Result<FlowControl, String> {
    match flow.to_lowercase().as_str() {
        "none" | "n" => Ok(FlowControl::None),
        "software" | "s" => Ok(FlowControl::Software),
        "hardware" | "h" => Ok(FlowControl::Hardware),
        _ => Err(format!(
            "Invalid flow control: '{}'. Must be 'none', 'software', or 'hardware'",
            flow
        )),
    }
}

pub fn get_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let secs = now / 1000;
    let millis = now % 1000;
    format!("{}.{:03}", secs, millis)
}
