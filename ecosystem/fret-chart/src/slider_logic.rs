use delinea::engine::window::{DataWindow, WindowSpanAnchor};
use fret_core::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SliderDragKind {
    Pan,
    HandleMin,
    HandleMax,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SliderDragPermissions {
    pub pan: bool,
    pub handle_min: bool,
    pub handle_max: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SliderDragStart {
    pub kind: SliderDragKind,
    pub start_window: DataWindow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SliderDragUpdate {
    pub window: DataWindow,
    pub anchor: WindowSpanAnchor,
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

pub(crate) fn slider_anchor_for_drag_kind(kind: SliderDragKind) -> WindowSpanAnchor {
    match kind {
        SliderDragKind::HandleMin => WindowSpanAnchor::LockMax,
        SliderDragKind::HandleMax => WindowSpanAnchor::LockMin,
        SliderDragKind::Pan => WindowSpanAnchor::Center,
    }
}

pub(crate) fn slider_drag_start_at_x(
    track: Rect,
    extent: DataWindow,
    window: DataWindow,
    position_x: f32,
    handle_hit_px: f32,
    permissions: SliderDragPermissions,
) -> Option<SliderDragStart> {
    let width = track.size.width.0;
    let span = extent.span();
    if width <= 0.0 || !span.is_finite() || span <= 0.0 {
        return None;
    }

    let t0 = slider_norm(extent, window.min);
    let t1 = slider_norm(extent, window.max);
    let left = track.origin.x.0 + t0 * width;
    let right = track.origin.x.0 + t1 * width;
    let handle_hit_px = handle_hit_px.max(0.0);

    let kind = if (position_x - left).abs() <= handle_hit_px {
        SliderDragKind::HandleMin
    } else if (position_x - right).abs() <= handle_hit_px {
        SliderDragKind::HandleMax
    } else {
        SliderDragKind::Pan
    };

    slider_drag_start_for_axis(
        track,
        extent,
        window,
        position_x,
        position_x >= left && position_x <= right,
        slider_value_at_x(track, extent, position_x),
        kind,
        permissions,
    )
}

pub(crate) fn slider_drag_start_at_y(
    track: Rect,
    extent: DataWindow,
    window: DataWindow,
    position_y: f32,
    handle_hit_px: f32,
    permissions: SliderDragPermissions,
) -> Option<SliderDragStart> {
    let height = track.size.height.0;
    let span = extent.span();
    if height <= 0.0 || !span.is_finite() || span <= 0.0 {
        return None;
    }

    let t0 = slider_norm(extent, window.min);
    let t1 = slider_norm(extent, window.max);
    let bottom = track.origin.y.0 + height;
    let y_from_bottom = (bottom - position_y).clamp(0.0, height.max(1.0));
    let min_handle = t0 * height;
    let max_handle = t1 * height;
    let handle_hit_px = handle_hit_px.max(0.0);

    let kind = if (y_from_bottom - min_handle).abs() <= handle_hit_px {
        SliderDragKind::HandleMin
    } else if (y_from_bottom - max_handle).abs() <= handle_hit_px {
        SliderDragKind::HandleMax
    } else {
        SliderDragKind::Pan
    };

    slider_drag_start_for_axis(
        track,
        extent,
        window,
        y_from_bottom,
        y_from_bottom >= min_handle && y_from_bottom <= max_handle,
        slider_value_at_y(track, extent, position_y),
        kind,
        permissions,
    )
}

fn slider_drag_start_for_axis(
    _track: Rect,
    extent: DataWindow,
    window: DataWindow,
    _axis_position: f32,
    inside_window: bool,
    click_value: f64,
    kind: SliderDragKind,
    permissions: SliderDragPermissions,
) -> Option<SliderDragStart> {
    if matches!(kind, SliderDragKind::Pan) && !permissions.pan {
        return None;
    }
    if matches!(kind, SliderDragKind::HandleMin) && !permissions.handle_min {
        return None;
    }
    if matches!(kind, SliderDragKind::HandleMax) && !permissions.handle_max {
        return None;
    }

    let start_window = if matches!(kind, SliderDragKind::Pan) && !inside_window {
        let half = 0.5 * window.span();
        let start_window = DataWindow {
            min: click_value - half,
            max: click_value + half,
        };
        slider_window_after_delta(extent, start_window, 0.0, SliderDragKind::Pan)
    } else {
        window
    };

    Some(SliderDragStart { kind, start_window })
}

pub(crate) fn slider_drag_update_at_x(
    track: Rect,
    extent: DataWindow,
    start_window: DataWindow,
    start_x: f32,
    current_x: f32,
    kind: SliderDragKind,
) -> Option<SliderDragUpdate> {
    let width = track.size.width.0;
    let span = extent.span();
    if width <= 0.0 || !span.is_finite() || span <= 0.0 {
        return None;
    }

    let x = current_x.clamp(track.origin.x.0, track.origin.x.0 + width);
    let start_x = start_x.clamp(track.origin.x.0, track.origin.x.0 + width);
    let delta_px = x - start_x;
    let delta_value = (delta_px / width) as f64 * span;
    Some(SliderDragUpdate {
        window: slider_window_after_delta(extent, start_window, delta_value, kind),
        anchor: slider_anchor_for_drag_kind(kind),
    })
}

pub(crate) fn slider_drag_update_at_y(
    track: Rect,
    extent: DataWindow,
    start_window: DataWindow,
    start_y: f32,
    current_y: f32,
    kind: SliderDragKind,
) -> Option<SliderDragUpdate> {
    let height = track.size.height.0;
    let span = extent.span();
    if height <= 0.0 || !span.is_finite() || span <= 0.0 {
        return None;
    }

    let bottom = track.origin.y.0 + height;
    let y = current_y.clamp(track.origin.y.0, bottom);
    let start_y = start_y.clamp(track.origin.y.0, bottom);
    let y_from_bottom = bottom - y;
    let start_from_bottom = bottom - start_y;
    let delta_px = y_from_bottom - start_from_bottom;
    let delta_value = (delta_px / height) as f64 * span;
    Some(SliderDragUpdate {
        window: slider_window_after_delta(extent, start_window, delta_value, kind),
        anchor: slider_anchor_for_drag_kind(kind),
    })
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

    fn assert_window_close(actual: DataWindow, expected: DataWindow) {
        let eps = 1e-5;
        assert!(
            (actual.min - expected.min).abs() <= eps && (actual.max - expected.max).abs() <= eps,
            "expected window close to {expected:?}, got {actual:?}"
        );
    }

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

    #[test]
    fn slider_drag_start_at_x_selects_handle_pan_jump_and_respects_locks() {
        let extent = DataWindow {
            min: 0.0,
            max: 100.0,
        };
        let window = DataWindow {
            min: 20.0,
            max: 60.0,
        };
        let track = Rect::new(
            fret_core::Point::new(fret_core::Px(10.0), fret_core::Px(40.0)),
            fret_core::Size::new(fret_core::Px(200.0), fret_core::Px(10.0)),
        );
        let all = SliderDragPermissions {
            pan: true,
            handle_min: true,
            handle_max: true,
        };

        assert_eq!(
            slider_drag_start_at_x(track, extent, window, 50.0, 7.0, all),
            Some(SliderDragStart {
                kind: SliderDragKind::HandleMin,
                start_window: window
            })
        );
        assert_eq!(
            slider_drag_start_at_x(track, extent, window, 130.0, 7.0, all),
            Some(SliderDragStart {
                kind: SliderDragKind::HandleMax,
                start_window: window
            })
        );
        assert_eq!(
            slider_drag_start_at_x(track, extent, window, 90.0, 7.0, all),
            Some(SliderDragStart {
                kind: SliderDragKind::Pan,
                start_window: window
            })
        );
        assert_eq!(
            slider_drag_start_at_x(track, extent, window, 170.0, 7.0, all),
            Some(SliderDragStart {
                kind: SliderDragKind::Pan,
                start_window: DataWindow {
                    min: 60.0,
                    max: 100.0
                }
            })
        );
        assert_eq!(
            slider_drag_start_at_x(
                track,
                extent,
                window,
                170.0,
                7.0,
                SliderDragPermissions {
                    pan: false,
                    handle_min: true,
                    handle_max: true,
                },
            ),
            None
        );
    }

    #[test]
    fn slider_drag_start_at_y_uses_bottom_origin_for_handles_pan_and_jump() {
        let extent = DataWindow {
            min: 0.0,
            max: 100.0,
        };
        let window = DataWindow {
            min: 20.0,
            max: 60.0,
        };
        let track = Rect::new(
            fret_core::Point::new(fret_core::Px(10.0), fret_core::Px(20.0)),
            fret_core::Size::new(fret_core::Px(10.0), fret_core::Px(100.0)),
        );
        let all = SliderDragPermissions {
            pan: true,
            handle_min: true,
            handle_max: true,
        };

        assert_eq!(
            slider_drag_start_at_y(track, extent, window, 100.0, 7.0, all).map(|start| start.kind),
            Some(SliderDragKind::HandleMin)
        );
        assert_eq!(
            slider_drag_start_at_y(track, extent, window, 60.0, 7.0, all).map(|start| start.kind),
            Some(SliderDragKind::HandleMax)
        );
        assert_eq!(
            slider_drag_start_at_y(track, extent, window, 80.0, 7.0, all),
            Some(SliderDragStart {
                kind: SliderDragKind::Pan,
                start_window: window
            })
        );
        assert_eq!(
            slider_drag_start_at_y(track, extent, window, 30.0, 7.0, all),
            Some(SliderDragStart {
                kind: SliderDragKind::Pan,
                start_window: DataWindow {
                    min: 60.0,
                    max: 100.0
                }
            })
        );
    }

    #[test]
    fn slider_drag_updates_project_pointer_delta_to_window_and_anchor() {
        let extent = DataWindow {
            min: 0.0,
            max: 100.0,
        };
        let start = DataWindow {
            min: 20.0,
            max: 40.0,
        };
        let x_track = Rect::new(
            fret_core::Point::new(fret_core::Px(10.0), fret_core::Px(0.0)),
            fret_core::Size::new(fret_core::Px(200.0), fret_core::Px(10.0)),
        );
        let y_track = Rect::new(
            fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(20.0)),
            fret_core::Size::new(fret_core::Px(10.0), fret_core::Px(100.0)),
        );

        let x_update =
            slider_drag_update_at_x(x_track, extent, start, 50.0, 70.0, SliderDragKind::Pan)
                .expect("x drag should update");
        assert_eq!(x_update.anchor, WindowSpanAnchor::Center);
        assert_window_close(
            x_update.window,
            DataWindow {
                min: 30.0,
                max: 50.0,
            },
        );

        let y_update =
            slider_drag_update_at_y(y_track, extent, start, 100.0, 80.0, SliderDragKind::Pan)
                .expect("y drag should update");
        assert_eq!(y_update.anchor, WindowSpanAnchor::Center);
        assert_window_close(
            y_update.window,
            DataWindow {
                min: 40.0,
                max: 60.0,
            },
        );
        assert_eq!(
            slider_drag_update_at_x(
                x_track,
                extent,
                start,
                50.0,
                70.0,
                SliderDragKind::HandleMin
            )
            .map(|update| update.anchor),
            Some(WindowSpanAnchor::LockMax)
        );
    }
}
