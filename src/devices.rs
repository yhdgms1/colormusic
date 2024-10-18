use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host};

use super::config::Settings;

pub fn get_device(host: &Host, settings: &Settings) -> Option<Device> {
    let mut device: Option<Device> = None;

    if let Some(devices) = &settings.devices {
        let output_devices = host
            .output_devices()
            .expect("Не удалось получить устройства вывода");

        for output_device in output_devices {
            for name in devices {
                let device_name = output_device
                    .name()
                    .expect("Не удалось получить имя устройства вывода.");

                if device_name == *name {
                    device = Some(output_device);
                    break;
                }
            }
        }
    } else {
        println!("Используется устройство вывода по умолчанию.");

        return Some(
            host.default_output_device()
                .expect("Не удалось получить устройство вывода по умолчанию."),
        );
    }

    return device;
}
