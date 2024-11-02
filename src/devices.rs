use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host};

pub fn get_device(host: &Host, devices: &Option<Vec<String>>) -> Option<Device> {
    if let Some(devices) = devices {
        if devices.contains(&"default".to_string()) {
            return host.default_output_device();
        }

        let output_devices = host
            .output_devices()
            .expect("Не удалось получить устройства вывода");

        for output_device in output_devices {
            let device_name = output_device
                .name()
                .expect("Не удалось получить имя устройства вывода.");

            if devices.iter().any(|name| &device_name == name) {
                return Some(output_device);
            }
        }

        None
    } else {
        println!("Используется устройство вывода по умолчанию.");

        host.default_output_device()
    }
}
