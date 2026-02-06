use super::prelude::*;

pub(in super::super) fn handle_edge_insert_left_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
) -> bool {
    if canvas.interaction.pending_edge_insert_drag.take().is_some() {
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
        return true;
    }

    if let Some(drag) = canvas.interaction.edge_insert_drag.take() {
        if canvas.interaction.searcher.is_none() && canvas.interaction.context_menu.is_none() {
            canvas.open_edge_insert_node_picker(cx.app, cx.window, drag.edge, position);
        }
        canvas.interaction.hover_edge = None;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
        return true;
    }

    false
}
