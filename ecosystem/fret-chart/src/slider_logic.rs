use delinea::engine::window::DataWindow;
use fret_core::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SliderDragKind {
    Pan,
    HandleMin,
    HandleMax,
}

pub(crate) fn slider_norm(extent: DataWindow, value: f64) -> f32 {
    let span = extent.span();
    if !span.is_finite() || span <= 0.0 {
        return 0.0;
    }
    (((value - extent.min) / span) as f32).clamp(0.0, 1.0)
}

pub(crate) fn slider_value_at_x(track: Rect, extent: DataWindow, px_x: f32) -> f64 {
    delinea::engine::axis::data_at_px(extent, px_x, track.origin.x.0, track.size.width.0)
}

pub(crate) fn slider_value_at_y(track: Rect, extent: DataWindow, px_y: f32) -> f64 {
    let height = track.size.height.0.max(1.0);
    let bottom = track.origin.y.0 + height;
    let y = px_y.clamp(track.origin.y.0, bottom);
    let y_from_bottom = bottom - y;
    delinea::engine::axis::data_at_px(extent, y_from_bottom, 0.0, height)
}

pub(crate) fn slider_window_after_delta(
    extent: DataWindow,
    start_window: DataWindow,
    delta_value: f64,
    kind: SliderDragKind,
) -> DataWindow {
    let extent_span = extent.span();
    if !extent_span.is_finite() || extent_span <= 0.0 {
        return start_window;
    }

    let mut min = start_window.min;
    let mut max = start_window.max;

    if !delta_value.is_finite() || !min.is_finite() || !max.is_finite() {
        return start_window;
    }

    match kind {
        SliderDragKind::Pan => {
            min += delta_value;
            max += delta_value;
        }
        SliderDragKind::HandleMin => {
            min += delta_value;
        }
        SliderDragKind::HandleMax => {
            max += delta_value;
        }
    }

    let eps = (extent_span.abs() * 1e-12).max(1e-9).max(f64::MIN_POSITIVE);

    match kind {
        SliderDragKind::Pan => {
            let mut span = (max - min).abs();
            if !span.is_finite() || span <= eps {
                span = start_window.span().abs();
            }
            if !span.is_finite() || span <= eps {
                span = eps;
            }

            if span >= extent_span {
                return extent;
            }

            if max <= min {
                max = min + span;
            } else {
                span = max - min;
            }

            if min < extent.min {
                let d = extent.min - min;
                min += d;
                max += d;
            }
            if max > extent.max {
                let d = max - extent.max;
                min -= d;
                max -= d;
            }

            min = min.max(extent.min);
            max = max.min(extent.max);

            if max - min < eps {
                min = extent.min;
                max = (extent.min + span).min(extent.max);
                if max - min < eps {
                    max = (min + eps).min(extent.max);
                }
            }

            if max <= min {
                return extent;
            }

            DataWindow { min, max }
        }
        SliderDragKind::HandleMin => {
            let mut out_max = max.clamp(extent.min + eps, extent.max);
            let mut out_min = min.clamp(extent.min, out_max - eps);
            if out_max <= out_min {
                out_min = (out_max - eps).max(extent.min);
                if out_max <= out_min {
                    out_max = (out_min + eps).min(extent.max);
                }
            }
            DataWindow {
                min: out_min,
                max: out_max,
            }
        }
        SliderDragKind::HandleMax => {
            let mut out_min = min.clamp(extent.min, extent.max - eps);
            let mut out_max = max.clamp(out_min + eps, extent.max);
            if out_max <= out_min {
                out_max = (out_min + eps).min(extent.max);
                if out_max <= out_min {
                    out_min = (out_max - eps).max(extent.min);
                }
            }
            DataWindow {
                min: out_min,
                max: out_max,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_window_after_delta_clamps_and_never_inverts() {
        let extent = DataWindow {
            min: 0.0,
            max: 100.0,
        };
        let start = DataWindow {
            min: 20.0,
            max: 30.0,
        };

        let left = slider_window_after_delta(extent, start, -999.0, SliderDragKind::Pan);
        assert_eq!(
            left,
            DataWindow {
                min: 0.0,
                max: 10.0
            }
        );

        let right = slider_window_after_delta(extent, start, 999.0, SliderDragKind::Pan);
        assert_eq!(
            right,
            DataWindow {
                min: 90.0,
                max: 100.0
            }
        );

        let inverted_min =
            slider_window_after_delta(extent, start, 999.0, SliderDragKind::HandleMin);
        assert!(inverted_min.max > inverted_min.min);
        assert_eq!(inverted_min.max, start.max);
        assert!(inverted_min.min >= extent.min && inverted_min.max <= extent.max);

        let inverted_max =
            slider_window_after_delta(extent, start, -999.0, SliderDragKind::HandleMax);
        assert!(inverted_max.max > inverted_max.min);
        assert_eq!(inverted_max.min, start.min);
        assert!(inverted_max.min >= extent.min && inverted_max.max <= extent.max);
    }

    #[test]
    fn slider_norm_and_values_map_track_positions() {
        let extent = DataWindow {
            min: 10.0,
            max: 30.0,
        };
        let track = Rect::new(
            fret_core::Point::new(fret_core::Px(100.0), fret_core::Px(40.0)),
            fret_core::Size::new(fret_core::Px(200.0), fret_core::Px(80.0)),
        );

        assert_eq!(slider_norm(extent, 20.0), 0.5);
        assert_eq!(slider_value_at_x(track, extent, 200.0), 20.0);
        assert_eq!(slider_value_at_y(track, extent, 80.0), 20.0);
    }
}
