use super::super::cursor_cx::CanvasCursorCx;
use super::*;

pub(super) fn update_pointer_move_cursors<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl CanvasCursorCx<H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    super::super::cursor::update_cursors(canvas, cx, snapshot, position, zoom);
}
