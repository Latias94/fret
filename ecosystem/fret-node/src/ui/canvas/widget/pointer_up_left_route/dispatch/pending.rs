use fret_core::Point;
use fret_ui::UiHost;

use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith,
    pending_node_drag_release_cx::PendingNodeDragReleaseCx,
};

pub(super) fn handle_pending_release_chain<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: PendingNodeDragReleaseCx<H>,
{
    super::super::super::pointer_up_pending::handle_pending_group_drag_release(canvas, cx)
        || super::super::super::pointer_up_pending::handle_pending_group_resize_release(canvas, cx)
        || super::super::super::pointer_up_pending::handle_pending_node_drag_release(
            canvas, cx, snapshot, position, zoom,
        )
        || super::super::super::pointer_up_pending::handle_pending_node_resize_release(canvas, cx)
        || super::super::super::pointer_up_pending::handle_pending_wire_drag_release(
            canvas, cx, snapshot, position,
        )
}
