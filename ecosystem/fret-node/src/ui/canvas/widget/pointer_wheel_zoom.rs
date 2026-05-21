mod apply;
mod pinch;
mod wheel;

use super::*;

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
    wheel::handle_scroll_zoom(canvas, cx, snapshot, position, delta, modifiers, zoom)
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
    pinch::handle_pinch_zoom(canvas, cx, snapshot, position, delta)
}
