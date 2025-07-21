use serialport;
/// Find the first available USB serial port
pub fn find_usb_port() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let ports = serialport::available_ports()?;

    for port in ports {
        if let serialport::SerialPortType::UsbPort(_) = port.port_type {
            return Ok(Some(port.port_name));
        }
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
