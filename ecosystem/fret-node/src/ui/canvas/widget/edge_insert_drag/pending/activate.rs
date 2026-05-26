use fret_core::Point;
use fret_ui::UiHost;

use super::super::super::paint_invalidation::invalidate_paint;
use super::super::super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, low_level_adapter::CanvasPaintInvalidationCx,
};
use crate::ui::canvas::state::PendingEdgeInsertDrag;

pub(super) fn activate_pending_edge_insert_drag<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    pending: PendingEdgeInsertDrag,
    position: Point,
) where
    M: NodeGraphCanvasMiddleware,
    Cx: CanvasPaintInvalidationCx<H>,
{
    super::super::super::pending_connection_session::activate_pending_edge_insert_drag(
        &mut canvas.interaction,
        pending,
        position,
    );
    invalidate_paint(cx);
}
