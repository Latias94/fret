mod group;
mod node;

use fret_ui::UiHost;

use super::super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, pointer_up_commit_cx::PointerUpCommitCx,
};

pub(in super::super) fn handle_node_resize_release<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: PointerUpCommitCx<H>,
{
    node::handle_node_resize_release(canvas, cx)
}

pub(in super::super) fn handle_group_resize_release<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: PointerUpCommitCx<H>,
{
    group::handle_group_resize_release(canvas, cx)
}
