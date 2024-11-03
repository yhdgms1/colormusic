mod colorizer;
mod colors;
mod config;
mod devices;
mod math;
mod splitter;

use colorizer::frequencies_to_color;
use colors::{Colors, Interpolator};
use config::Settings;
use devices::get_device;
use splitter::split_into_frequencies;

use cpal::traits::{DeviceTrait, StreamTrait};
use palette::{FromColor, Oklch, Srgb};
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const COLOR_CHANGE_DURATION: Duration = Duration::from_millis(50);
const DEFAULT_UDP_ADDRESS: &str = "192.168.1.167:8488";

enum Mode {
    Colormusic,
    Static,
}

fn main() {
    let Settings {
        tcp,
        tcp_port,
        udp,
        udp_address,
        udp_port,
        devices,
    } = config::get_config();

    let tcp = tcp.unwrap_or(false);
    let udp = udp.unwrap_or(true);
    let udp_address = udp_address.unwrap_or(DEFAULT_UDP_ADDRESS.to_string());

    let tcp_port = tcp_port.unwrap_or("8043".to_string());
    let udp_port = udp_port.unwrap_or("8044".to_string());

    let listener = TcpListener::bind(format!("0.0.0.0:{}", tcp_port))
        .expect("Не удалось создать слушатель TCP");
    let socket =
        UdpSocket::bind(format!("0.0.0.0:{}", udp_port)).expect("Не удалось создать Udp сокет");

    let colors = Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Colors::new()))));
    let colors_setter = Arc::clone(&colors);
    let colors_reader_tcp = Arc::clone(&colors);
    let colors_reader_udp = Arc::clone(&colors);

    let mode = Arc::new(Mutex::new(Mode::Colormusic));
    let mode_setter = Arc::clone(&mode);

    let instant = Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Instant::now()))));
    let instant_clone_tcp = Arc::clone(&instant);
    let instant_clone_udp = Arc::clone(&instant);

    thread::spawn(move || {
        let host = cpal::default_host();
        let sample_rate = Arc::new(AtomicU32::new(1));
        let sample_rate_clone = Arc::clone(&sample_rate);

        let data_callback = move |data: &[f32]| {
            let elapsed = unsafe {
                instant.load(Ordering::Relaxed).as_mut().unwrap().elapsed()
            };

            if elapsed < COLOR_CHANGE_DURATION {
                return;
            }

            if let Mode::Colormusic = *mode.lock().unwrap() {
                let (low, mid, high) =
                    split_into_frequencies(data, sample_rate.load(Ordering::SeqCst));
                let color = frequencies_to_color(low, mid, high);

                let colors = unsafe { colors.load(Ordering::Relaxed).as_mut().unwrap() };

                colors.update_current(color);
                instant.store(Box::into_raw(Box::new(Instant::now())), Ordering::Relaxed);
            }
        };

        let data_callback = Arc::new(AtomicPtr::new(Box::into_raw(Box::new(data_callback))));

        let restart = Arc::new(AtomicBool::new(false));
        let restart_clone = Arc::clone(&restart);

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

            sample_rate_clone.store(config.sample_rate.0, Ordering::SeqCst);

            let data_callback_clone = Arc::clone(&data_callback);
            let restart_setter = Arc::clone(&restart_clone);

            let stream = device
                .build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let callback = unsafe {
                            data_callback_clone
                                .load(Ordering::Relaxed)
                                .as_mut()
                                .unwrap()
                        };

                        callback(data);
                    },
                    move |error| {
                        restart_setter.store(true, Ordering::SeqCst);

                        match error {
                            cpal::StreamError::DeviceNotAvailable => {
                                println!("Запрошенное устройство больше недоступно.");
                            }
                            _ => println!("{}", error),
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

    let get_t = move |instant: &Instant| {
        let elapsed = instant.elapsed().as_millis() as f32;
        let duration = COLOR_CHANGE_DURATION.as_millis() as f32;

        (elapsed / duration).min(1.0).max(0.0)
    };

    if tcp {
        thread::spawn(move || {
            let mut interpolator = Interpolator::new();
            let mut read_buffer = [0; 1024];

            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    match stream.read(&mut read_buffer) {
                        Ok(0) => break,
                        Ok(bytes_read) => {
                            let instant = unsafe {
                                instant_clone_tcp.load(Ordering::Relaxed).as_mut().unwrap()
                            };
                    
                            let t = get_t(&instant);

                            let colors = unsafe {
                                colors_reader_tcp.load(Ordering::Relaxed).as_mut().unwrap()
                            };

                            let color = interpolator.interpolate(colors, t);
                            let color: Srgb<u8> = Srgb::from_color(*color).into();

                            let body = String::from_utf8_lossy(&read_buffer[0..bytes_read]);
                            let http = body.contains("HTTP");

                            // Если HTTP — то отправляем http ответ, иначе более удобную для парсинга форму без лишнего
                            let payload = if http {
                                format!("HTTP/1.1 201 OK\r\nContent-Type: text/plain\r\nAccess-Control-Allow-Origin: *\r\n\r\n{} {} {}", color.red, color.green, color.blue)
                            } else {
                                format!("{} {} {}\n", color.red, color.green, color.blue)
                            };

                            _ = stream.write_all(payload.as_bytes());
                        }
                        Err(_) => break,
                    }
                }
            }
        });
    }

    if udp {
        thread::spawn(move || {
            let mut interpolator = Interpolator::new();

            loop {
                let instant = unsafe {
                    instant_clone_udp.load(Ordering::Relaxed).as_mut().unwrap()
                };
        
                let t = get_t(&instant);

                let colors = unsafe { colors_reader_udp.load(Ordering::Relaxed).as_mut().unwrap() };

                let color = interpolator.interpolate(colors, t);
                let color: Srgb<u8> = Srgb::from_color(*color).into();

                let (r, mut g, b) = (color.red, color.green, color.blue);

                // Зелёный уменьшен, т.к. на моей ленте жёлтый выглядит слишком зелёным
                if r > 240 && b < 20 && g > 220 {
                    g -= 60;
                }

                let payload = format!("{} {} {}\n", r, g, b);

                // В случае если не получилось отправить данные, то будет установлена большая задержка
                if socket.send_to(payload.as_bytes(), &udp_address).is_ok() {
                    thread::sleep(Duration::from_millis(20));
                } else {
                    thread::sleep(Duration::from_millis(1000));
                }
            }
        });
    }

    let set_mode = move |mode: Mode| {
        *mode_setter.lock().unwrap() = mode;
    };

    loop {
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Не удалось получить ввод из коммандной строки.");

        let input = input.trim();

        let colors = unsafe { colors_setter.load(Ordering::Relaxed).as_mut().unwrap() };

        match input {
            "white" => {
                set_mode(Mode::Static);
                colors.update_current((1.0, 0.0, 0.0));
            }
            "off" => {
                set_mode(Mode::Static);
                colors.update_current((0.0, 0.0, 0.0));
            }
            "red" => {
                set_mode(Mode::Static);
                colors.update_current((0.628, 0.25768330773615683, 29.2338851923426));
            }
            "green" => {
                set_mode(Mode::Static);
                colors.update_current((0.8664, 0.2947552610302938, 142.49533888780996));
            }
            "blue" => {
                set_mode(Mode::Static);
                colors.update_current((0.452, 0.3131362576587438, 264.05300810418345));
            }
            "pink" => {
                set_mode(Mode::Static);
                colors.update_current((0.6122, 0.2415, 22.94));
            }
            "yellow" => {
                set_mode(Mode::Static);
                colors.update_current((0.968, 0.21095439261133309, 109.76923207652135));
            }
            "music" => {
                set_mode(Mode::Colormusic);
            }
            hex if hex.starts_with("#") => {
                if hex.len() == 7 {
                    let r = &hex[1..3];
                    let g = &hex[3..5];
                    let b = &hex[5..7];

                    let r = u8::from_str_radix(r, 16).unwrap_or(0);
                    let g = u8::from_str_radix(g, 16).unwrap_or(0);
                    let b = u8::from_str_radix(b, 16).unwrap_or(0);

                    let rgb: Srgb = Srgb::new(r, g, b).into();
                    let oklch = Oklch::from_color(rgb);

                    set_mode(Mode::Static);
                    colors.update_current((oklch.l, oklch.chroma, oklch.hue.into_raw_degrees()));
                }
            }
            _ => {}
        }
    }
}
