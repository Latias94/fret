use std::sync::Arc;

/// Formats a slider value for display in the semantics tree.
///
/// Radix exposes slider state via `aria-valuenow` (number) and `aria-valuetext` (optional). In
/// Fret we currently store the value as a string in `SemanticsNode.value`, so we choose a stable,
/// human-readable format.
pub fn format_semantics_value(value: f32) -> Arc<str> {
    if !value.is_finite() {
        return Arc::from("NaN");
    }
    let rounded = value.round();
    if (value - rounded).abs() < 1e-4 {
        return Arc::from(format!("{}", rounded as i64).into_boxed_str());
    }
    Arc::from(format!("{value:.2}").into_boxed_str())
}

/// Normalizes a scalar value into the `[0, 1]` range.
pub fn normalize_value(value: f32, min: f32, max: f32) -> f32 {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() {
        return 0.0;
    }
    let span = max - min;
    if !span.is_finite() || span.abs() <= f32::EPSILON {
        return 0.0;
    }
    ((value - min) / span).clamp(0.0, 1.0)
}

/// Clamps and snaps a value to the nearest step (if step > 0).
pub fn snap_value(value: f32, min: f32, max: f32, step: f32) -> f32 {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() {
        return min;
    }
    let mut out = value.clamp(min, max);
    if step.is_finite() && step > 0.0 {
        let steps = ((out - min) / step).round();
        out = (min + steps * step).clamp(min, max);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_value_handles_degenerate_ranges() {
        assert_eq!(normalize_value(5.0, 0.0, 0.0), 0.0);
        assert_eq!(normalize_value(f32::NAN, 0.0, 1.0), 0.0);
        assert_eq!(normalize_value(0.0, f32::NAN, 1.0), 0.0);
    }

    #[test]
    fn normalize_value_clamps_to_unit_interval() {
        assert_eq!(normalize_value(-1.0, 0.0, 10.0), 0.0);
        assert_eq!(normalize_value(0.0, 0.0, 10.0), 0.0);
        assert_eq!(normalize_value(5.0, 0.0, 10.0), 0.5);
        assert_eq!(normalize_value(10.0, 0.0, 10.0), 1.0);
        assert_eq!(normalize_value(999.0, 0.0, 10.0), 1.0);
    }

    #[test]
    fn snap_value_snaps_to_nearest_step() {
        assert_eq!(snap_value(0.0, 0.0, 10.0, 1.0), 0.0);
        assert_eq!(snap_value(0.49, 0.0, 10.0, 1.0), 0.0);
        assert_eq!(snap_value(0.51, 0.0, 10.0, 1.0), 1.0);
        assert_eq!(snap_value(9.8, 0.0, 10.0, 1.0), 10.0);
        assert_eq!(snap_value(5.3, 0.0, 10.0, 0.5), 5.5);
    }

    #[test]
    fn format_semantics_value_uses_integer_when_close() {
        assert_eq!(format_semantics_value(12.0).as_ref(), "12");
        assert_eq!(format_semantics_value(12.00001).as_ref(), "12");
    }
}
