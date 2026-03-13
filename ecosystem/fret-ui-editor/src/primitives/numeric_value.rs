//! Shared numeric constraint helpers for editor controls.
//!
//! This keeps step/clamp semantics consistent across scrub-style and typed numeric surfaces.

use crate::primitives::drag_value_core::DragValueScalar;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NumericValueConstraints {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub clamp: bool,
    pub step: Option<f64>,
}

impl NumericValueConstraints {
    pub fn normalized(self) -> Self {
        let mut min = self.min.filter(|v| v.is_finite());
        let mut max = self.max.filter(|v| v.is_finite());
        if let (Some(lo), Some(hi)) = (min, max)
            && lo > hi
        {
            min = Some(hi);
            max = Some(lo);
        }

        Self {
            min,
            max,
            clamp: self.clamp,
            step: self.step.filter(|step| step.is_finite() && *step > 0.0),
        }
    }

    pub fn contains_f64(self, value: f64) -> bool {
        let constraints = self.normalized();
        if let Some(min) = constraints.min
            && value < min
        {
            return false;
        }
        if let Some(max) = constraints.max
            && value > max
        {
            return false;
        }
        true
    }

    pub fn apply_f64(self, value: f64) -> f64 {
        if !value.is_finite() {
            return value;
        }

        let constraints = self.normalized();
        let mut out = value;

        if let Some(step) = constraints.step {
            let origin = constraints.min.unwrap_or(0.0);
            out = ((out - origin) / step).round() * step + origin;
        }

        if constraints.clamp {
            if let Some(min) = constraints.min {
                out = out.max(min);
            }
            if let Some(max) = constraints.max {
                out = out.min(max);
            }
        }

        out
    }
}

pub fn constrain_numeric_value<T>(constraints: NumericValueConstraints, value: T) -> T
where
    T: DragValueScalar,
{
    T::from_f64(constraints.apply_f64(value.to_f64()))
}

#[cfg(test)]
mod tests {
    use super::{NumericValueConstraints, constrain_numeric_value};

    #[test]
    fn numeric_constraints_swap_inverted_bounds() {
        let constraints = NumericValueConstraints {
            min: Some(10.0),
            max: Some(2.0),
            clamp: true,
            step: None,
        }
        .normalized();

        assert_eq!(constraints.min, Some(2.0));
        assert_eq!(constraints.max, Some(10.0));
    }

    #[test]
    fn numeric_constraints_quantize_from_min_origin_then_clamp() {
        let constraints = NumericValueConstraints {
            min: Some(0.0),
            max: Some(1.0),
            clamp: true,
            step: Some(0.125),
        };

        assert!((constraints.apply_f64(0.61) - 0.625).abs() < 1e-9);
        assert!((constraints.apply_f64(1.24) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn numeric_constraints_quantize_without_range_uses_zero_origin() {
        let constraints = NumericValueConstraints {
            min: None,
            max: None,
            clamp: false,
            step: Some(0.5),
        };

        assert!((constraints.apply_f64(1.24) - 1.0).abs() < 1e-9);
        assert_eq!(constrain_numeric_value(constraints, 3_i32), 3);
    }
}
