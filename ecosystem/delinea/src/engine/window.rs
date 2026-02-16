#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WindowSpanAnchor {
    /// Keep the window centered.
    #[default]
    Center,
    /// Keep `min` stable and adjust `max`.
    LockMin,
    /// Keep `max` stable and adjust `min`.
    LockMax,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataWindow {
    pub min: f64,
    pub max: f64,
}

impl DataWindow {
    pub fn is_valid(&self) -> bool {
        self.min.is_finite() && self.max.is_finite() && self.max > self.min
    }

    pub fn clamp_non_degenerate(&mut self) {
        if !self.is_valid() {
            self.min = 0.0;
            self.max = 1.0;
        }
    }

    pub fn span(&self) -> f64 {
        self.max - self.min
    }

    pub fn with_span(self, span: f64, anchor: WindowSpanAnchor) -> Self {
        let span = if span.is_finite() && span > 0.0 {
            span.max(f64::MIN_POSITIVE)
        } else {
            return self;
        };

        let mut out = self;
        out.clamp_non_degenerate();

        match anchor {
            WindowSpanAnchor::Center => {
                let center = 0.5 * (out.min + out.max);
                out.min = center - 0.5 * span;
                out.max = center + 0.5 * span;
            }
            WindowSpanAnchor::LockMin => {
                out.max = out.min + span;
            }
            WindowSpanAnchor::LockMax => {
                out.min = out.max - span;
            }
        }

        out.clamp_non_degenerate();
        out
    }

    /// Applies min/max span limits for interaction-derived zoom updates.
    ///
    /// The important invariant is that the limits do not *force* the view into range if the base
    /// span was already outside of the limits (e.g. programmatic range writes). Instead, the limits
    /// only prevent interactions from moving further out of bounds.
    pub fn apply_span_limits_from_base(
        self,
        base: DataWindow,
        min_value_span: Option<f64>,
        max_value_span: Option<f64>,
        anchor: WindowSpanAnchor,
    ) -> Self {
        let base = {
            let mut base = base;
            base.clamp_non_degenerate();
            base
        };
        let mut out = self;
        out.clamp_non_degenerate();

        let base_span = base.span();
        let next_span = out.span();
        if !base_span.is_finite() || base_span <= 0.0 || !next_span.is_finite() || next_span <= 0.0
        {
            return out;
        }

        let min_value_span = min_value_span.filter(|v| v.is_finite() && *v > 0.0);
        let max_value_span = max_value_span.filter(|v| v.is_finite() && *v > 0.0);
        let (min_value_span, max_value_span) = match (min_value_span, max_value_span) {
            (Some(min), Some(max)) if min > max => (Some(max), Some(max)),
            other => other,
        };

        if next_span < base_span {
            if let Some(min) = min_value_span {
                if base_span >= min && next_span < min {
                    return out.with_span(min, anchor);
                }
                if base_span < min && next_span < base_span {
                    return base;
                }
            }
        } else if next_span > base_span
            && let Some(max) = max_value_span
        {
            if base_span <= max && next_span > max {
                return out.with_span(max, anchor);
            }
            if base_span > max && next_span > base_span {
                return base;
            }
        }

        out
    }

    /// Returns a new window panned by a pixel delta.
    ///
    /// Convention: positive `delta_px` means the pointer moved right/down in screen space,
    /// and the view follows the pointer (content under the pointer stays stable). That
    /// implies the data window moves in the opposite direction.
    pub fn pan_by_px(self, delta_px: f32, viewport_span_px: f32) -> Self {
        let span = self.span();
        if !span.is_finite() || span <= 0.0 {
            return self;
        }
        let viewport_span_px = viewport_span_px as f64;
        if !viewport_span_px.is_finite() || viewport_span_px <= 0.0 {
            return self;
        }

        let delta_data = (delta_px as f64) * span / viewport_span_px;
        let mut out = Self {
            min: self.min - delta_data,
            max: self.max - delta_data,
        };
        out.clamp_non_degenerate();
        out
    }

    /// Returns a new window zoomed around a pixel center.
    ///
    /// `log2_scale` matches the common wheel convention:
    /// - `log2_scale > 0` zooms in (span shrinks)
    /// - `log2_scale < 0` zooms out (span expands)
    pub fn zoom_by_px(self, center_px: f32, log2_scale: f32, viewport_span_px: f32) -> Self {
        let span = self.span();
        if !span.is_finite() || span <= 0.0 {
            return self;
        }
        let viewport_span_px = viewport_span_px as f64;
        if !viewport_span_px.is_finite() || viewport_span_px <= 0.0 {
            return self;
        }

        let t = ((center_px as f64) / viewport_span_px).clamp(0.0, 1.0);
        let center_data = self.min + t * span;

        let factor = 2.0_f64.powf(-(log2_scale as f64));
        if !factor.is_finite() || factor <= 0.0 {
            return self;
        }

        let new_span = (span * factor).max(f64::MIN_POSITIVE);
        let mut out = Self {
            min: center_data - t * new_span,
            max: center_data + (1.0 - t) * new_span,
        };
        out.clamp_non_degenerate();
        out
    }

    pub fn apply_constraints(self, locked_min: Option<f64>, locked_max: Option<f64>) -> Self {
        let mut out = self;
        out.clamp_non_degenerate();

        let locked_min = locked_min.filter(|v| v.is_finite());
        let locked_max = locked_max.filter(|v| v.is_finite());

        if locked_min.is_none() && locked_max.is_none() {
            return out;
        }

        let span = {
            let span = out.span();
            if span.is_finite() && span > 0.0 {
                span
            } else {
                1.0
            }
        };

        if let Some(min) = locked_min {
            out.min = min;
        }
        if let Some(max) = locked_max {
            out.max = max;
        }

        if out.max.partial_cmp(&out.min) != Some(std::cmp::Ordering::Greater) {
            match (locked_min.is_some(), locked_max.is_some()) {
                (true, false) => out.max = out.min + span,
                (false, true) => out.min = out.max - span,
                (true, true) => out.max = out.min + span,
                (false, false) => {}
            }
        }

        out.clamp_non_degenerate();
        out
    }
}

pub type DataWindowX = DataWindow;
pub type DataWindowY = DataWindow;

#[cfg(test)]
mod tests {
    use super::{DataWindow, WindowSpanAnchor};

    #[test]
    fn pan_by_px_moves_window_opposite_direction() {
        let window = DataWindow {
            min: 0.0,
            max: 10.0,
        };
        let panned = window.pan_by_px(10.0, 100.0);
        assert_eq!(
            panned,
            DataWindow {
                min: -1.0,
                max: 9.0
            }
        );
    }

    #[test]
    fn zoom_by_px_zooms_in_around_center() {
        let window = DataWindow {
            min: 0.0,
            max: 10.0,
        };
        let zoomed = window.zoom_by_px(50.0, 1.0, 100.0);
        assert_eq!(zoomed, DataWindow { min: 2.5, max: 7.5 });
    }

    #[test]
    fn apply_constraints_overrides_bounds() {
        let window = DataWindow {
            min: 0.0,
            max: 10.0,
        };
        let constrained = window.apply_constraints(Some(2.0), Some(3.0));
        assert_eq!(constrained, DataWindow { min: 2.0, max: 3.0 });
    }

    #[test]
    fn apply_constraints_lock_min_preserves_span() {
        let window = DataWindow {
            min: 0.0,
            max: 10.0,
        };
        let constrained = window.apply_constraints(Some(200.0), None);
        assert_eq!(
            constrained,
            DataWindow {
                min: 200.0,
                max: 210.0
            }
        );
    }

    #[test]
    fn apply_constraints_lock_max_preserves_span() {
        let window = DataWindow {
            min: 0.0,
            max: 10.0,
        };
        let constrained = window.apply_constraints(None, Some(-100.0));
        assert_eq!(
            constrained,
            DataWindow {
                min: -110.0,
                max: -100.0
            }
        );
    }

    #[test]
    fn apply_span_limits_does_not_expand_when_base_is_below_min_span() {
        let base = DataWindow { min: 0.0, max: 1.0 };
        let requested = DataWindow { min: 0.0, max: 0.5 };

        let limited =
            requested.apply_span_limits_from_base(base, Some(10.0), None, WindowSpanAnchor::Center);

        assert_eq!(limited, base);
    }

    #[test]
    fn apply_span_limits_clamps_zoom_in_to_min_span() {
        let base = DataWindow {
            min: 0.0,
            max: 20.0,
        };
        let requested = DataWindow {
            min: 9.0,
            max: 10.0,
        };

        let limited =
            requested.apply_span_limits_from_base(base, Some(5.0), None, WindowSpanAnchor::LockMax);

        assert_eq!(
            limited,
            DataWindow {
                min: 5.0,
                max: 10.0
            }
        );
    }
}
