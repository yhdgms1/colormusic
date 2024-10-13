pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + t * (end - start)
}
