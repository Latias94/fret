use fret_core::Point;
use fret_ui::UiHost;

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::core::CanvasPoint;

pub(super) fn open_edge_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    edge_id: crate::core::EdgeId,
    position: Point,
) -> bool {
    canvas.open_edge_insert_node_picker(cx.app, cx.window, edge_id, position);
    finish_sticky_wire_target_picker(cx);
    true
}

pub(super) fn open_connection_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    from_port: crate::core::PortId,
    at: CanvasPoint,
) -> bool {
    canvas.open_connection_insert_node_picker(cx.app, from_port, at);
    finish_sticky_wire_target_picker(cx);
    true
}

fn finish_sticky_wire_target_picker<H: UiHost>(cx: &mut fret_ui::retained_bridge::EventCx<'_, H>) {
    cx.stop_propagation();
    super::super::paint_invalidation::invalidate_paint(cx);
}
