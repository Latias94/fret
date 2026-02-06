use super::prelude::*;

pub(super) fn canvas_to_window<M: NodeGraphCanvasMiddleware>(
    bounds: Rect,
    pos: Point,
    pan: CanvasPoint,
    zoom: f32,
) -> Point {
    let viewport = NodeGraphCanvasWith::<M>::viewport_from_pan_zoom(bounds, pan, zoom);
    viewport.canvas_to_screen(pos)
}
