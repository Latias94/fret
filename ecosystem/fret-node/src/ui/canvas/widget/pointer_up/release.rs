use super::*;

pub(super) fn handle_non_left_releases<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    button: MouseButton,
) -> bool {
    super::super::pointer_up_state::handle_sticky_wire_ignored_release(canvas, cx, button)
        || super::super::pointer_up_state::handle_pan_release(canvas, cx, snapshot, button)
}
