use std::time::{Duration, Instant};

pub struct Timer {
    stop: Instant
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            stop: Instant::now()
        }
    }

    pub fn update(&mut self) {
        self.stop = Instant::now()
    }

    pub fn elapsed(&self) -> Duration {
        self.stop.elapsed()
    }
}