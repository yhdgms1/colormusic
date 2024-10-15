mod colorizer;
mod config;
mod devices;
mod math;
mod splitter;
mod colors;

use colorizer::frequencies_to_color;
use devices::get_device;
use splitter::split_into_frequencies;
use colors::{Colors, Interpolator};

use std::net::TcpListener;
use std::io::{Read, Write};
use std::thread;
use cpal::traits::{DeviceTrait, StreamTrait};
use palette::{FromColor, Srgb};
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const COLOR_CHANGE_DURATION: Duration = Duration::from_millis(50);

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8043").expect("Не удалось привязаться к прослушивателю TCP");

    let colors = Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Colors::new()))));
    let colors_reader = Arc::clone(&colors);

    let mut interpolator = Interpolator::new();
    let mut instant = Instant::now();

    thread::spawn(move || {
        let host = cpal::default_host();
        let sample_rate = Arc::new(AtomicU32::new(1));
        let sample_rate_clone = Arc::clone(&sample_rate);

        let data_callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if instant.elapsed() < COLOR_CHANGE_DURATION {
                return;
            }

            let (low, mid, high) = split_into_frequencies(data, sample_rate.load(Ordering::SeqCst));
            let color = frequencies_to_color(low, mid, high);

            let colors = unsafe {
                colors.load(Ordering::SeqCst).as_mut().unwrap()
            };

            colors.update_current(color);
            instant = Instant::now();
        };

        let data_callback = Arc::new(AtomicPtr::new(Box::into_raw(Box::new(data_callback))));

        let restart = Arc::new(AtomicBool::new(false));
        let restart_clone = Arc::clone(&restart);
        

        loop {
            let Some(device) = get_device(&host) else {
                println!("Не найдено ни одного устройства вывода. Ожидание устройства...");

                std::thread::sleep(Duration::from_secs(5));

                continue;
            };

            let device_name = device.name().expect("Не удалось получить имя устройства вывода.");

            println!("Используемое устройство вывода: {}", device_name);
        
            let config = device
                .default_output_config()
                .expect("Не удалось получить настройку вывода по умолчанию.")
                .config();

            sample_rate_clone.store(config.sample_rate.0, Ordering::SeqCst);
        
            let data_callback_clone = Arc::clone(&data_callback);
            let restart_setter = Arc::clone(&restart_clone);

            let stream = device
                .build_input_stream(
                    &config,
                    move |data, info| {
                        let callback = unsafe {
                            data_callback_clone.load(Ordering::SeqCst).as_mut().unwrap()
                        };
                        
                        callback(data, info);
                    },
                    move |error| {
                        *&restart_setter.store(true, Ordering::SeqCst);

                        match error {
                            cpal::StreamError::DeviceNotAvailable => {
                                println!("Запрошенное устройство больше недоступно.");
                            },
                            _ => println!("{}", error)
                        };
                    },
                    None,
                )
                .expect("Не удалось создать входной поток.");
            
            stream.play().expect("Не удалось воспроизвести поток.");

            'inner: loop {
                if restart_clone.load(Ordering::SeqCst) {
                    restart_clone.store(false, Ordering::SeqCst);
                    break 'inner;
                }

                thread::sleep(Duration::from_millis(500));
            }
        }
    });

    // Без чтения не получается отправить ответ
    let mut read_buffer = [0; 512];

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            if stream.read(&mut read_buffer).is_ok() {
                let elapsed = instant.elapsed();
                let t = (elapsed.as_secs_f32() / COLOR_CHANGE_DURATION.as_secs_f32()).min(1.0);
    
                let colors = unsafe {
                    colors_reader.load(Ordering::SeqCst).as_mut().unwrap()
                };
    
                let color = interpolator.interpolate(colors, t);
            
                let color: Srgb<u8> = Srgb::from_color(*color).into();
                let payload = format!("HTTP/1.1 201 OK\r\nContent-Type: text/plain\r\nAccess-Control-Allow-Origin: *\r\n\r\n{} {} {}", color.red, color.green, color.blue);
    
                if stream.write(payload.as_bytes()).is_ok() {
                    _ = stream.flush();
                }
            }
        }
    }
}
