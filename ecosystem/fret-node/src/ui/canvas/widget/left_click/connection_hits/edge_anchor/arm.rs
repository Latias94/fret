use fret_core::Point;
use fret_ui::UiHost;

use super::super::super::{LeftClickCx, capture_pointer_and_invalidate_paint};
use crate::core::{EdgeId, PortId};
use crate::rules::EdgeEndpoint;
use crate::ui::canvas::state::{PendingWireDrag, WireDragKind};
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn arm_edge_anchor_reconnect<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl LeftClickCx<H>,
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
    capture_pointer_and_invalidate_paint(cx);
}
