use super::math::lerp;
use palette::Oklch;

pub struct Interpolator {
    color: Oklch,
}

impl Interpolator {
    pub fn new() -> Interpolator {
        Interpolator {
            color: Oklch::new(0f32, 0f32, 0f32),
        }
    }

    pub fn interpolate(&mut self, colors: &Colors, t: f32) -> &Oklch {
        self.color.l = lerp(colors.prev.l, colors.curr.l, t);
        self.color.chroma = lerp(colors.prev.chroma, colors.curr.chroma, t);
        self.color.hue = lerp(
            colors.prev.hue.into_raw_degrees(),
            colors.curr.hue.into_raw_degrees(),
            t,
        )
        .into();

        &self.color
    }
}

pub struct Colors {
    pub prev: Oklch,
    pub curr: Oklch,
}

impl Colors {
    pub fn new() -> Colors {
        Colors {
            prev: Oklch::new(0f32, 0f32, 0f32),
            curr: Oklch::new(0f32, 0f32, 0f32),
        }
    }

    pub fn update_current(&mut self, current: (f32, f32, f32)) {
        self.prev.l = self.curr.l;
        self.prev.chroma = self.curr.chroma;
        self.prev.hue = self.curr.hue;

        let (l, c, h) = current;

        self.curr.l = l;
        self.curr.chroma = c;
        self.curr.hue = h.into();
    }
}
