#[path = "pointer_down_double_click_background/apply.rs"]
mod apply;
#[path = "pointer_down_double_click_background/hit.rs"]
mod hit;
#[path = "pointer_down_double_click_background/preflight.rs"]
mod preflight;

use super::*;

pub(super) fn handle_background_zoom_double_click<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    click_count: u8,
    zoom: f32,
) -> bool {
    if !preflight::can_zoom_background_double_click(canvas, snapshot, click_count) {
        return false;
    }

    if !hit::pointer_is_background(canvas, cx, snapshot, position, zoom) {
        return false;
    }

    apply::apply_background_zoom_double_click(canvas, cx, snapshot, position, modifiers);
    true
}
