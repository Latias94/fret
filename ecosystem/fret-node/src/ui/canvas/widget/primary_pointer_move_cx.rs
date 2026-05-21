use fret_ui::UiHost;

use super::{
    group_preview_move_cx::GroupPreviewMoveCx, marquee_cx::MarqueeCx,
    pending_group_activation_cx::PendingGroupActivationCx,
    pending_node_drag_activation_cx::PendingNodeDragActivationCx,
    wire_drag_move_cx::WireDragMoveCx,
};

pub(super) trait PrimaryPointerMoveCx<H: UiHost>:
    MarqueeCx<H>
    + PendingGroupActivationCx<H>
    + GroupPreviewMoveCx<H>
    + PendingNodeDragActivationCx<H>
    + WireDragMoveCx<H>
{
}

impl<H, T> PrimaryPointerMoveCx<H> for T
where
    H: UiHost,
    T: MarqueeCx<H>
        + PendingGroupActivationCx<H>
        + GroupPreviewMoveCx<H>
        + PendingNodeDragActivationCx<H>
        + WireDragMoveCx<H>,
{
}
