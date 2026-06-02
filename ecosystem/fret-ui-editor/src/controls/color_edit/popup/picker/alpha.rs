use std::sync::Arc;

mod bar;
mod interaction;
mod preview;

pub(in crate::controls::color_edit::popup) use bar::alpha_bar;
pub(super) use bar::vertical_alpha_bar;

pub(in crate::controls::color_edit) fn alpha_from_local_x(x: f32, width: f32) -> f32 {
    if !width.is_finite() || width <= f32::EPSILON {
        return 0.0;
    }
    (x / width).clamp(0.0, 1.0)
}

pub(in crate::controls::color_edit) fn alpha_from_local_y(y: f32, height: f32) -> f32 {
    if !height.is_finite() || height <= f32::EPSILON {
        return 1.0;
    }
    (1.0 - y / height).clamp(0.0, 1.0)
}

pub(in crate::controls::color_edit) fn alpha_percent_text(alpha: f32) -> Arc<str> {
    Arc::from(format!(
        "{}%",
        (alpha.clamp(0.0, 1.0) * 100.0).round() as u8
    ))
}
