//! Progress primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/progress/src/progress.tsx`
//!
//! Radix progress is conceptually a value clamped into a range, where the value can be absent
//! (indeterminate). Fret's shadcn recipe uses these helpers to compute the visual fill fraction.

/// Normalizes a progress value into the `[0, 1]` range.
pub fn normalize_progress(value: f32, min: f32, max: f32) -> f32 {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() {
        return 0.0;
    }
    let span = max - min;
    if !span.is_finite() || span.abs() <= f32::EPSILON {
        return 0.0;
    }
    ((value - min) / span).clamp(0.0, 1.0)
}

/// Normalizes an optional progress value; `None` represents indeterminate.
pub fn normalize_progress_opt(value: Option<f32>, min: f32, max: f32) -> Option<f32> {
    value.map(|v| normalize_progress(v, min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_progress_clamps_to_unit_interval() {
        assert_eq!(normalize_progress(-1.0, 0.0, 10.0), 0.0);
        assert_eq!(normalize_progress(0.0, 0.0, 10.0), 0.0);
        assert_eq!(normalize_progress(5.0, 0.0, 10.0), 0.5);
        assert_eq!(normalize_progress(10.0, 0.0, 10.0), 1.0);
        assert_eq!(normalize_progress(999.0, 0.0, 10.0), 1.0);
    }

    #[test]
    fn normalize_progress_handles_degenerate_ranges() {
        assert_eq!(normalize_progress(5.0, 0.0, 0.0), 0.0);
        assert_eq!(normalize_progress(f32::NAN, 0.0, 1.0), 0.0);
    }

    #[test]
    fn normalize_progress_opt_preserves_none() {
        assert_eq!(normalize_progress_opt(None, 0.0, 10.0), None);
        assert_eq!(normalize_progress_opt(Some(5.0), 0.0, 10.0), Some(0.5));
    }
}
