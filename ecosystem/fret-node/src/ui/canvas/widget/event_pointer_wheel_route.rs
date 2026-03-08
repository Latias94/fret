use super::*;

pub(super) fn route_pointer_wheel<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) {
    pointer_wheel_viewport::stop_scroll_viewport_motion(canvas, cx, snapshot);
    if searcher::handle_searcher_wheel(canvas, cx, delta, modifiers, zoom) {
        return;
    }

    if pointer_wheel_viewport::handle_scroll_zoom(
        canvas, cx, snapshot, position, delta, modifiers, zoom,
    ) {
        return;
    }

    let _ = pointer_wheel_viewport::handle_scroll_pan(canvas, cx, snapshot, delta, modifiers);
}

pub(super) fn route_pinch_gesture<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: f32,
) {
    pointer_wheel_viewport::stop_pinch_viewport_motion(canvas, cx, snapshot);
    let _ = pointer_wheel_viewport::handle_pinch_zoom(canvas, cx, snapshot, position, delta);
}
