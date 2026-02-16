use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SliderValuesUpdate {
    pub values: Vec<f32>,
    pub value_index_to_change: usize,
}

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

fn decimal_count(step: f32) -> u32 {
    let step = step.abs();
    if !step.is_finite() || step <= 0.0 {
        return 0;
    }

    let s = step.to_string();
    let mut exp = 0i32;
    let mut base = s.as_str();
    if let Some(exp_at) = s.find(['e', 'E']) {
        base = &s[..exp_at];
        exp = s[exp_at + 1..].parse::<i32>().unwrap_or(0);
    }

    let base_decimals = base
        .split_once('.')
        .map(|(_, frac)| frac.len())
        .unwrap_or(0) as i32;
    let decimals = if exp < 0 {
        base_decimals.saturating_add((-exp).min(38))
    } else {
        base_decimals.saturating_sub(exp)
    };

    decimals.max(0) as u32
}

fn round_to_step_decimals(value: f32, step: f32) -> f32 {
    let decimals = decimal_count(step).min(10);
    if decimals == 0 {
        return value;
    }
    let factor = 10f64.powi(decimals as i32);
    ((value as f64 * factor).round() / factor) as f32
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
        out = round_to_step_decimals(out, step);
    }
    out
}

/// Returns the next value array after updating the value at `at_index` and sorting ascending.
///
/// This mirrors Radix `getNextSortedValues`.
pub fn next_sorted_values(prev_values: &[f32], next_value: f32, at_index: usize) -> Vec<f32> {
    if prev_values.is_empty() {
        return vec![next_value];
    }

    let mut next_values = prev_values.to_vec();
    let index = at_index.min(next_values.len().saturating_sub(1));
    next_values[index] = next_value;
    next_values.sort_by(|a, b| a.total_cmp(b));
    next_values
}

/// Given a `values` slice and `next_value`, returns the index of the closest current value.
///
/// This mirrors Radix `getClosestValueIndex`.
pub fn closest_value_index(values: &[f32], next_value: f32) -> usize {
    if values.len() <= 1 {
        return 0;
    }

    let mut closest_index = 0;
    let mut closest_distance = (values[0] - next_value).abs();
    for (index, value) in values.iter().copied().enumerate().skip(1) {
        let distance = (value - next_value).abs();
        if distance < closest_distance {
            closest_index = index;
            closest_distance = distance;
        }
    }
    closest_index
}

/// Returns the step delta between each adjacent value.
///
/// This mirrors Radix `getStepsBetweenValues`.
pub fn steps_between_values(values: &[f32]) -> Vec<f32> {
    values.windows(2).map(|pair| pair[1] - pair[0]).collect()
}

/// Verifies that all adjacent values are separated by at least `min_steps_between_values`.
///
/// This mirrors Radix `hasMinStepsBetweenValues`.
pub fn has_min_steps_between_values(values: &[f32], min_steps_between_values: f32) -> bool {
    if min_steps_between_values <= 0.0 {
        return true;
    }

    let Some(min_delta) = steps_between_values(values).into_iter().reduce(f32::min) else {
        return true;
    };

    min_delta >= min_steps_between_values
}

/// Updates a multi-thumb slider value array using Radix sorting + minimum distance rules.
///
/// Returns `None` when the update violates `min_steps_between_thumbs` (in step units).
pub fn update_multi_thumb_values(
    prev_values: &[f32],
    raw_value: f32,
    at_index: usize,
    min: f32,
    max: f32,
    step: f32,
    min_steps_between_thumbs: u32,
) -> Option<SliderValuesUpdate> {
    let step = if step.is_finite() && step > 0.0 {
        step
    } else {
        1.0
    };
    let next_value = snap_value(raw_value, min, max, step);

    let next_values = next_sorted_values(prev_values, next_value, at_index);
    let min_steps_between_values = min_steps_between_thumbs as f32 * step;
    if !has_min_steps_between_values(&next_values, min_steps_between_values) {
        return None;
    }

    let value_index_to_change = next_values
        .iter()
        .position(|value| *value == next_value)
        .unwrap_or(0);

    Some(SliderValuesUpdate {
        values: next_values,
        value_index_to_change,
    })
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
    fn snap_value_rounds_float_steps_like_radix() {
        let v = snap_value(0.30000004, 0.0, 1.0, 0.1);
        assert!((v - 0.3).abs() < 1e-6);

        let v = snap_value(1.0000001, 0.0, 2.0, 0.25);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn format_semantics_value_uses_integer_when_close() {
        assert_eq!(format_semantics_value(12.0).as_ref(), "12");
        assert_eq!(format_semantics_value(12.00001).as_ref(), "12");
    }

    #[test]
    fn closest_value_index_matches_radix_examples() {
        assert_eq!(closest_value_index(&[10.0, 30.0], 25.0), 1);
        assert_eq!(closest_value_index(&[10.0, 30.0], 11.0), 0);
    }

    #[test]
    fn update_multi_thumb_values_sorts_and_updates_index() {
        let update = update_multi_thumb_values(&[30.0, 10.0], 11.0, 1, 0.0, 100.0, 1.0, 0)
            .expect("update should be allowed");
        assert_eq!(update.values, vec![11.0, 30.0]);
        assert_eq!(update.value_index_to_change, 0);
    }

    #[test]
    fn update_multi_thumb_values_enforces_min_steps() {
        let rejected = update_multi_thumb_values(&[10.0, 12.0], 11.0, 0, 0.0, 100.0, 1.0, 2);
        assert!(rejected.is_none());

        let allowed = update_multi_thumb_values(&[10.0, 13.0], 11.0, 0, 0.0, 100.0, 1.0, 2);
        assert!(allowed.is_some());
    }
}
