use fret_core::Rect;
use fret_ui::UiHost;

use super::{
    node_drag_geometry_cx::NodeDragGeometryCx, node_drag_move_tail_cx::NodeDragMoveTailCx,
    node_drag_preview_cx::NodeDragPreviewCx,
};

pub(super) trait NodeDragMoveCx<H: UiHost>:
    NodeDragGeometryCx<H> + NodeDragPreviewCx<H> + NodeDragMoveTailCx<H>
{
    fn bounds(&self) -> Rect;
}
