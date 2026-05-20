use fret_ui::UiHost;

use super::{
    NodeGraphCanvasMiddleware, marquee_cx::MarqueeCx,
    pending_node_drag_release_cx::PendingNodeDragReleaseCx,
    pointer_up_commit_cx::PointerUpCommitCx, pointer_up_release_cx::PointerUpReleaseCx,
    wire_drag::WireCommitCx,
};

pub(super) trait PointerUpCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    PointerUpReleaseCx<H>
    + PointerUpCommitCx<H>
    + PendingNodeDragReleaseCx<H>
    + WireCommitCx<H>
    + MarqueeCx<H>
{
}

impl<H, M, T> PointerUpCx<H, M> for T
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    T: PointerUpReleaseCx<H>
        + PointerUpCommitCx<H>
        + PendingNodeDragReleaseCx<H>
        + WireCommitCx<H>
        + MarqueeCx<H>,
{
}
