pub fn frequencies_to_color(low: f32, mid: f32, high: f32) -> (f32, f32, f32) {
    if low == 0.0 && mid == 0.0 && high == 0.0 {
        return (0f32, 0f32, 0f32);
    }

    let lightness = (high / 1000.0).min(0.7).max(0.05);
    let mut chroma = (mid / 200.0).min(0.3).max(0.15);
    let hue;

    if low <= 400.0 || mid <= 200.0 {
        hue = low.min(282.0).max(102.0);
        chroma = chroma.min(0.267).max(0.2255);
    } else if low > 400.0 && high < 500.0 {
        hue = low % 360.0;
    } else {
        let max_value = 900.0;
        let max_scale = 76.0;
        let scaled_value = (low / max_value) * max_scale;

        let half = max_scale / 2.0;

        if scaled_value <= half {
            hue = (scaled_value / half) * 20.0;
        } else {
            hue = 304.0 + ((scaled_value - half) / half) * (360.0 - 304.0);
        }

        chroma = chroma.min(0.2412);
    }

    (lightness, chroma, hue)
}
