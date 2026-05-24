use super::*;

pub(super) fn stop_scroll_viewport_motion<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
) where
    M: NodeGraphCanvasMiddleware,
    Cx: viewport_motion_cx::ViewportMotionCx<H>,
{
    super::pointer_wheel_motion::stop_scroll_viewport_motion(canvas, cx, snapshot)
}

pub(super) fn stop_pinch_viewport_motion<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
) where
    M: NodeGraphCanvasMiddleware,
    Cx: viewport_motion_cx::ViewportMotionCx<H>,
{
    super::pointer_wheel_motion::stop_pinch_viewport_motion(canvas, cx, snapshot)
}

pub(super) fn handle_scroll_zoom<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: viewport_motion_cx::ViewportMotionCx<H>,
{
    super::pointer_wheel_zoom::handle_scroll_zoom(
        canvas, cx, snapshot, position, delta, modifiers, zoom,
    )
}

pub(super) fn handle_scroll_pan<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    platform: fret_runtime::Platform,
    snapshot: &ViewSnapshot,
    delta: Point,
    modifiers: fret_core::Modifiers,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: viewport_motion_cx::ViewportMotionCx<H>,
{
    super::pointer_wheel_pan::handle_scroll_pan(canvas, cx, platform, snapshot, delta, modifiers)
}

pub(super) fn handle_pinch_zoom<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: viewport_motion_cx::ViewportMotionCx<H>,
{
    super::pointer_wheel_zoom::handle_pinch_zoom(canvas, cx, snapshot, position, delta)
}
