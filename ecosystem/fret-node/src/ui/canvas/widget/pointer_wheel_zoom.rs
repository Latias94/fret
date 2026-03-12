mod apply;
mod pinch;
mod wheel;

use super::*;

pub(super) fn handle_scroll_zoom<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    wheel::handle_scroll_zoom(canvas, cx, snapshot, position, delta, modifiers, zoom)
}

pub(super) fn handle_pinch_zoom<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: f32,
) -> bool {
    pinch::handle_pinch_zoom(canvas, cx, snapshot, position, delta)
}
