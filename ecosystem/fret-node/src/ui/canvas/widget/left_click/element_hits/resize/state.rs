use fret_core::{Point, Rect};
use fret_ui::UiHost;

use super::super::super::{LeftClickCx, capture_pointer_and_invalidate_paint};
use crate::core::{CanvasSize, NodeId as GraphNodeId};
use crate::ui::canvas::state::{NodeResizeHandle, PendingNodeResize};
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn arm_pending_node_resize<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl LeftClickCx<H>,
    node: GraphNodeId,
    rect: Rect,
    handle: NodeResizeHandle,
    position: Point,
    zoom: f32,
) {
    let pending = pending_node_resize_from_hit(
        canvas,
        cx.left_click_host(),
        node,
        rect,
        handle,
        position,
        zoom,
    );
    canvas.interaction.pending_node_resize = Some(pending);
    capture_pointer_and_invalidate_paint(cx);
}

fn pending_node_resize_from_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
    node: GraphNodeId,
    rect: Rect,
    handle: NodeResizeHandle,
    position: Point,
    zoom: f32,
) -> PendingNodeResize {
    let start_size = start_size_from_rect(rect, zoom);
    let (start_node_pos, start_size_opt) = canvas
        .graph
        .read_ref(host, |graph| {
            graph
                .nodes
                .get(&node)
                .map(|node| (node.pos, node.size))
                .unwrap_or_default()
        })
        .ok()
        .unwrap_or_default();

    PendingNodeResize {
        node,
        handle,
        start_pos: position,
        start_node_pos,
        start_size,
        start_size_opt,
    }
}

fn start_size_from_rect(rect: Rect, zoom: f32) -> CanvasSize {
    CanvasSize {
        width: rect.size.width.0 * zoom,
        height: rect.size.height.0 * zoom,
    }
}

#[cfg(test)]
mod tests;
