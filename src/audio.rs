use cpal::{traits::{DeviceTrait, HostTrait}, Device, StreamConfig};

pub fn get_output() -> (Device, StreamConfig) {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Устройство для вывода звука не обнаружено.");

    let device_name = device
        .name()
        .unwrap_or("unknown".to_string());

    println!("Используемое устройство вывода: {}", device_name);

    let config = device
        .default_output_config()
        .expect("Ошибка при получении формата выходного потока для устройства.")
        .config();

    return (device, config)
}