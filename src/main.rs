use clap::Parser;
use inline_colorization::*;

mod config;
mod export;
mod monitor;
mod parser;
mod plotter;
mod port_finder;
mod replay;
mod rtt_reader;
mod serial;

#[cfg(feature = "ble")]
mod ble;

use config::{
    Args, generate_default_config, load_config, merge_config_and_args, print_completions,
};

pub enum AppExitState {
    Quit,
    SwitchToPlotter {
        port: Option<Box<dyn serialport::SerialPort>>,
        rtt_reader: Option<crate::rtt_reader::RttDefmtReader>,
        #[cfg(feature = "ble")]
        ble_rx: Option<std::sync::mpsc::Receiver<crate::ble::BleEvent>>,
    },
    SwitchToMonitor {
        port: Option<Box<dyn serialport::SerialPort>>,
        rtt_reader: Option<crate::rtt_reader::RttDefmtReader>,
        #[cfg(feature = "ble")]
        ble_rx: Option<std::sync::mpsc::Receiver<crate::ble::BleEvent>>,
    },
}

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

    if let Some(shell) = args.completions {
        print_completions(shell);
        return Ok(());
    }

    if args.generate_config {
        return generate_default_config(args.config_file);
    }

    let config = load_config(args.config_file.clone())?;
    let merged = merge_config_and_args(config, args);

    if merged.list_ports {
        return list_available_ports();
    }

    let port_name = if merged.simulate || merged.replay_file.is_some() || merged.rtt || merged.ble {
        if merged.rtt {
            println!("{color_magenta}Starting in RTT/DEFMT debug probe mode....{color_reset}");
            "RTT_DEBUG_PROBE".to_string()
        } else if merged.ble {
            println!("{color_magenta}Starting in BLE stream mode....{color_reset}");
            "BLE_STREAM".to_string()
        } else {
            println!("{color_magenta}Starting in SIMULATE/REPLAY mode....{color_reset}");
            "SIMULATE_PORT".to_string()
        }
    } else {
        match &merged.port {
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
                        "{color_red}No USB serial ports found for auto-detection{color_reset}"
                    );
                    eprintln!("{color_yellow}Try --list-ports to see available ports{color_reset}");
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
        }
    };

    let mut is_plot_mode = merged.plot;
    let mut active_port: Option<Box<dyn serialport::SerialPort>> = None;
    let mut active_rtt: Option<crate::rtt_reader::RttDefmtReader> = None;

    #[cfg(feature = "ble")]
    let mut active_ble_rx: Option<std::sync::mpsc::Receiver<crate::ble::BleEvent>> = None;

    #[cfg(feature = "ble")]
    let _ble_rt = if merged.ble {
        let (tx, rx) = std::sync::mpsc::channel();
        let rt = ble::start_ble_stream(tx).unwrap_or_else(|e| {
            eprintln!("{color_red}BLE Error: {}{color_reset}", e);
            std::process::exit(1);
        });
        active_ble_rx = Some(rx);
        Some(rt)
    } else {
        None
    };

    loop {
        let result = if is_plot_mode {
            #[cfg(feature = "ble")]
            let res = crate::plotter::run_plotter_mode(
                merged.clone(),
                port_name.clone(),
                active_port,
                active_rtt,
                active_ble_rx,
            );
            #[cfg(not(feature = "ble"))]
            let res = crate::plotter::run_plotter_mode(
                merged.clone(),
                port_name.clone(),
                active_port,
                active_rtt,
            );
            res
        } else {
            #[cfg(feature = "ble")]
            let res = crate::monitor::run_normal_mode(
                merged.clone(),
                port_name.clone(),
                active_port,
                active_rtt,
                active_ble_rx,
            );
            #[cfg(not(feature = "ble"))]
            let res = crate::monitor::run_normal_mode(
                merged.clone(),
                port_name.clone(),
                active_port,
                active_rtt,
            );
            res
        };

        match result {
            Ok(AppExitState::Quit) => break,
            Ok(AppExitState::SwitchToPlotter {
                port,
                rtt_reader,
                #[cfg(feature = "ble")]
                ble_rx,
            }) => {
                is_plot_mode = true;
                active_port = port;
                active_rtt = rtt_reader;
                #[cfg(feature = "ble")]
                {
                    active_ble_rx = ble_rx;
                }
            }
            Ok(AppExitState::SwitchToMonitor {
                port,
                rtt_reader,
                #[cfg(feature = "ble")]
                ble_rx,
            }) => {
                is_plot_mode = false;
                active_port = port;
                active_rtt = rtt_reader;
                #[cfg(feature = "ble")]
                {
                    active_ble_rx = ble_rx;
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
