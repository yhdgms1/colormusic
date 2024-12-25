use crate::spectrum::Spectrum;

pub fn spectrum_to_color(spectrum: Spectrum) -> (f32, f32, f32) {
    if spectrum.is_zero() {
        return (0f32, 0f32, 0f32);
    }

    let lightness = (spectrum.bass + spectrum.midrange + spectrum.presence) / 300.0;
    let hue = 360.0 * spectrum.sub_bass.min(500.0).hypot(spectrum.brilliance.min(2000.0)) / 3000.0;

    dbg!(hue);

    

    // shit
    return (lightness.clamp(0.0, 0.7), 0.3, hue);
}
