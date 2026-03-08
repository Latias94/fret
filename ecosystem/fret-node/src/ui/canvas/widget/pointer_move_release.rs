use super::*;

pub(super) fn handle_missing_pan_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::pointer_move_release_pan::handle_missing_pan_release(
        canvas, cx, position, buttons, modifiers,
    )
}

pub(super) fn handle_pending_right_click_pan_start<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: fret_core::MouseButtons,
    zoom: f32,
) -> bool {
    super::pointer_move_release_pan::handle_pending_right_click_pan_start(
        canvas, cx, snapshot, position, buttons, zoom,
    )
}

pub(super) fn handle_missing_left_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::pointer_move_release_left::handle_missing_left_release(
        canvas, cx, position, buttons, modifiers,
    )
}

pub(super) fn seed_or_update_last_pointer_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    position: Point,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::pointer_move_pointer_state::seed_or_update_last_pointer_state(
        canvas, position, modifiers,
    )
}
