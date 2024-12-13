use crate::shared::Mode;
use crate::events::Event;

use std::sync::mpsc::Sender;
use std::{io, thread};
use palette::{Oklch, Srgb, FromColor};

pub enum Command {
    SetMode(Mode),
    SetColor((f32, f32, f32)),
    SetOpacity(f32),
    SetScale(f32)
}

pub fn create_input_handler(tx: Sender<Event>) {
    let handle = move |commands: Vec<Command>| {
        _ = tx.send(Event::Commands(commands));
    };

    thread::spawn(move || {
        loop {
            let mut input = String::new();
    
            io::stdin()
                .read_line(&mut input)
                .expect("Не удалось получить ввод из коммандной строки.");
    
            match input.trim() {
                "white" => {
                    handle(vec![Command::SetMode(Mode::Static), Command::SetColor((1.0, 0.0, 0.0))]);
                }
                "off" => {
                    handle(vec![Command::SetMode(Mode::Static), Command::SetColor((0.0, 0.0, 0.0))]);
                }
                "red" => {
                    handle(vec![Command::SetMode(Mode::Static), Command::SetColor((0.628, 0.25768330773615683, 29.2338851923426))]);
                }
                "green" => {
                    handle(vec![Command::SetMode(Mode::Static), Command::SetColor((0.8664, 0.2947552610302938, 142.49533888780996))]);
                }
                "blue" => {
                    handle(vec![Command::SetMode(Mode::Static), Command::SetColor((0.452, 0.3131362576587438, 264.05300810418345))]);
                }
                "pink" => {
                    handle(vec![Command::SetMode(Mode::Static), Command::SetColor((0.6122, 0.2415, 22.94))]);
                }
                "yellow" => {
                    handle(vec![Command::SetMode(Mode::Static), Command::SetColor((0.968, 0.21095439261133309, 109.76923207652135))]);
                }
                "music" => {
                    handle(vec![Command::SetMode(Mode::Colormusic)]);
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
    
                        handle(vec![Command::SetMode(Mode::Static), Command::SetColor((oklch.l, oklch.chroma, oklch.hue.into_raw_degrees()))]);
                    }
                }
                opacity if opacity.starts_with("op") => {
                    if let Ok(op) = opacity[2..].trim().parse::<f32>() {
                        if op >= 0.0 && op <= 1.0 {
                            handle(vec![Command::SetOpacity(op)]);
                        }
                    }
                }
                scale if scale.starts_with("sc") => {
                    if let Ok(sc) = scale[2..].trim().parse::<f32>() {
                        if sc >= 0.0 {
                            handle(vec![Command::SetScale(sc)]);
                        }
                    }
                }
                _ => {}
            }
        }
    });
}