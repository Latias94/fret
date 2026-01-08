#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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

        if let Some(min) = locked_min
            && min.is_finite()
        {
            out.min = min;
        }
        if let Some(max) = locked_max
            && max.is_finite()
        {
            out.max = max;
        }

        out.clamp_non_degenerate();
        out
    }
}

pub type DataWindowX = DataWindow;
pub type DataWindowY = DataWindow;

#[cfg(test)]
mod tests {
    use super::DataWindow;

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
}
