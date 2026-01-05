use fret_core::{Px, Rect};
use fret_runtime::Model;
use fret_ui::action::UiPointerActionHost;

use crate::headless::slider as headless_slider;

/// Update a single-value slider model based on an X pointer position.
///
/// This is a small wiring helper used by shadcn recipes and other primitive consumers. It does
/// not implement the full multi-thumb Radix slider contract yet; it focuses on the common
/// single-thumb use case.
pub fn update_single_slider_model_from_pointer_x(
    host: &mut dyn UiPointerActionHost,
    model: &Model<Vec<f32>>,
    bounds: Rect,
    pointer_x: Px,
    min: f32,
    max: f32,
    step: f32,
    thumb_size: Px,
) {
    let step = if step.is_finite() && step > 0.0 { step } else { 1.0 };

    let thumb_size = thumb_size.0.max(0.0);
    let track_w = (bounds.size.width.0 - thumb_size).max(0.0);
    if track_w <= 0.0 {
        return;
    }

    let left = bounds.origin.x.0 + thumb_size * 0.5;
    let t = ((pointer_x.0 - left) / track_w).clamp(0.0, 1.0);
    let v = headless_slider::snap_value(min + (max - min) * t, min, max, step);

    let mut next = host
        .models_mut()
        .get_cloned(model)
        .unwrap_or_else(|| vec![min]);
    if next.is_empty() {
        next.push(min);
    }
    next[0] = v;
    let _ = host.models_mut().update(model, |values| *values = next);
}

