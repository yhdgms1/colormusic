mod audio_listener;
mod colorizer;
mod colors;
mod config;
mod devices;
mod math;
mod splitter;
mod timer;

use audio_listener::lister_for_audio;
use colorizer::frequencies_to_color;
use colors::{Colors, Interpolator};
use config::Settings;
use splitter::split_into_frequencies;

use palette::{FromColor, Oklch, Srgb};
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;
use std::time::Duration;
use timer::Timer;

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

static COLOR_CHANGE_INTERVAL: LazyLock<Arc<Mutex<Duration>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Duration::from_millis(166))));

fn get_interpolator() -> &'static mut Interpolator {
    unsafe { INTERPOLATOR.load(Ordering::Relaxed).as_mut().unwrap() }
}

fn get_colors() -> &'static mut Colors {
    unsafe { COLORS.load(Ordering::Relaxed).as_mut().unwrap() }
}

fn get_timer() -> &'static mut Timer {
    unsafe { TIMER.load(Ordering::Relaxed).as_mut().unwrap() }
}

fn handle_audio(data: &[f32], sample_rate: u32) {
    let timer = get_timer();

    if timer.elapsed() < *COLOR_CHANGE_INTERVAL.lock().unwrap() {
        return;
    }

    if let Mode::Colormusic = *MODE.lock().unwrap() {
        let (low, mid, high) = split_into_frequencies(data, sample_rate);
        let color = frequencies_to_color(low, mid, high);

        let colors = get_colors();

        colors.update_current(color);
        timer.update();
    }
}

fn get_interpolator_factor() -> f32 {
    let elapsed = get_timer().elapsed().as_millis() as f32;
    let duration = COLOR_CHANGE_INTERVAL.lock().unwrap().as_millis() as f32;

    (elapsed / duration).min(1.0).max(0.0)
}

fn get_interpolated_rgb_color() -> (u8, u8, u8) {
    let interpolator = get_interpolator();
    let colors = get_colors();

    let color = interpolator.interpolate(colors, get_interpolator_factor());
    let color: Srgb<u8> = Srgb::from_color(*color).into();

    return (color.red, color.green, color.blue);
}

fn set_mode(mode: Mode) {
    *MODE.lock().unwrap() = mode;
}

fn main() {
    let Settings {
        tcp,
        tcp_port,
        udp,
        udp_address,
        udp_port,
        devices,
        color_change_interval,
    } = config::get_config();

    let color_change_interval = Duration::from_millis(color_change_interval.unwrap_or(160));

    *COLOR_CHANGE_INTERVAL.lock().unwrap() = color_change_interval;

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
        lister_for_audio(&devices, handle_audio);
    });

    if tcp {
        thread::spawn(move || {
            let mut read_buffer = [0; 1024];

            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    match stream.read(&mut read_buffer) {
                        Ok(0) => break,
                        Ok(bytes_read) => {
                            let (r, g, b) = get_interpolated_rgb_color();

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
                let colors = get_colors();
                let color: Srgb<u8> = Srgb::from_color(colors.curr).into();
    
                let (r, mut g, b) = (color.red, color.green, color.blue);
    
                if r > 240 && b < 20 && g > 220 {
                    g -= 60;
                }

                let duration = *COLOR_CHANGE_INTERVAL.lock().unwrap();
                let payload = format!("{} {} {} {}\n", r, g, b, duration.as_millis());
    
                _ = socket.send_to(payload.as_bytes(), &udp_address);
    
                thread::sleep(duration);
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
