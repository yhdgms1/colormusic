mod audio_listener;
mod colorizer;
mod colors;
mod config;
mod devices;
mod splitter;
mod timer;

use audio_listener::lister_for_audio;
use colorizer::frequencies_to_color;
use colors::Colors;
use config::Settings;
use splitter::split_into_frequencies;

use palette::{FromColor, Oklch, Srgb};
use std::io;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;
use std::time::Duration;
use timer::Timer;

enum Mode {
    Colormusic,
    Static,
}

struct AppConfig {
    mode: Mode,
    opacity: f32,
}

static COLORS: LazyLock<Arc<AtomicPtr<Colors>>> =
    LazyLock::new(|| Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Colors::new())))));

static APP_CFG: LazyLock<Arc<Mutex<AppConfig>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(AppConfig {
        mode: Mode::Colormusic,
        opacity: 1.0,
    }))
});

fn get_colors() -> &'static mut Colors {
    unsafe { COLORS.load(Ordering::Relaxed).as_mut().unwrap() }
}

fn set_mode(mode: Mode) {
    APP_CFG.lock().unwrap().mode = mode;
}

fn set_opacity(opacity: f32) {
    APP_CFG.lock().unwrap().opacity = opacity;
}

fn main() {
    let Settings {
        udp_address,
        udp_port,
        devices,
        color_change_interval,
    } = config::get_config();

    let color_change_interval = Duration::from_millis(color_change_interval.unwrap_or(300));

    let udp_address = udp_address.unwrap_or("192.168.1.167:8488".to_string());
    let udp_port = udp_port.unwrap_or("8044".to_string());

    let udp_addr = format!("0.0.0.0:{}", udp_port);
    let socket = UdpSocket::bind(udp_addr).expect("Не удалось создать Udp сокет");

    socket
        .connect(&udp_address)
        .expect("Не удалось подключиться к Udp сокету");

    thread::spawn(move || {
        let mut timer = Timer::new();

        lister_for_audio(&devices, move |data, sr| {
            if timer.elapsed() < color_change_interval {
                return;
            }

            let colors = get_colors();
            let cfg = APP_CFG.lock().unwrap();

            if let Mode::Colormusic = cfg.mode {
                let (low, mid, high) = split_into_frequencies(data, sr);
                let lch = frequencies_to_color(low, mid, high);

                colors.update_current(lch);
                timer.update();
            }

            let rgb: Srgb<u8> = Srgb::from_color(colors.curr).into();

            let (mut r, mut g, mut b) = (rgb.red, rgb.green, rgb.blue);

            if r > 240 && b < 20 && g > 220 {
                g -= 60;
            }

            r = (r as f32 * cfg.opacity).round() as u8;
            g = (g as f32 * cfg.opacity).round() as u8;
            b = (b as f32 * cfg.opacity).round() as u8;

            let mut packet = [0u8; 5];
            let interval = color_change_interval.as_millis() as u16;

            packet[0] = r;
            packet[1] = g;
            packet[2] = b;
            packet[3..5].copy_from_slice(&interval.to_le_bytes());

            _ = socket.send(&packet);
        });
    });

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
            opacity if opacity.starts_with("op") => {
                if let Ok(op) = opacity[2..].trim().parse::<f32>() {
                    if op >= 0.0 && op <= 1.0 {
                        set_opacity(op);
                    }
                }
            }
            _ => {}
        }
    }
}
