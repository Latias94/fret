use fret_core::{Point, Rect};

pub(in crate::imui) fn slider_step_or_default(step: f32) -> f32 {
    if step.is_finite() && step > 0.0 {
        step
    } else {
        1.0
    }
}

pub(in crate::imui) fn slider_normalize_range(min: f32, max: f32) -> (f32, f32) {
    if min <= max { (min, max) } else { (max, min) }
}

pub(in crate::imui) fn slider_clamp_and_snap(value: f32, min: f32, max: f32, step: f32) -> f32 {
    let (min, max) = slider_normalize_range(min, max);
    if !value.is_finite() {
        return min;
    }
    if (max - min).abs() <= f32::EPSILON {
        return min;
    }
    let step = slider_step_or_default(step);
    let snapped = min + ((value - min) / step).round() * step;
    snapped.clamp(min, max)
}

pub(in crate::imui) fn slider_value_from_pointer(
    bounds: Rect,
    pointer: Point,
    min: f32,
    max: f32,
    step: f32,
) -> f32 {
    let (min, max) = slider_normalize_range(min, max);
    if (max - min).abs() <= f32::EPSILON {
        return min;
    }

    let width = bounds.size.width.0.max(1.0);
    let t = ((pointer.x.0 - bounds.origin.x.0) / width).clamp(0.0, 1.0);
    let raw = min + (max - min) * t;
    slider_clamp_and_snap(raw, min, max, step)
}
