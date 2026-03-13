use crate::ui::canvas::widget::move_ops::*;

pub(super) fn plan_nudge_ops(
    g: &Graph,
    geom: &CanvasGeometry,
    selected_nodes: &[GraphNodeId],
    selected_groups: &[crate::core::GroupId],
    snapshot: &ViewSnapshot,
    delta: CanvasPoint,
) -> Vec<GraphOp> {
    let selected_groups_set: std::collections::HashSet<crate::core::GroupId> =
        selected_groups.iter().copied().collect();
    let moved_by_group = collect_moved_by_group(g, &selected_groups_set);
    let moved_nodes = collect_moved_nodes(selected_nodes, &moved_by_group);
    let shared_move = !selected_groups.is_empty() || moved_nodes.len() > 1;

    let delta =
        clamp_shared_delta_to_global_extent(g, geom, snapshot, &moved_nodes, delta, shared_move);
    let delta = clamp_shared_delta_to_node_extents(g, geom, &moved_nodes, delta, shared_move);

    let mut ops = Vec::new();
    append_group_ops(&mut ops, g, selected_groups, delta);
    append_node_ops(
        &mut ops,
        g,
        geom,
        &moved_nodes,
        &moved_by_group,
        snapshot,
        delta,
        shared_move,
    );
    ops
}

fn collect_moved_by_group(
    g: &Graph,
    selected_groups_set: &std::collections::HashSet<crate::core::GroupId>,
) -> std::collections::HashSet<GraphNodeId> {
    let mut moved_by_group = std::collections::HashSet::new();
    for (&node_id, node) in &g.nodes {
        if let Some(parent) = node.parent
            && selected_groups_set.contains(&parent)
        {
            moved_by_group.insert(node_id);
        }
    }
    moved_by_group
}

fn collect_moved_nodes(
    selected_nodes: &[GraphNodeId],
    moved_by_group: &std::collections::HashSet<GraphNodeId>,
) -> std::collections::BTreeSet<GraphNodeId> {
    let mut moved_nodes: std::collections::BTreeSet<GraphNodeId> =
        selected_nodes.iter().copied().collect();
    for node_id in moved_by_group {
        moved_nodes.insert(*node_id);
    }
    moved_nodes
}

fn clamp_shared_delta_to_global_extent(
    g: &Graph,
    geom: &CanvasGeometry,
    snapshot: &ViewSnapshot,
    moved_nodes: &std::collections::BTreeSet<GraphNodeId>,
    mut delta: CanvasPoint,
    shared_move: bool,
) -> CanvasPoint {
    if !shared_move {
        return delta;
    }
    let Some(extent) = snapshot.interaction.node_extent else {
        return delta;
    };

    let mut min_x: f32 = f32::INFINITY;
    let mut min_y: f32 = f32::INFINITY;
    let mut max_x: f32 = f32::NEG_INFINITY;
    let mut max_y: f32 = f32::NEG_INFINITY;
    let mut any = false;

    for node_id in moved_nodes.iter().copied() {
        let Some(node) = g.nodes.get(&node_id) else {
            continue;
        };

        let (x0, y0, w, h) = if let Some(node_geom) = geom.nodes.get(&node_id) {
            (
                node_geom.rect.origin.x.0,
                node_geom.rect.origin.y.0,
                node_geom.rect.size.width.0.max(0.0),
                node_geom.rect.size.height.0.max(0.0),
            )
        } else if let Some(size) = node.size {
            (
                node.pos.x,
                node.pos.y,
                size.width.max(0.0),
                size.height.max(0.0),
            )
        } else {
            continue;
        };
        if !x0.is_finite() || !y0.is_finite() || !w.is_finite() || !h.is_finite() {
            continue;
        }

        any = true;
        min_x = min_x.min(x0);
        min_y = min_y.min(y0);
        max_x = max_x.max(x0 + w);
        max_y = max_y.max(y0 + h);
    }

    if any && min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
        let group_w = (max_x - min_x).max(0.0);
        let group_h = (max_y - min_y).max(0.0);
        let extent_w = extent.size.width.max(0.0);
        let extent_h = extent.size.height.max(0.0);

        let min_dx = extent.origin.x - min_x;
        let mut max_dx = extent.origin.x + (extent_w - group_w) - min_x;
        if !max_dx.is_finite() || max_dx < min_dx {
            max_dx = min_dx;
        }
        delta.x = delta.x.clamp(min_dx, max_dx);

        let min_dy = extent.origin.y - min_y;
        let mut max_dy = extent.origin.y + (extent_h - group_h) - min_y;
        if !max_dy.is_finite() || max_dy < min_dy {
            max_dy = min_dy;
        }
        delta.y = delta.y.clamp(min_dy, max_dy);
    }

    delta
}

fn clamp_shared_delta_to_node_extents(
    g: &Graph,
    geom: &CanvasGeometry,
    moved_nodes: &std::collections::BTreeSet<GraphNodeId>,
    mut delta: CanvasPoint,
    shared_move: bool,
) -> CanvasPoint {
    if !shared_move {
        return delta;
    }

    let mut min_dx: f32 = f32::NEG_INFINITY;
    let mut max_dx: f32 = f32::INFINITY;
    let mut min_dy: f32 = f32::NEG_INFINITY;
    let mut max_dy: f32 = f32::INFINITY;
    let mut any_x = false;
    let mut any_y = false;

    for node_id in moved_nodes.iter().copied() {
        let Some(node) = g.nodes.get(&node_id) else {
            continue;
        };
        let Some(crate::core::NodeExtent::Rect { rect }) = node.extent else {
            continue;
        };

        let Some(node_size) = node_size_for_ops(geom, node_id, node) else {
            continue;
        };
        let node_w = node_size.width.max(0.0);
        let node_h = node_size.height.max(0.0);
        if !node_w.is_finite() || !node_h.is_finite() {
            continue;
        }

        let min_x = rect.origin.x;
        let max_x = rect.origin.x + (rect.size.width - node_w).max(0.0);
        if min_x.is_finite() && max_x.is_finite() && node.pos.x.is_finite() {
            any_x = true;
            min_dx = min_dx.max(min_x - node.pos.x);
            max_dx = max_dx.min(max_x - node.pos.x);
        }

        let min_y = rect.origin.y;
        let max_y = rect.origin.y + (rect.size.height - node_h).max(0.0);
        if min_y.is_finite() && max_y.is_finite() && node.pos.y.is_finite() {
            any_y = true;
            min_dy = min_dy.max(min_y - node.pos.y);
            max_dy = max_dy.min(max_y - node.pos.y);
        }
    }

    if any_x && min_dx.is_finite() && max_dx.is_finite() {
        if max_dx < min_dx {
            max_dx = min_dx;
        }
        delta.x = delta.x.clamp(min_dx, max_dx);
    }
    if any_y && min_dy.is_finite() && max_dy.is_finite() {
        if max_dy < min_dy {
            max_dy = min_dy;
        }
        delta.y = delta.y.clamp(min_dy, max_dy);
    }

    delta
}

fn append_group_ops(
    ops: &mut Vec<GraphOp>,
    g: &Graph,
    selected_groups: &[crate::core::GroupId],
    delta: CanvasPoint,
) {
    let mut groups_sorted = selected_groups.to_vec();
    groups_sorted.sort();
    for group_id in groups_sorted {
        let Some(group) = g.groups.get(&group_id) else {
            continue;
        };
        let from = group.rect;
        let to = crate::core::CanvasRect {
            origin: CanvasPoint {
                x: from.origin.x + delta.x,
                y: from.origin.y + delta.y,
            },
            size: from.size,
        };
        if from != to {
            ops.push(GraphOp::SetGroupRect {
                id: group_id,
                from,
                to,
            });
        }
    }
}

fn append_node_ops(
    ops: &mut Vec<GraphOp>,
    g: &Graph,
    geom: &CanvasGeometry,
    moved_nodes: &std::collections::BTreeSet<GraphNodeId>,
    moved_by_group: &std::collections::HashSet<GraphNodeId>,
    snapshot: &ViewSnapshot,
    delta: CanvasPoint,
    shared_move: bool,
) {
    let node_origin = snapshot.interaction.node_origin.normalized();
    for node_id in moved_nodes.iter().copied() {
        let Some(node) = g.nodes.get(&node_id) else {
            continue;
        };
        let from = node.pos;
        let mut to = CanvasPoint {
            x: from.x + delta.x,
            y: from.y + delta.y,
        };

        if !moved_by_group.contains(&node_id) {
            let Some(node_size) = node_size_for_ops(geom, node_id, node) else {
                continue;
            };
            let node_w = node_size.width;
            let node_h = node_size.height;

            if !shared_move && let Some(extent) = snapshot.interaction.node_extent {
                let min_x = extent.origin.x;
                let min_y = extent.origin.y;
                let max_x = extent.origin.x + (extent.size.width - node_w).max(0.0);
                let max_y = extent.origin.y + (extent.size.height - node_h).max(0.0);
                to = clamp_node_anchor_to_bounds(
                    to,
                    node_size,
                    node_origin,
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                );
            }

            if let Some(crate::core::NodeExtent::Rect { rect }) = node.extent {
                let min_x = rect.origin.x;
                let min_y = rect.origin.y;
                let max_x = rect.origin.x + (rect.size.width - node_w).max(0.0);
                let max_y = rect.origin.y + (rect.size.height - node_h).max(0.0);
                to = clamp_node_anchor_to_bounds(
                    to,
                    node_size,
                    node_origin,
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                );
            }

            if let Some(parent) = node.parent
                && let Some(group) = g.groups.get(&parent)
            {
                let min_x = group.rect.origin.x;
                let min_y = group.rect.origin.y;
                let max_x = group.rect.origin.x + (group.rect.size.width - node_w).max(0.0);
                let max_y = group.rect.origin.y + (group.rect.size.height - node_h).max(0.0);
                to = clamp_node_anchor_to_bounds(
                    to,
                    node_size,
                    node_origin,
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                );
            }
        }

        if from != to {
            ops.push(GraphOp::SetNodePos {
                id: node_id,
                from,
                to,
            });
        }
    }
}

fn node_size_for_ops(
    geom: &CanvasGeometry,
    node_id: GraphNodeId,
    node: &crate::core::Node,
) -> Option<CanvasSize> {
    if let Some(node_geom) = geom.nodes.get(&node_id) {
        Some(CanvasSize {
            width: node_geom.rect.size.width.0,
            height: node_geom.rect.size.height.0,
        })
    } else {
        node.size
    }
}

fn clamp_node_anchor_to_bounds(
    anchor: CanvasPoint,
    node_size: CanvasSize,
    node_origin: crate::io::NodeGraphNodeOrigin,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
) -> CanvasPoint {
    let mut rect_origin = node_rect_origin_from_anchor(anchor, node_size, node_origin);
    rect_origin.x = rect_origin.x.clamp(min_x, max_x);
    rect_origin.y = rect_origin.y.clamp(min_y, max_y);
    node_anchor_from_rect_origin(rect_origin, node_size, node_origin)
}
