use super::prelude::*;

pub(super) fn handle_edge_left_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) -> bool {
    if canvas.interaction.edge_drag.take().is_some() {
        cx.release_pointer_capture();
        invalidate_paint(cx);
        return true;
    }

    false
}
