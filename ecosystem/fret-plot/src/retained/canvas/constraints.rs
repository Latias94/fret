//! Axis constraints for plot view bounds.

use crate::cartesian::{AxisScale, DataRect};
use crate::plot::view::sanitize_data_rect_scaled;

#[derive(Debug, Clone, Copy, Default)]
pub struct AxisConstraints {
    pub min: Option<f64>,
    pub max: Option<f64>,
    /// Minimum axis span in axis units (linear: data units, log: decades).
    pub min_span: Option<f64>,
    /// Maximum axis span in axis units (linear: data units, log: decades).
    pub max_span: Option<f64>,
}

impl AxisConstraints {
    pub fn limits(min: f64, max: f64) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
            ..Default::default()
        }
    }

    pub fn min_span(mut self, span: f64) -> Self {
        self.min_span = Some(span);
        self
    }

    pub fn max_span(mut self, span: f64) -> Self {
        self.max_span = Some(span);
        self
    }
}

fn axis_to_units(scale: AxisScale, v: f64) -> Option<f64> {
    if !v.is_finite() {
        return None;
    }
    match scale {
        AxisScale::Linear => Some(v),
        AxisScale::Log10 => (v > 0.0).then(|| v.log10()).filter(|u| u.is_finite()),
    }
}

fn axis_from_units(scale: AxisScale, u: f64) -> Option<f64> {
    if !u.is_finite() {
        return None;
    }
    match scale {
        AxisScale::Linear => Some(u),
        AxisScale::Log10 => Some(10.0_f64.powf(u)).filter(|v| v.is_finite() && *v > 0.0),
    }
}

fn constrain_axis_range_scaled(
    scale: AxisScale,
    min: f64,
    max: f64,
    constraints: AxisConstraints,
) -> Option<(f64, f64)> {
    if !min.is_finite() || !max.is_finite() || max <= min {
        return None;
    }

    let mut u_min = axis_to_units(scale, min)?;
    let mut u_max = axis_to_units(scale, max)?;
    if u_max <= u_min {
        return None;
    }

    let mut u_allowed_min = constraints.min.and_then(|v| axis_to_units(scale, v));
    let mut u_allowed_max = constraints.max.and_then(|v| axis_to_units(scale, v));

    if let (Some(a), Some(b)) = (u_allowed_min, u_allowed_max)
        && (!(a.is_finite() && b.is_finite()) || b <= a)
    {
        u_allowed_min = None;
        u_allowed_max = None;
    }

    if let Some(max_span) = constraints.max_span.filter(|s| s.is_finite() && *s > 0.0) {
        let span = u_max - u_min;
        if span.is_finite() && span > max_span {
            let center = (u_min + u_max) * 0.5;
            u_min = center - max_span * 0.5;
            u_max = center + max_span * 0.5;
        }
    }

    if let Some(min_span) = constraints.min_span.filter(|s| s.is_finite() && *s > 0.0) {
        let span = u_max - u_min;
        if span.is_finite() && span < min_span {
            let center = (u_min + u_max) * 0.5;
            u_min = center - min_span * 0.5;
            u_max = center + min_span * 0.5;
        }
    }

    let span = u_max - u_min;
    if !span.is_finite() || span <= 0.0 {
        return None;
    }

    match (u_allowed_min, u_allowed_max) {
        (Some(a), Some(b)) => {
            let allowed_span = b - a;
            if !allowed_span.is_finite() || allowed_span <= 0.0 {
                return None;
            }

            if span >= allowed_span {
                u_min = a;
                u_max = b;
            } else {
                if u_min < a {
                    let d = a - u_min;
                    u_min += d;
                    u_max += d;
                }
                if u_max > b {
                    let d = u_max - b;
                    u_min -= d;
                    u_max -= d;
                }

                u_min = u_min.max(a);
                u_max = u_max.min(b);
            }
        }
        (Some(a), None) => {
            if u_min < a {
                let d = a - u_min;
                u_min += d;
                u_max += d;
            }
        }
        (None, Some(b)) => {
            if u_max > b {
                let d = u_max - b;
                u_min -= d;
                u_max -= d;
            }
        }
        (None, None) => {}
    }

    if u_max <= u_min || !u_min.is_finite() || !u_max.is_finite() {
        return None;
    }

    let out_min = axis_from_units(scale, u_min)?;
    let out_max = axis_from_units(scale, u_max)?;
    (out_max > out_min).then_some((out_min, out_max))
}

pub(super) fn constrain_view_bounds_scaled(
    view: DataRect,
    x_scale: AxisScale,
    y_scale: AxisScale,
    x_constraints: AxisConstraints,
    y_constraints: AxisConstraints,
) -> DataRect {
    let mut out = view;

    if let Some((x0, x1)) =
        constrain_axis_range_scaled(x_scale, out.x_min, out.x_max, x_constraints)
    {
        out.x_min = x0;
        out.x_max = x1;
    }
    if let Some((y0, y1)) =
        constrain_axis_range_scaled(y_scale, out.y_min, out.y_max, y_constraints)
    {
        out.y_min = y0;
        out.y_max = y1;
    }

    sanitize_data_rect_scaled(out, x_scale, y_scale)
}

#[cfg(test)]
mod axis_constraints_tests {
    use super::*;

    #[test]
    fn linear_limits_clamp_span_to_allowed_range() {
        let c = AxisConstraints::limits(2.0, 8.0);
        let out = constrain_axis_range_scaled(AxisScale::Linear, 0.0, 10.0, c).unwrap();
        assert_eq!(out, (2.0, 8.0));
    }

    #[test]
    fn linear_min_shifts_range_without_changing_span() {
        let c = AxisConstraints {
            min: Some(2.0),
            max: None,
            ..Default::default()
        };
        let out = constrain_axis_range_scaled(AxisScale::Linear, 0.0, 10.0, c).unwrap();
        assert_eq!(out, (2.0, 12.0));
    }

    #[test]
    fn log10_min_span_is_expressed_in_decades() {
        let c = AxisConstraints::default().min_span(4.0);
        let (a, b) = constrain_axis_range_scaled(AxisScale::Log10, 1.0, 1000.0, c).unwrap();
        let span_decades = b.log10() - a.log10();
        assert!((span_decades - 4.0).abs() < 1.0e-9);
    }
}
