use crate::primitives::numeric_value::NumericValueConstraints;

#[cfg(test)]
mod tests;

pub(super) fn quantize_value(min: f64, max: f64, clamp: bool, step: Option<f64>, v: f64) -> f64 {
    NumericValueConstraints {
        min: Some(min),
        max: Some(max),
        clamp,
        step,
    }
    .apply_f64(v)
}

pub(super) fn t_from_value(min: f64, max: f64, clamp: bool, v: f64) -> f32 {
    let range = max - min;
    if !range.is_finite() || range.abs() <= f64::EPSILON {
        return 0.0;
    }
    let mut out = (v - min) / range;
    if clamp {
        out = out.clamp(0.0, 1.0);
    }
    out as f32
}

pub(super) fn value_from_x(
    min: f64,
    max: f64,
    clamp: bool,
    step: Option<f64>,
    x: f64,
    width: f64,
    thumb_d: f64,
) -> f64 {
    let avail = (width - thumb_d).max(0.0);
    if avail <= f64::EPSILON {
        return quantize_value(min, max, clamp, step, min);
    }
    let thumb_r = thumb_d * 0.5;
    let thumb_left = (x - thumb_r).clamp(0.0, avail);
    let t = thumb_left / avail;
    let v = min + (max - min) * t;
    quantize_value(min, max, clamp, step, v)
}
