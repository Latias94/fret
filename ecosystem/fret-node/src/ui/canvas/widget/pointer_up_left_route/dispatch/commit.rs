use fret_ui::UiHost;

use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, pointer_up_commit_cx::PointerUpCommitCx,
};

pub(super) fn handle_commit_release_chain<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: PointerUpCommitCx<H>,
{
    super::super::super::pointer_up_commit::handle_node_resize_release(canvas, cx)
        || super::super::super::pointer_up_commit::handle_group_resize_release(canvas, cx)
        || super::super::super::pointer_up_commit::handle_group_drag_release(canvas, cx)
        || super::super::super::pointer_up_commit::handle_node_drag_release(canvas, cx, snapshot)
}
