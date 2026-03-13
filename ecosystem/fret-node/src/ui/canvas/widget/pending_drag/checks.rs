use fret_core::Point;
use fret_ui::UiHost;

use crate::core::NodeId as GraphNodeId;
use crate::ui::canvas::state::{PendingNodeDrag, ViewSnapshot};
use crate::ui::canvas::widget::*;

pub(super) fn pending_drag_threshold_exceeded(
    pending: &PendingNodeDrag,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    let threshold_screen = snapshot.interaction.node_drag_threshold;
    super::super::threshold::exceeds_drag_threshold(
        pending.start_pos,
        position,
        threshold_screen,
        zoom,
    )
}

pub(super) fn primary_node_is_draggable<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    node_id: GraphNodeId,
) -> bool {
    canvas
        .graph
        .read_ref(host, |g| {
            NodeGraphCanvasWith::<M>::node_is_draggable(g, &snapshot.interaction, node_id)
        })
        .ok()
        .unwrap_or(false)
}
