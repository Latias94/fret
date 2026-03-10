use fret_ui::UiHost;

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(in super::super) fn handle_pending_group_drag_release<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    super::super::pointer_up_session::finish_pending_release(
        &mut canvas.interaction.pending_group_drag,
        cx,
    )
}

pub(in super::super) fn handle_pending_group_resize_release<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    super::super::pointer_up_session::finish_pending_release(
        &mut canvas.interaction.pending_group_resize,
        cx,
    )
}

pub(in super::super) fn handle_pending_node_resize_release<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    super::super::pointer_up_session::finish_pending_release(
        &mut canvas.interaction.pending_node_resize,
        cx,
    )
}
