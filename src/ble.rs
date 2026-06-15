#![cfg(feature = "ble")]

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use inquire::Select;
use std::error::Error;
use std::sync::mpsc;
use std::time::Duration;
use uuid::Uuid;

//const NUS_SERVICE_UUID: Uuid = Uuid::from_u128(0x6e400001_b5a3_f393_e0a9_e50e24dcca9e);
const NUS_TX_CHAR_UUID: Uuid = Uuid::from_u128(0x6e400003_b5a3_f393_e0a9_e50e24dcca9e);

#[derive(Debug, Clone)]
pub enum BleEvent {
    Payload(String),
    Disconnected,
}

/// Starts the BLE interactive menu and background streaming task.
/// Returns the Tokio runtime so the caller can keep it alive for the duration of the app.
pub fn start_ble_stream(
    tx: mpsc::Sender<BleEvent>,
) -> Result<tokio::runtime::Runtime, Box<dyn Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // Block the main thread while the interactive setup happens first
    rt.block_on(async {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().next().ok_or("No Bluetooth Adapters found on this system")?;

        println!("Scanning for BLE devices for 4 seconds...");
        central.start_scan(ScanFilter::default()).await?;
        tokio::time::sleep(Duration::from_secs(4)).await;

        let peripherals = central.peripherals().await?;
        if peripherals.is_empty() {
            return Err("No BLE devices found in range".into());
        }

        let mut device_map = Vec::new();
        let mut display_list = Vec::new();

        for p in peripherals {
            let properties = p.properties().await?.unwrap_or_default();
            let name = properties.local_name.unwrap_or_else(|| "Unknown Device".to_string());
            let mac = p.id().to_string();

            display_list.push(format!("[{}] {}", mac, name));
            device_map.push(p);
        }

        let selection = Select::new("Select Target BLE device: ", display_list.clone())
            .with_page_size(10)
            .prompt()?;

        let index = display_list.iter().position(|r| r == &selection).unwrap();
        let peripheral = device_map.remove(index);

        println!("Connecting to {}...", selection);
        peripheral.connect().await?;
        println!("Connected! Discovering services....");
        peripheral.discover_services().await?;

        // Hunt down the NUS TX characteristic 
        let chars = peripheral.characteristics();
        let tx_char = chars
            .iter()
            .find(|c| c.uuid == NUS_TX_CHAR_UUID)
            .ok_or("Nordic UART Service (NUS) TX characteristic not found! Ensure your board is flashing the correct profile.")?;

        println!("Subscribing to the NUS TX stream....");
        peripheral.subscribe(tx_char).await?;

        let mut notification_stream = peripheral.notifications().await?;

        println!("Stream Established! Handing over to ComChan Monitor...\n");

        tokio::spawn(async move {
            while let Some(data) = notification_stream.next().await {
                if data.uuid == NUS_TX_CHAR_UUID {
                    let text = String::from_utf8_lossy(&data.value).to_string();

                    if tx.send(BleEvent::Payload(text)).is_err() {
                        break;
                    }
                }
            }

            eprintln!("BLE notification stream disconnected unexpectedly.");
            let _ = tx.send(BleEvent::Disconnected);
        });

        Ok::<(), Box<dyn Error>>(())
    })?;

    Ok(rt)
}
