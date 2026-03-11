use std::collections::HashMap;

use fret_ui::UiHost;

use crate::core::{
    CanvasPoint, CanvasRect, CanvasSize, GroupId, NodeExtent, NodeId as GraphNodeId,
};
use crate::io::NodeGraphNodeOrigin;
use crate::ui::canvas::geometry::{CanvasGeometry, node_rect_origin_from_anchor};
use crate::ui::canvas::state::NodeDrag;

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

pub(super) fn compute_preview_positions<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    drag: &NodeDrag,
    delta: CanvasPoint,
    multi_drag: bool,
) -> (Vec<(GraphNodeId, CanvasPoint)>, Vec<(GroupId, CanvasRect)>) {
    let geometry = canvas.canvas_geometry(&*cx.app, snapshot);
    let node_origin = snapshot.interaction.node_origin.normalized();
    canvas
        .graph
        .read_ref(cx.app, |graph| {
            let mut next_nodes: Vec<(GraphNodeId, CanvasPoint)> =
                Vec::with_capacity(drag.nodes.len());
            let mut group_overrides: HashMap<GroupId, CanvasRect> = HashMap::new();

            for &(id, start) in &drag.nodes {
                let Some((target, group_override)) = compute_dragged_node_preview(
                    snapshot,
                    graph,
                    geometry.as_ref(),
                    id,
                    start,
                    delta,
                    multi_drag,
                    node_origin,
                ) else {
                    continue;
                };

                if let Some((group_id, rect)) = group_override {
                    merge_group_override(&mut group_overrides, group_id, rect);
                }
                next_nodes.push((id, target));
            }

            (next_nodes, sorted_group_overrides(group_overrides))
        })
        .ok()
        .unwrap_or_default()
}

fn compute_dragged_node_preview(
    snapshot: &ViewSnapshot,
    graph: &crate::core::Graph,
    geometry: &CanvasGeometry,
    node_id: GraphNodeId,
    start: CanvasPoint,
    delta: CanvasPoint,
    multi_drag: bool,
    node_origin: NodeGraphNodeOrigin,
) -> Option<(CanvasPoint, Option<(GroupId, CanvasRect)>)> {
    let node = graph.nodes.get(&node_id)?;
    let node_geom = geometry.nodes.get(&node_id)?;
    let node_size = CanvasSize {
        width: node_geom.rect.size.width.0,
        height: node_geom.rect.size.height.0,
    };
    let mut target = CanvasPoint {
        x: start.x + delta.x,
        y: start.y + delta.y,
    };

    if !multi_drag && let Some(extent) = snapshot.interaction.node_extent {
        target = clamp_target(target, node_size, extent, node_origin);
    }

    if let Some(NodeExtent::Rect { rect }) = node.extent {
        target = clamp_target(target, node_size, rect, node_origin);
    }

    let (target, parent_override) =
        apply_parent_constraint_or_expansion(graph, node, target, node_size, node_origin);
    Some((target, parent_override))
}

fn apply_parent_constraint_or_expansion(
    graph: &crate::core::Graph,
    node: &crate::core::Node,
    target: CanvasPoint,
    node_size: CanvasSize,
    node_origin: NodeGraphNodeOrigin,
) -> (CanvasPoint, Option<(GroupId, CanvasRect)>) {
    let Some(parent) = node.parent else {
        return (target, None);
    };
    let Some(parent_rect) = graph.groups.get(&parent).map(|group| group.rect) else {
        return (target, None);
    };
    let expand_parent = node.expand_parent.unwrap_or(false);
    let clamp_to_parent = !expand_parent
        && matches!(
            node.extent,
            Some(NodeExtent::Parent) | None | Some(NodeExtent::Rect { .. })
        );

    if clamp_to_parent {
        return (
            clamp_target(target, node_size, parent_rect, node_origin),
            None,
        );
    }

    if expand_parent {
        let rect_origin = node_rect_origin_from_anchor(target, node_size, node_origin);
        let child_rect = CanvasRect {
            origin: rect_origin,
            size: node_size,
        };
        let next_rect = super::super::node_drag_constraints::union_rect(parent_rect, child_rect);
        return (target, Some((parent, next_rect)));
    }

    (target, None)
}

fn clamp_target(
    target: CanvasPoint,
    node_size: CanvasSize,
    rect: CanvasRect,
    node_origin: NodeGraphNodeOrigin,
) -> CanvasPoint {
    super::super::node_drag_constraints::clamp_anchor_in_rect_with_size(
        target,
        node_size,
        rect,
        node_origin,
    )
}

fn merge_group_override(
    group_overrides: &mut HashMap<GroupId, CanvasRect>,
    group_id: GroupId,
    rect: CanvasRect,
) {
    group_overrides
        .entry(group_id)
        .and_modify(|existing| {
            *existing = super::super::node_drag_constraints::union_rect(*existing, rect)
        })
        .or_insert(rect);
}

fn sorted_group_overrides(
    group_overrides: HashMap<GroupId, CanvasRect>,
) -> Vec<(GroupId, CanvasRect)> {
    let mut next_groups: Vec<(GroupId, CanvasRect)> = group_overrides.into_iter().collect();
    next_groups.sort_by(|a, b| a.0.cmp(&b.0));
    next_groups
}
