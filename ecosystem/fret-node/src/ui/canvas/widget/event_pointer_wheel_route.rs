use super::*;

pub(super) fn route_pointer_wheel<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    platform: fret_runtime::Platform,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) where
    M: NodeGraphCanvasMiddleware,
    Cx: pointer_wheel_cx::PointerWheelCx<H, M>,
{
    pointer_wheel_viewport::stop_scroll_viewport_motion(canvas, cx, snapshot);
    if searcher::handle_searcher_wheel(canvas, cx, delta, modifiers, zoom) {
        return;
    }

    if pointer_wheel_viewport::handle_scroll_zoom(
        canvas, cx, snapshot, position, delta, modifiers, zoom,
    ) {
        return;
    }

    let _ =
        pointer_wheel_viewport::handle_scroll_pan(canvas, cx, platform, snapshot, delta, modifiers);
}

pub(super) fn route_pinch_gesture<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: f32,
) where
    M: NodeGraphCanvasMiddleware,
    Cx: viewport_motion_cx::ViewportMotionCx<H>,
{
    pointer_wheel_viewport::stop_pinch_viewport_motion(canvas, cx, snapshot);
    let _ = pointer_wheel_viewport::handle_pinch_zoom(canvas, cx, snapshot, position, delta);
}
