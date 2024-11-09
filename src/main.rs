mod commands;
mod audio;
mod bluetooth;
mod colorizer;
mod splitter;
mod colors;
mod math;
mod timer;

use colors::{Colors, Interpolator};
use timer::Timer;

use palette::{FromColor, Oklch, Srgb};
use btleplug::api::Peripheral;
use btleplug::api::WriteType;
use cpal::traits::DeviceTrait;
use cpal::traits::StreamTrait;
use tokio::time::{sleep, Duration};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

enum Mode {
    Colormusic,
    Static,
}

const COLOR_CHANGE_INTERVAL: Duration = Duration::from_millis(166);

static COLORS: LazyLock<Arc<AtomicPtr<Colors>>> =
    LazyLock::new(|| Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Colors::new())))));

static TIMER: LazyLock<Arc<AtomicPtr<Timer>>> =
    LazyLock::new(|| Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Timer::new())))));

static INTERPOLATOR: LazyLock<Arc<AtomicPtr<Interpolator>>> =
    LazyLock::new(|| Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Interpolator::new())))));

static MODE: LazyLock<Arc<Mutex<Mode>>> = LazyLock::new(|| Arc::new(Mutex::new(Mode::Colormusic)));

fn get_interpolator() -> &'static mut Interpolator {
    unsafe { INTERPOLATOR.load(Ordering::Relaxed).as_mut().unwrap() }
}

fn get_colors() -> &'static mut Colors {
    unsafe { COLORS.load(Ordering::Relaxed).as_mut().unwrap() }
}

fn get_timer() -> &'static mut Timer {
    unsafe { TIMER.load(Ordering::Relaxed).as_mut().unwrap() }
}

fn get_interpolator_factor() -> f32 {
    let elapsed = get_timer().elapsed().as_millis() as f32;
    let duration = COLOR_CHANGE_INTERVAL.as_millis() as f32;

    (elapsed / duration).min(1.0).max(0.0)
}

fn get_current_rgb_color() -> (u8, u8, u8) {
    let interpolator = get_interpolator();
    let colors = get_colors();

    let color = interpolator.interpolate(colors, get_interpolator_factor());
    let color: Srgb<u8> = Srgb::from_color(*color).into();

    return (color.red, color.green, color.blue);
}

fn set_mode(mode: Mode) {
    *MODE.lock().unwrap() = mode;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (audio_device, audio_config) = audio::get_output();
    let (bluetooth_device, bluetooth_characteristic) = bluetooth::get_device().await?.unwrap();
    
    let stream = audio_device
        .build_input_stream(
            &audio_config,
            move |data: &[f32], _| {
                if let Mode::Colormusic = *MODE.lock().unwrap() {
                    let timer = get_timer();

                    if timer.elapsed() < COLOR_CHANGE_INTERVAL {
                        return;
                    }
    
                    let (low, mid, high) = splitter::split_into_frequencies(data, audio_config.sample_rate.0);
                    let color = colorizer::frequencies_to_color(low, mid, high);
            
                    let colors = get_colors();
                    
                    colors.update_current(color);
                    timer.update();
                }
            },
            |error| {
                let message = if matches!(error, cpal::StreamError::DeviceNotAvailable) {
                    "Запрошенное устройство больше недоступно."
                } else {
                    &error.to_string()
                };

                panic!("{message}");
            },
            None,
        )
        .expect("Не удалось создать входной поток.");

    stream.play().expect("Не удалось воспроизвести поток.");

    tokio::spawn(async move {
        loop {
            let (r, g, b) = get_current_rgb_color();
            let command = commands::create_color_command(r, g, b);
    
            let _ = bluetooth_device.write(&bluetooth_characteristic, &command, WriteType::WithoutResponse).await;
    
            sleep(Duration::from_millis(25)).await;
        }
    });

    loop {
        let mut input = String::new();

        std::io::stdin()
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
