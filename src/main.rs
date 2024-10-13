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
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::SampleRate;
use palette::{FromColor, Srgb};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const COLOR_CHANGE_DURATION: Duration = Duration::from_millis(50);

fn main() {
    let host = cpal::default_host();

    let Some(device) = get_device(&host) else {
        panic!("Не найдено ни одного устройства вывода.");
    };

    println!(
        "Используемое устройство вывода: {}",
        device.name().expect("Не удалось получить имя устройства вывода.")
    );

    let listener = TcpListener::bind("0.0.0.0:8043").expect("Не удалось привязаться к прослушивателю TCP");

    let colors = Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Colors::new()))));
    let colors_reader = Arc::clone(&colors);

    let mut interpolator = Interpolator::new();

    let mut instant = Instant::now();

    let config = device
        .default_output_config()
        .expect("Не удалось получить настройку вывода по умолчанию.")
        .config();

    let SampleRate(sample_rate) = config.sample_rate;

    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let elapsed = instant.elapsed();

                if elapsed >= COLOR_CHANGE_DURATION {
                    let (low, mid, high) = split_into_frequencies(data, sample_rate);
                    let color = frequencies_to_color(low, mid, high);

                    let colors = unsafe {
                        colors.load(Ordering::SeqCst).as_mut().unwrap()
                    };

                    colors.update_current(color);

                    instant = Instant::now();
                }
            },
            move |error| {
                eprintln!("{}", error);

                std::process::exit(1);
            },
            Some(Duration::from_secs(1)),
        )
        .expect("Не удалось создать входной поток.");

    stream.play().expect("Не удалось воспроизвести поток.");

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
