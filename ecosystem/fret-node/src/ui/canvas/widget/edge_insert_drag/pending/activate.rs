use fret_core::Point;
use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::super::super::paint_invalidation::invalidate_paint;
use super::super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::PendingEdgeInsertDrag;

pub(super) fn activate_pending_edge_insert_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    pending: PendingEdgeInsertDrag,
    position: Point,
) {
    super::super::super::pending_connection_session::activate_pending_edge_insert_drag(
        &mut canvas.interaction,
        pending,
        position,
    );
    invalidate_paint(cx);
}
