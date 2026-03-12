use super::*;

pub(super) fn handle_pointer_move_release_guards<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    pointer_move_release::handle_missing_pan_release(canvas, cx, position, buttons, modifiers)
        || pointer_move_release::handle_pending_right_click_pan_start(
            canvas, cx, snapshot, position, buttons, zoom,
        )
        || pointer_move_release::handle_missing_left_release(
            canvas, cx, position, buttons, modifiers,
        )
}
