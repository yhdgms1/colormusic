use crate::devices::get_device;
use crate::events::Event;

use cpal::traits::{DeviceTrait, StreamTrait};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering}, mpsc::Sender, Arc
    },
    thread,
    time::Duration,
};

pub fn listen_for_audio(devices: Option<Vec<String>>, tx: Sender<Event>) {
    thread::spawn(move || {
        let host = cpal::default_host();

        let sample_rate = Arc::new(AtomicU32::new(48000));
        let restart = Arc::new(AtomicBool::new(false));
    
        loop {
            let Some(device) = get_device(&host, &devices) else {
                println!("Не найдено ни одного устройства вывода. Ожидание устройства...");
    
                std::thread::sleep(Duration::from_secs(5));
    
                continue;
            };
    
            let device_name = device
                .name()
                .expect("Не удалось получить имя устройства вывода.");
    
            println!("Используемое устройство вывода: {}", device_name);
    
            let config = device
                .default_output_config()
                .expect("Не удалось получить настройку вывода по умолчанию.")
                .config();
    
            let sample_rate_reader = Arc::clone(&sample_rate);
            let restart_writer = Arc::clone(&restart);
    
            sample_rate.store(config.sample_rate.0, Ordering::Relaxed);
    
            let tx = tx.clone();
    
            let stream = device
                .build_input_stream(
                    &config,
                    move |data: &[f32], _| {
                        let tx = tx.clone();
                        let sr = sample_rate_reader.load(Ordering::Relaxed);
                        
                        _ = tx.send(Event::Audio((Vec::from(data), sr)));
                    },
                    move |error| {
                        if matches!(error, cpal::StreamError::DeviceNotAvailable) {
                            println!("Запрошенное устройство больше недоступно.");
                        } else {
                            println!("{}", error);
                        }
    
                        restart_writer.store(true, Ordering::SeqCst);
                    },
                    None,
                )
                .expect("Не удалось создать входной поток.");
    
            stream.play().expect("Не удалось воспроизвести поток.");
    
            while !restart.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(500));
            }
    
            restart.store(false, Ordering::SeqCst);
        }
    });
}
