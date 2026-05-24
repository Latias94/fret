use super::*;

pub(super) fn handle_pointer_down_starts<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl pointer_down_gesture_start::PointerDownStartCx<H, M>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    pointer_down_gesture_start::handle_context_menu_pointer_down(canvas, cx, position, button, zoom)
        || pointer_down_gesture_start::handle_pending_right_click_start(
            canvas, cx, snapshot, position, button,
        )
        || pointer_down_gesture_start::handle_sticky_wire_pointer_down(
            canvas, cx, snapshot, position, button, zoom,
        )
        || pointer_down_gesture_start::handle_pan_start_pointer_down(
            canvas, cx, snapshot, position, button, modifiers,
        )
}
