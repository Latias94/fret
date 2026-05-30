use crate::primitives::numeric_value::NumericValueConstraints;

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

#[cfg(test)]
mod tests {
    use super::{t_from_value, value_from_x};

    #[test]
    fn slider_t_from_value_returns_zero_for_degenerate_ranges() {
        assert_eq!(t_from_value(1.0, 1.0, true, 1.0), 0.0);
        assert_eq!(t_from_value(0.0, f64::INFINITY, true, 1.0), 0.0);
    }

    #[test]
    fn slider_t_from_value_clamps_when_requested() {
        assert_eq!(t_from_value(0.0, 10.0, true, 12.0), 1.0);
        assert_eq!(t_from_value(0.0, 10.0, true, -2.0), 0.0);
        assert_eq!(t_from_value(0.0, 10.0, false, 12.0), 1.2);
    }

    #[test]
    fn slider_value_from_x_accounts_for_thumb_radius_and_step_quantization() {
        let value = value_from_x(0.0, 10.0, true, Some(0.5), 55.0, 110.0, 10.0);

        assert_eq!(value, 5.0);
    }

    #[test]
    fn slider_value_from_x_returns_quantized_min_when_track_has_no_available_width() {
        let value = value_from_x(0.3, 1.0, true, Some(0.25), 10.0, 8.0, 10.0);

        assert_eq!(value, 0.3);
    }
}
