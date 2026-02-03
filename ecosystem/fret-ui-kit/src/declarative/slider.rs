use fret_core::{Px, Rect};
use fret_runtime::Model;
use fret_ui::action::UiPointerActionHost;

use crate::headless::slider as headless_slider;

fn step_or_default(step: f32) -> f32 {
    if step.is_finite() && step > 0.0 {
        step
    } else {
        1.0
    }
}

fn value_from_pointer_x(
    bounds: Rect,
    pointer_x: Px,
    min: f32,
    max: f32,
    thumb_size: Px,
) -> Option<f32> {
    let track_w = bounds.size.width.0.max(0.0);
    if track_w <= 0.0 {
        return None;
    }

    let left = bounds.origin.x.0;
    let thumb_r = (thumb_size.0.max(0.0) * 0.5).max(0.0);
    let usable_w = (track_w - 2.0 * thumb_r).max(0.0);

    // Radix keeps thumbs inside the slider bounds at the edges. Model the same behavior by mapping
    // pointer positions into the usable thumb-center span:
    //   [left + thumb_r, left + track_w - thumb_r]
    //
    // When the track is too small for the thumb, fall back to clamping to the ends.
    let t = if usable_w > 0.0 {
        ((pointer_x.0 - left - thumb_r) / usable_w).clamp(0.0, 1.0)
    } else if pointer_x.0 < (left + track_w * 0.5) {
        0.0
    } else {
        1.0
    };
    Some(min + (max - min) * t)
}

/// Starts a slider drag interaction from a pointer X position.
///
/// This mirrors Radix slide start behavior: pick the closest thumb, then update (snap + clamp)
/// the value, keeping the chosen thumb index for subsequent pointer move events.
pub fn start_slider_drag_from_pointer_x(
    host: &mut dyn UiPointerActionHost,
    model: &Model<Vec<f32>>,
    bounds: Rect,
    pointer_x: Px,
    min: f32,
    max: f32,
    step: f32,
    thumb_size: Px,
    min_steps_between_thumbs: u32,
) -> usize {
    let step = step_or_default(step);
    let Some(raw_value) = value_from_pointer_x(bounds, pointer_x, min, max, thumb_size) else {
        return 0;
    };

    let mut prev_values = host
        .models_mut()
        .get_cloned(model)
        .unwrap_or_else(|| vec![min]);
    if prev_values.is_empty() {
        prev_values.push(min);
    }

    let at_index = headless_slider::closest_value_index(&prev_values, raw_value);
    let at_index = at_index.min(prev_values.len().saturating_sub(1));

    let Some(update) = headless_slider::update_multi_thumb_values(
        &prev_values,
        raw_value,
        at_index,
        min,
        max,
        step,
        min_steps_between_thumbs,
    ) else {
        return at_index;
    };

    let _ = host
        .models_mut()
        .update(model, |values| *values = update.values);
    update.value_index_to_change
}

/// Updates a (potentially multi-thumb) slider model based on an X pointer position.
///
/// Returns the updated `value_index_to_change` which should be stored by the caller while the
/// drag gesture is active (Radix `valueIndexToChangeRef`).
pub fn update_slider_model_from_pointer_x(
    host: &mut dyn UiPointerActionHost,
    model: &Model<Vec<f32>>,
    bounds: Rect,
    pointer_x: Px,
    min: f32,
    max: f32,
    step: f32,
    thumb_size: Px,
    value_index_to_change: usize,
    min_steps_between_thumbs: u32,
) -> usize {
    let step = step_or_default(step);
    let Some(raw_value) = value_from_pointer_x(bounds, pointer_x, min, max, thumb_size) else {
        return value_index_to_change;
    };

    let mut prev_values = host
        .models_mut()
        .get_cloned(model)
        .unwrap_or_else(|| vec![min]);
    if prev_values.is_empty() {
        prev_values.push(min);
    }

    let at_index = value_index_to_change.min(prev_values.len().saturating_sub(1));
    let Some(update) = headless_slider::update_multi_thumb_values(
        &prev_values,
        raw_value,
        at_index,
        min,
        max,
        step,
        min_steps_between_thumbs,
    ) else {
        return at_index;
    };

    let _ = host
        .models_mut()
        .update(model, |values| *values = update.values);
    update.value_index_to_change
}

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
    let _ = update_slider_model_from_pointer_x(
        host, model, bounds, pointer_x, min, max, step, thumb_size, 0, 0,
    );
}
