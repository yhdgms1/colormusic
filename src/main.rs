mod app_input;
mod audio_listener;
mod colorizer;
mod colors;
mod config;
mod devices;
mod splitter;
mod timer;
mod shared;
mod events;

use events::Event;
use shared::{AppConfig, Mode};
use app_input::{create_input_handler, Command};
use audio_listener::listen_for_audio;
use colorizer::frequencies_to_color;
use colors::Colors;
use config::Settings;
use splitter::split_into_frequencies;
use palette::{FromColor, Srgb};
use std::net::UdpSocket;
use std::time::Duration;
use std::sync::mpsc::channel;
use timer::Timer;

fn main() {
    let Settings {
        udp_address,
        udp_port,
        devices,
        color_change_interval,
    } = config::get_config();

    let color_change_interval = Duration::from_millis(color_change_interval.unwrap_or(260));

    let udp_address = udp_address.unwrap_or("192.168.1.167:8488".to_string());
    let udp_port = udp_port.unwrap_or("8044".to_string());

    let udp_addr = format!("0.0.0.0:{}", udp_port);
    let socket = UdpSocket::bind(udp_addr).expect("Не удалось создать Udp сокет");

    socket
        .connect(&udp_address)
        .expect("Не удалось подключиться к Udp сокету");
    
    let (tx, rx) = channel();

    create_input_handler(tx.clone());
    listen_for_audio(devices, tx);

    let mut timer = Timer::new();
    let mut colors = Colors::new();
    let mut conf = AppConfig::default();

    for event in rx {
        match event {
            Event::Commands(commands) => {
                for command in commands {
                    match command {
                        Command::SetColor(color) => {
                            colors.update_current(color);
                        },
                        Command::SetMode(mode) => {
                            conf.mode = mode;
                        },
                        Command::SetOpacity(opacity) => {
                            conf.opacity = opacity;
                        },
                        Command::SetScale(scale) => {
                            conf.scale = scale;
                        }
                    }
                }
            },
            Event::Audio((data, sr)) => {
                if timer.elapsed() < color_change_interval {
                    continue;
                }

                if let Mode::Colormusic = conf.mode {
                    let (low, mid, high) = split_into_frequencies(&data, sr);
                    let lch = frequencies_to_color(low * conf.scale, mid * conf.scale, high * conf.scale);

                    colors.update_current(lch);
                    timer.update();
                }

                let rgb: Srgb<u8> = Srgb::from_color(colors.curr).into();

                let (mut r, mut g, mut b) = (rgb.red, rgb.green, rgb.blue);

                if r > 240 && b < 20 && g > 220 {
                    g -= 60;
                }

                r = (r as f32 * conf.opacity).round() as u8;
                g = (g as f32 * conf.opacity).round() as u8;
                b = (b as f32 * conf.opacity).round() as u8;

                let mut packet = [0u8; 5];
                let interval = color_change_interval.as_millis() as u16;

                packet[0] = r;
                packet[1] = g;
                packet[2] = b;
                packet[3..5].copy_from_slice(&interval.to_le_bytes());

                _ = socket.send(&packet);
            }
        }
    }
}
