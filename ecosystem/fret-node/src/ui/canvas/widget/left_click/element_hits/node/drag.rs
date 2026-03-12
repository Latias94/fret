use fret_core::Point;
use fret_ui::UiHost;

use crate::core::NodeId as GraphNodeId;
use crate::interaction::NodeGraphDragHandleMode;
use crate::ui::canvas::state::{PendingNodeDrag, PendingNodeSelectAction, ViewSnapshot};
use crate::ui::canvas::widget::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, paint_invalidation::invalidate_paint,
};

pub(super) fn drag_nodes_for_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    node: GraphNodeId,
    node_selectable: bool,
    node_draggable: bool,
    already_selected: bool,
) -> Vec<GraphNodeId> {
    let candidate_nodes = base_drag_nodes(
        node,
        node_selectable,
        node_draggable,
        already_selected,
        &snapshot.selected_nodes,
    );
    canvas
        .graph
        .read_ref(host, |graph| {
            candidate_nodes
                .iter()
                .copied()
                .filter(|id| {
                    NodeGraphCanvasWith::<M>::node_is_draggable(graph, &snapshot.interaction, *id)
                })
                .collect::<Vec<_>>()
        })
        .ok()
        .unwrap_or_else(|| vec![node])
}

pub(super) fn drag_enabled_for_node_hit(
    mode: NodeGraphDragHandleMode,
    header_hit: bool,
    node_draggable: bool,
) -> bool {
    let handle_enabled = match mode {
        NodeGraphDragHandleMode::Any => true,
        NodeGraphDragHandleMode::Header => header_hit,
    };
    handle_enabled && node_draggable
}

pub(super) fn arm_pending_node_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    primary: GraphNodeId,
    nodes: Vec<GraphNodeId>,
    grab_offset: Point,
    start_pos: Point,
    select_action: PendingNodeSelectAction,
    drag_enabled: bool,
) {
    canvas.interaction.pending_node_drag = Some(PendingNodeDrag {
        primary,
        nodes,
        grab_offset,
        start_pos,
        select_action,
        drag_enabled,
    });
    cx.capture_pointer(cx.node);
    invalidate_paint(cx);
}

fn base_drag_nodes(
    node: GraphNodeId,
    node_selectable: bool,
    node_draggable: bool,
    already_selected: bool,
    selected_nodes: &[GraphNodeId],
) -> Vec<GraphNodeId> {
    if node_draggable && node_selectable && already_selected && selected_nodes.len() > 1 {
        selected_nodes.to_vec()
    } else {
        vec![node]
    }
}

#[cfg(test)]
mod tests;
