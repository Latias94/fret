mod group_drag;
mod resize;

use fret_ui::UiHost;

use super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, pointer_up_commit_cx::PointerUpCommitCx,
};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) use group_drag::handle_group_drag_release;
pub(super) use resize::{handle_group_resize_release, handle_node_resize_release};

pub(super) fn handle_node_drag_release<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: PointerUpCommitCx<H>,
{
    super::pointer_up_node_drag::handle_node_drag_release(canvas, cx, snapshot)
}
