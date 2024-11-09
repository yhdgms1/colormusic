use std::time::Duration;

use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Manager, Peripheral};
use tokio::time::sleep;
use uuid::Uuid;

const SERVICE_UUID: Uuid = Uuid::from_u128(0x0000fff0_0000_1000_8000_00805f9b34fb);
const CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x0000fff3_0000_1000_8000_00805f9b34fb);

pub async fn get_device() -> Result<Option<(Peripheral, Characteristic)>, Box<dyn std::error::Error>> {
    let manager = Manager::new().await?;

    let adapters = manager.adapters().await?;
    let adapter = adapters.into_iter().nth(0).expect("Адаптер Bluetooth не найден.");

    adapter.start_scan(ScanFilter::default()).await?;

    println!("Поиск устройств Bluetooth...");
    sleep(Duration::from_secs(2)).await;

    let devices = adapter.peripherals().await?;

    for device in devices.iter() {
        if let Some(properties) = device.properties().await? {
            if properties.services.contains(&SERVICE_UUID) {
                if let Some(name) = properties.local_name {
                    println!("Обнаружено устройство: {name}");

                    device.connect().await?;
                    device.discover_services().await?;

                    if let Some(characteristic) = device.characteristics().iter().find(|c| c.uuid == CHARACTERISTIC_UUID) {
                        return Ok(Some((device.to_owned(), characteristic.to_owned())));
                    } else {
                        println!("Требуемая характеристика устройства не обнаружена.");
                    }
                }
            }
        }
    }

    Ok(None)
}