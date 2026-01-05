pub fn linear_ticks(min: f32, max: f32, tick_count: usize) -> Vec<f32> {
    if tick_count == 0 {
        return Vec::new();
    }
    if !min.is_finite() || !max.is_finite() {
        return Vec::new();
    }
    let n = tick_count.max(1);
    if n == 1 {
        return vec![min];
    }
    let denom = (n - 1) as f32;
    (0..n)
        .map(|i| {
            let t = (i as f32) / denom;
            min + (max - min) * t
        })
        .filter(|v| v.is_finite())
        .collect()
}
