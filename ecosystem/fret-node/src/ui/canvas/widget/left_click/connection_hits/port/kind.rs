use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::core::{EdgeId, PortId};
use crate::rules::EdgeEndpoint;
use crate::ui::canvas::state::{PendingWireDrag, ViewSnapshot, WireDragKind};
use crate::ui::canvas::widget::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, paint_invalidation::invalidate_paint,
};

type YankedReconnectEdges = Vec<(EdgeId, EdgeEndpoint, PortId)>;

pub(super) fn wire_drag_kind_for_port_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    modifiers: Modifiers,
    port: PortId,
) -> WireDragKind {
    let yank = should_yank_edges(modifiers)
        .then(|| canvas.yank_reconnectable_edges_from_port(host, &snapshot.interaction, port));
    wire_drag_kind_from_yanked_edges(port, yank)
}

pub(super) fn arm_pending_wire_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    kind: WireDragKind,
    position: Point,
) {
    canvas.interaction.pending_wire_drag = Some(PendingWireDrag {
        kind,
        start_pos: position,
    });
    cx.capture_pointer(cx.node);
    invalidate_paint(cx);
}

fn should_yank_edges(modifiers: Modifiers) -> bool {
    modifiers.ctrl || modifiers.meta
}

fn wire_drag_kind_from_yanked_edges(
    port: PortId,
    yank: Option<YankedReconnectEdges>,
) -> WireDragKind {
    match yank {
        Some(edges) if edges.len() > 1 => WireDragKind::ReconnectMany { edges },
        Some(mut edges) if edges.len() == 1 => {
            let (edge, endpoint, fixed) = edges.remove(0);
            WireDragKind::Reconnect {
                edge,
                endpoint,
                fixed,
            }
        }
        _ => WireDragKind::New {
            from: port,
            bundle: vec![port],
        },
    }
}

#[cfg(test)]
mod tests;
