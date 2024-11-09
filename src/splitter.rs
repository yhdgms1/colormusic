use rustfft::{num_complex::Complex, FftPlanner};

pub fn split_into_frequencies(samples: &[f32], sample_rate: u32) -> (f32, f32, f32) {
    if samples.iter().all(|&f| f == 0.0) {
        return (0.0, 0.0, 0.0);
    }

    let mut buffer: Vec<Complex<f32>> = samples.iter().map(|&s| Complex::new(s, 0.0)).collect();
    let len = samples.len();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(len);

    fft.process(&mut buffer);

    let mut low = 0f32;
    let mut med = 0f32;
    let mut high = 0f32;

    // В случае, если частота дискретизации превышает u16::MAX, нужно будет изменить на u32
    let mut index = 0u16;
    let freq_factor = sample_rate as u16 / len as u16;

    for complex in buffer {
        let freq = index * freq_factor;
        let norm = complex.norm();

        if freq < 300 {
            low += norm;
        } else if freq >= 300 && freq < 2000 {
            med += norm;
        } else {
            high += norm;
        }

        index += 1;
    }

    (low, med, high)
}
