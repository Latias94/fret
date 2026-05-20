mod active;
mod pending;

use super::prelude::*;
use crate::ui::canvas::widget::pointer_up_release_cx::PointerUpReleaseCx;

pub(in super::super) fn handle_edge_insert_left_up<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    position: Point,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: PointerUpReleaseCx<H>,
{
    if pending::handle_pending_edge_insert_left_up(canvas, cx) {
        return true;
    }

    active::handle_active_edge_insert_left_up(canvas, cx, position)
}
