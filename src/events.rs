use crate::app_input::Command;

pub enum Event {
    Commands(Vec<Command>),
    Audio((Vec<f32>, u32)),
}