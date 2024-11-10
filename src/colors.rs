use palette::Oklch;

pub struct Colors {
    pub curr: Oklch,
}

impl Colors {
    pub fn new() -> Colors {
        Colors {
            curr: Oklch::new(0f32, 0f32, 0f32),
        }
    }

    pub fn update_current(&mut self, current: (f32, f32, f32)) {
        let (l, c, h) = current;

        self.curr.l = l;
        self.curr.chroma = c;
        self.curr.hue = h.into();
    }
}
