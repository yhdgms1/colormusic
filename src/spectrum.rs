use rustfft::{num_complex::Complex, FftPlanner};

// https://www.teachmeaudio.com/mixing/techniques/audio-spectrum
#[derive(Default, Debug, Clone)]
pub struct Spectrum {
    pub sub_bass: f32,
    pub bass: f32,
    pub low_midrange: f32,
    pub midrange: f32,
    pub upper_midrange: f32,
    pub presence: f32,
    pub brilliance: f32
}

impl Spectrum {
    pub fn is_zero(&self) -> bool {
        return self.sub_bass == 0.0 && self.bass == 0.0 && self.low_midrange == 0.0 && self.midrange == 0.0 && self.upper_midrange == 0.0 && self.presence == 0.0 && self.brilliance == 0.0;
    }

    pub fn scale(&mut self, scale: f32) {
        self.sub_bass *= scale;
        self.bass *= scale;
        self.low_midrange *= scale;
        self.midrange *= scale;
        self.upper_midrange *= scale;
        self.presence *= scale;
        self.brilliance *= scale;
    }
}

pub fn get_spectrum(samples: &[f32], sample_rate: u32) -> Spectrum {
    if samples.iter().all(|&f| f == 0.0) {
        return Spectrum::default();
    }

    let mut buffer: Vec<Complex<f32>> = samples.iter().map(|&s| Complex::new(s, 0.0)).collect();
    let len = samples.len();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(len);

    fft.process(&mut buffer);

    // В случае, если частота дискретизации превышает u16::MAX, нужно будет изменить на u32
    let mut index = 0u16;
    let freq_factor = sample_rate as u16 / len as u16 / 2u16;

    let mut spectrum = Spectrum::default();

    for complex in buffer {
        let freq = index * freq_factor;
        let norm = complex.norm();

        match freq {
            20..60 => {
                spectrum.sub_bass += norm
            },
            60..250 => {
                spectrum.bass += norm;
            },
            250..500 => {
                spectrum.low_midrange += norm;
            },
            500..2000 => {
                spectrum.midrange += norm;
            },
            2000..4000 => {
                spectrum.upper_midrange += norm;
            },
            4000..6000 => {
                spectrum.presence += norm;
            },
            6000..20000 => {
                spectrum.brilliance += norm;
            }
            _ => {}
        }

        index += 1;
    }

    dbg!(spectrum.clone());

    return spectrum;
}
