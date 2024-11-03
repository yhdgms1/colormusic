mod colorizer;
mod colors;
mod config;
mod devices;
mod math;
mod splitter;
mod timer;

use colorizer::frequencies_to_color;
use colors::{Colors, Interpolator};
use config::Settings;
use cpal::StreamError;
use devices::get_device;
use splitter::split_into_frequencies;

use cpal::traits::{DeviceTrait, StreamTrait};
use palette::{FromColor, Oklch, Srgb};
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;
use std::time::Duration;
use timer::Timer;

const COLOR_CHANGE_DURATION: Duration = Duration::from_millis(166);
const DEFAULT_UDP_ADDRESS: &str = "192.168.1.167:8488";

enum Mode {
    Colormusic,
    Static,
}

static COLORS: LazyLock<Arc<AtomicPtr<Colors>>> =
    LazyLock::new(|| Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Colors::new())))));

static TIMER: LazyLock<Arc<AtomicPtr<Timer>>> =
    LazyLock::new(|| Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Timer::new())))));

static INTERPOLATOR: LazyLock<Arc<AtomicPtr<Interpolator>>> =
    LazyLock::new(|| Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Interpolator::new())))));

static MODE: LazyLock<Arc<Mutex<Mode>>> = LazyLock::new(|| Arc::new(Mutex::new(Mode::Colormusic)));

static SAMPLE_RATE: LazyLock<Arc<Mutex<u32>>> = LazyLock::new(|| Arc::new(Mutex::new(44000)));

static CPAL_RESTART: LazyLock<Arc<Mutex<bool>>> = LazyLock::new(|| Arc::new(Mutex::new(false)));

fn get_interpolator() -> &'static mut Interpolator {
    unsafe { INTERPOLATOR.load(Ordering::Relaxed).as_mut().unwrap() }
}

fn get_colors() -> &'static mut Colors {
    unsafe { COLORS.load(Ordering::Relaxed).as_mut().unwrap() }
}

fn get_timer() -> &'static mut Timer {
    unsafe { TIMER.load(Ordering::Relaxed).as_mut().unwrap() }
}

fn handle_audio(data: &[f32], _: &cpal::InputCallbackInfo) {
    let timer = get_timer();

    if timer.elapsed() < COLOR_CHANGE_DURATION {
        return;
    }

    if let Mode::Colormusic = *MODE.lock().unwrap() {
        let sr = SAMPLE_RATE.lock().unwrap();

        let (low, mid, high) = split_into_frequencies(data, *sr);
        let color = frequencies_to_color(low, mid, high);

        let colors = get_colors();

        colors.update_current(color);
        timer.update();
    }
}

fn get_interpolator_factor() -> f32 {
    let elapsed = get_timer().elapsed().as_millis() as f32;
    let duration = COLOR_CHANGE_DURATION.as_millis() as f32;

    (elapsed / duration).min(1.0).max(0.0)
}

fn set_mode(mode: Mode) {
    *MODE.lock().unwrap() = mode;
}

fn get_current_rgb_color() -> (u8, u8, u8) {
    let interpolator = get_interpolator();
    let colors = get_colors();

    let color = interpolator.interpolate(colors, get_interpolator_factor());
    let color: Srgb<u8> = Srgb::from_color(*color).into();

    return (color.red, color.green, color.blue);
}

fn on_error(error: StreamError) {
    match error {
        cpal::StreamError::DeviceNotAvailable => {
            println!("Запрошенное устройство больше недоступно.");
        }
        _ => println!("{}", error),
    };

    *CPAL_RESTART.lock().unwrap() = true;
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

    let tcp_addr = format!("0.0.0.0:{}", tcp_port);
    let listener = TcpListener::bind(tcp_addr).expect("Не удалось создать слушатель TCP");

    let udp_addr = format!("0.0.0.0:{}", udp_port);
    let socket = UdpSocket::bind(udp_addr).expect("Не удалось создать Udp сокет");

    thread::spawn(move || {
        let host = cpal::default_host();

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

            *SAMPLE_RATE.lock().unwrap() = config.sample_rate.0;

            let stream = device
                .build_input_stream(&config, handle_audio, on_error, None)
                .expect("Не удалось создать входной поток.");

            stream.play().expect("Не удалось воспроизвести поток.");

            'inner: loop {
                let mut restart = CPAL_RESTART.lock().unwrap();

                if *restart {
                    *restart = false;
                    break 'inner;
                }

                thread::sleep(Duration::from_millis(500));
            }
        }
    });

    if tcp {
        thread::spawn(move || {
            let mut read_buffer = [0; 1024];

            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    match stream.read(&mut read_buffer) {
                        Ok(0) => break,
                        Ok(bytes_read) => {
                            let (r, g, b) = get_current_rgb_color();

                            let body = String::from_utf8_lossy(&read_buffer[0..bytes_read]);
                            let http = body.contains("HTTP");

                            // Если HTTP — то отправляем http ответ, иначе более удобную для парсинга форму без лишнего
                            let payload = if http {
                                format!("HTTP/1.1 201 OK\r\nContent-Type: text/plain\r\nAccess-Control-Allow-Origin: *\r\n\r\n{} {} {}", r, g, b)
                            } else {
                                format!("{} {} {}\n", r, g, b)
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
            loop {
                let (r, mut g, b) = get_current_rgb_color();

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

    loop {
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Не удалось получить ввод из коммандной строки.");

        let colors = get_colors();

        match input.trim() {
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
