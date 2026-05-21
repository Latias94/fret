use fret_ui::UiHost;

use super::{
    edge_drag_move_cx::EdgeDragMoveCx, insert_node_drag_move_cx::InsertNodeDragMoveCx,
    node_drag_move_cx::NodeDragMoveCx, node_resize_move_cx::NodeResizeMoveCx,
    wire_drag_move_cx::WireDragMoveCx,
};

pub(super) trait SecondaryPointerMoveCx<H: UiHost>:
    NodeResizeMoveCx<H>
    + NodeDragMoveCx<H>
    + WireDragMoveCx<H>
    + EdgeDragMoveCx<H>
    + InsertNodeDragMoveCx<H>
{
}

impl<H, T> SecondaryPointerMoveCx<H> for T
where
    H: UiHost,
    T: NodeResizeMoveCx<H>
        + NodeDragMoveCx<H>
        + WireDragMoveCx<H>
        + EdgeDragMoveCx<H>
        + InsertNodeDragMoveCx<H>,
{
}
