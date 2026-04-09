use clap::Parser;
use inline_colorization::*;
use serialport;

mod config;
mod monitor;
mod parser;
mod plotter;
mod port_finder;
mod serial;

use config::{Args, generate_default_config, load_config, merge_config_and_args};
use monitor::run_normal_mode;
use plotter::run_plotter_mode;

fn list_available_ports() -> Result<(), Box<dyn std::error::Error>> {
    println!("{color_cyan}󰅍 All Available Serial Ports:{color_reset}");
    let ports = serialport::available_ports()?;

    if ports.is_empty() {
        println!("  {color_yellow}  No serial ports found{color_reset}");
        return Ok(());
    }

    for port in ports {
        let type_str = match port.port_type {
            serialport::SerialPortType::UsbPort(info) => {
                format!("USB (VID: {:04x}, PID: {:04x})", info.vid, info.pid)
            }
            serialport::SerialPortType::BluetoothPort => "Bluetooth".to_string(),
            serialport::SerialPortType::PciPort => "PCI".to_string(),
            serialport::SerialPortType::Unknown => "Unknown".to_string(),
        };
        println!("   {} — {}", port.port_name, type_str);
    }

    println!();
    port_finder::show_usb_ports()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.generate_config {
        return generate_default_config(args.config_file);
    }

    let config = load_config(args.config_file.clone())?;
    let merged = merge_config_and_args(config, args);

    if merged.list_ports {
        return list_available_ports();
    }

    let port_name = match &merged.port {
        Some(p) if p.to_lowercase() == "auto" => match port_finder::find_usb_port()? {
            Some(detected) => {
                println!(
                    "{color_green} Auto-detected USB port: {}{color_reset}",
                    detected
                );
                detected
            }
            None => {
                eprintln!(
                    "{color_red}❌ No USB serial ports found for auto-detection{color_reset}"
                );
                eprintln!("{color_yellow}󰌵 Try --list-ports to see available ports{color_reset}");
                std::process::exit(1);
            }
        },
        Some(p) => p.clone(),
        None => {
            eprintln!(
                "{color_red}❌ No port specified. Use --port <PORT>, --auto, or set port in config{color_reset}"
            );
            eprintln!("{color_yellow}󰌵 Try --list-ports or --generate-config{color_reset}");
            std::process::exit(1);
        }
    };

    if merged.plot {
        run_plotter_mode(merged, port_name)
    } else {
        run_normal_mode(merged, port_name)
    }
}
