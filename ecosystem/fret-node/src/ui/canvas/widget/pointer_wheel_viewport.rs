use super::*;

pub(super) fn stop_scroll_viewport_motion<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
) {
    super::pointer_wheel_motion::stop_scroll_viewport_motion(canvas, cx, snapshot)
}

pub(super) fn stop_pinch_viewport_motion<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
) {
    super::pointer_wheel_motion::stop_pinch_viewport_motion(canvas, cx, snapshot)
}

pub(super) fn handle_scroll_zoom<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    super::pointer_wheel_zoom::handle_scroll_zoom(
        canvas, cx, snapshot, position, delta, modifiers, zoom,
    )
}

pub(super) fn handle_scroll_pan<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    delta: Point,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::pointer_wheel_pan::handle_scroll_pan(canvas, cx, snapshot, delta, modifiers)
}

pub(super) fn handle_pinch_zoom<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: f32,
) -> bool {
    super::pointer_wheel_zoom::handle_pinch_zoom(canvas, cx, snapshot, position, delta)
}
