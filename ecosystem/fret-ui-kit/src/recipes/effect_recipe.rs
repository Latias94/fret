use fret_core::{Color, Px};

pub fn clamp_u32_from_metric(px: Px, min: u32, max: u32, default: u32) -> u32 {
    if min > max {
        return default;
    }
    let raw = px.0.round();
    if !raw.is_finite() {
        return default;
    }
    (raw as i64).clamp(min as i64, max as i64) as u32
}

pub fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

pub fn alpha_set(mut c: Color, a: f32) -> Color {
    c.a = a.clamp(0.0, 1.0);
    c
}
