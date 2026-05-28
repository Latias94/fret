//! Shared Material modal panel motion helpers.
//!
//! Modal Material surfaces enter by fading, rising, and scaling toward their final centered
//! geometry. Keeping this transform in foundation avoids per-component drift between Dialog,
//! DatePickerDialog, TimePickerDialog, and later modal recipes.

use fret_core::{Point, Px, Rect, Transform2D};

pub const MODAL_PANEL_CLOSED_SCALE: f32 = 0.9;
pub const MODAL_PANEL_OPEN_SCALE: f32 = 1.0;
pub const MODAL_PANEL_RISE_DISTANCE: Px = Px(20.0);

pub fn material_modal_panel_transform(bounds: Rect, progress: f32) -> Transform2D {
    let progress = progress.clamp(0.0, 1.0);
    let translate_y = Px((1.0 - progress) * MODAL_PANEL_RISE_DISTANCE.0);
    let scale =
        MODAL_PANEL_CLOSED_SCALE + (MODAL_PANEL_OPEN_SCALE - MODAL_PANEL_CLOSED_SCALE) * progress;

    let origin = Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    );
    let origin_inv = Point::new(Px(-origin.x.0), Px(-origin.y.0));

    Transform2D::translation(Point::new(Px(0.0), translate_y))
        * Transform2D::translation(origin)
        * Transform2D::scale_uniform(scale)
        * Transform2D::translation(origin_inv)
}
