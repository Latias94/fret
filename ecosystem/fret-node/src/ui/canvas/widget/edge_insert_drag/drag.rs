use super::prelude::*;

pub(in super::super) fn handle_edge_insert_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
) -> bool {
    let Some(mut drag) = canvas.interaction.edge_insert_drag.clone() else {
        return false;
    };
    drag.pos = position;
    canvas.interaction.edge_insert_drag = Some(drag);
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
    true
}
