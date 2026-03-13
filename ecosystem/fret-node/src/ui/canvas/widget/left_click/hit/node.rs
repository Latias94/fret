use fret_core::{Point, Rect};

use crate::core::{Graph, NodeId as GraphNodeId};
use crate::ui::canvas::state::{NodeResizeHandle, ViewSnapshot};
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

use super::Hit;

pub(super) fn compute_node_hit<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    graph: &Graph,
    snapshot: &ViewSnapshot,
    node: GraphNodeId,
    rect: Rect,
    position: Point,
    zoom: f32,
) -> Hit {
    let is_selected = snapshot.selected_nodes.iter().any(|id| *id == node);
    if is_selected {
        let resize_handles = canvas
            .presenter
            .node_resize_handles(graph, node, &canvas.style);
        for handle in NodeResizeHandle::ALL {
            if !resize_handles.contains(handle) {
                continue;
            }
            let hit_rect = canvas.node_resize_handle_rect(rect, handle, zoom);
            if NodeGraphCanvasWith::<M>::rect_contains(hit_rect, position) {
                return Hit::Resize(node, rect, handle);
            }
        }
    }

    Hit::Node(node, rect)
}
