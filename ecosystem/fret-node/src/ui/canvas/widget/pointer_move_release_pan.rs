mod missing_release;
mod pending_right_click;

use super::*;

pub(super) fn handle_missing_pan_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
) -> bool {
    missing_release::handle_missing_pan_release(canvas, cx, position, buttons, modifiers)
}

pub(super) fn handle_pending_right_click_pan_start<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: fret_core::MouseButtons,
    zoom: f32,
) -> bool {
    pending_right_click::handle_pending_right_click_pan_start(
        canvas, cx, snapshot, position, buttons, zoom,
    )
}
