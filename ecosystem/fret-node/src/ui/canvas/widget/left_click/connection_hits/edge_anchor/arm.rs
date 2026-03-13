use fret_core::Point;
use fret_ui::UiHost;

use crate::core::{EdgeId, PortId};
use crate::rules::EdgeEndpoint;
use crate::ui::canvas::state::{PendingWireDrag, WireDragKind};
use crate::ui::canvas::widget::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, paint_invalidation::invalidate_paint,
};

pub(super) fn arm_edge_anchor_reconnect<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    edge: EdgeId,
    endpoint: EdgeEndpoint,
    fixed: PortId,
    position: Point,
) {
    canvas.interaction.pending_wire_drag = Some(PendingWireDrag {
        kind: WireDragKind::Reconnect {
            edge,
            endpoint,
            fixed,
        },
        start_pos: position,
    });
    cx.capture_pointer(cx.node);
    invalidate_paint(cx);
}
