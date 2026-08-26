use serialport::SerialPortType;
/// Find the first available USB serial port
use std::path::Path;

#[derive(Debug, Clone)]
pub struct UsbIdentity {
    pub vid: u16,
    pub pid: u16,
    pub serial_number: Option<String>,
}

pub fn identity_for_port(
    port_name: &str,
) -> Result<Option<UsbIdentity>, Box<dyn std::error::Error>> {
    for port in serialport::available_ports()? {
        if port.port_name != port_name {
            continue;
        }

        if let SerialPortType::UsbPort(info) = port.port_type {
            return Ok(Some(UsbIdentity {
                vid: info.vid,
                pid: info.pid,
                serial_number: info.serial_number,
            }));
        }
    }

    Ok(None)
}

pub fn find_usb_port() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let ports = serialport::available_ports()?;
    for port in ports {
        if let serialport::SerialPortType::UsbPort(_) = port.port_type {
            return Ok(Some(port.port_name));
        }
    }
    Ok(None)
}

pub fn resolve_port_after_detach(
    previous: &str,
    used_auto: bool,
    skip_serial: bool,
    previous_id: Option<&UsbIdentity>,
) -> Result<String, Box<dyn std::error::Error>> {
    if skip_serial || previous == "BLE_STREAM" {
        return Ok(previous.to_string());
    }

    let path_gone = !Path::new(previous).exists();

    if used_auto {
        match find_usb_port()? {
            Some(p) if p != previous => {
                println!("[ComChan] port re-enumerated: {previous} -> {p}");
                Ok(p)
            }
            Some(p) => Ok(p),
            None => {
                println!(
                    "[ComChan] no USB serial yet (old path {previous} is gone); will retry on open"
                );
                Ok(previous.to_string())
            }
        }
    } else if path_gone {
        if let Some(p) = find_port_matching_previous(previous_id)? {
            if p != previous {
                println!("[ComChan] recovered device at new path: {previous} -> {p}");
            }
            Ok(p)
        } else {
            println!("[ComChan] waiting for {previous} (not attaching to a different USB serial)");
            Ok(previous.to_string())
        }
    } else {
        Ok(previous.to_string())
    }
}

fn find_port_matching_previous(
    previous_id: Option<&UsbIdentity>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let Some(id) = previous_id else {
        return Ok(None);
    };

    let mut vid_pid_matches: Vec<String> = Vec::new();

    for port in serialport::available_ports()? {
        let SerialPortType::UsbPort(info) = port.port_type else {
            continue;
        };

        if info.vid != id.vid || info.pid != id.pid {
            continue;
        }

        match (&id.serial_number, &info.serial_number) {
            (Some(want), Some(got)) if want == got => {
                return Ok(Some(port.port_name));
            }
            (Some(_), Some(_)) => {}
            _ => {
                vid_pid_matches.push(port.port_name);
            }
        }
    }

    if vid_pid_matches.len() == 1 {
        return Ok(Some(vid_pid_matches.remove(0)));
    }

    Ok(None)
}

/// Display detailed USB port information
pub fn show_usb_ports() -> Result<(), Box<dyn std::error::Error>> {
    let ports = serialport::available_ports()?;
    println!("🔍 USB Serial Ports:");
    let mut found_usb = false;
    for port in ports {
        if let serialport::SerialPortType::UsbPort(ref info) = port.port_type {
            found_usb = true;
            println!("  📱 Port: {}", port.port_name);
            println!("     USB VID: {:04x}, PID: {:04x}", info.vid, info.pid);
            if let Some(manufacturer) = &info.manufacturer {
                println!("     Manufacturer: {}", manufacturer);
            }
            if let Some(product) = &info.product {
                println!("     Product: {}", product);
            }
            if let Some(serial) = &info.serial_number {
                println!("     Serial: {}", serial);
            }
            println!();
        }
    }
    if !found_usb {
        println!("  ⚠️  No USB serial ports found");
    }
    Ok(())
}
