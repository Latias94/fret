use super::*;

pub(super) fn update_pointer_move_cursors<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    super::super::cursor::update_cursors(canvas, cx, snapshot, position, zoom);
}
