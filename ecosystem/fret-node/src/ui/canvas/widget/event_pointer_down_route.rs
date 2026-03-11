mod dispatch;
mod double_click;

use super::*;

pub(super) fn route_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    modifiers: fret_core::Modifiers,
    click_count: u8,
    zoom: f32,
) {
    if searcher::handle_searcher_pointer_down(canvas, cx, position, button, zoom) {
        return;
    }

    if pointer_down_gesture_start::handle_close_button_pointer_down(
        canvas, cx, snapshot, position, button, zoom,
    ) {
        return;
    }

    if double_click::handle_left_button_double_click_routes(
        canvas,
        cx,
        snapshot,
        position,
        button,
        modifiers,
        click_count,
        zoom,
    ) {
        return;
    }

    if pointer_down_gesture_start::handle_context_menu_pointer_down(
        canvas, cx, position, button, zoom,
    ) {
        return;
    }

    if pointer_down_gesture_start::handle_pending_right_click_start(
        canvas, cx, snapshot, position, button,
    ) {
        return;
    }

    if pointer_down_gesture_start::handle_sticky_wire_pointer_down(
        canvas, cx, snapshot, position, button, zoom,
    ) {
        return;
    }

    if pointer_down_gesture_start::handle_pan_start_pointer_down(
        canvas, cx, snapshot, position, button, modifiers,
    ) {
        return;
    }

    dispatch::dispatch_tail_pointer_down(canvas, cx, snapshot, position, button, modifiers, zoom);
}
