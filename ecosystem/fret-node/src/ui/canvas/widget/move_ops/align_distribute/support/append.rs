use crate::ui::canvas::widget::move_ops::*;

pub(in super::super) fn append_group_ops(
    ops: &mut Vec<GraphOp>,
    g: &Graph,
    geom: &CanvasGeometry,
    selected_groups: &[crate::core::GroupId],
    snapshot: &ViewSnapshot,
    per_group_delta: &std::collections::HashMap<crate::core::GroupId, CanvasPoint>,
    shift: CanvasPoint,
    aligns: bool,
) {
    let mut groups_sorted = selected_groups.to_vec();
    groups_sorted.sort();
    for group_id in groups_sorted {
        let base = per_group_delta.get(&group_id).copied().unwrap_or_default();
        let mut delta = CanvasPoint {
            x: base.x + shift.x,
            y: base.y + shift.y,
        };

        delta = clamp_group_delta_to_child_extents(g, geom, group_id, delta);
        if !aligns {
            delta = clamp_group_delta_to_global_extent(g, geom, group_id, snapshot, delta);
        }

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

        if delta.x.abs() <= 1.0e-9 && delta.y.abs() <= 1.0e-9 {
            continue;
        }
        for (&node_id, node) in &g.nodes {
            if node.parent != Some(group_id) {
                continue;
            }
            let from = node.pos;
            let to = CanvasPoint {
                x: from.x + delta.x,
                y: from.y + delta.y,
            };
            if from != to {
                ops.push(GraphOp::SetNodePos {
                    id: node_id,
                    from,
                    to,
                });
            }
        }
    }
}

pub(in super::super) fn append_node_ops(
    ops: &mut Vec<GraphOp>,
    g: &Graph,
    geom: &CanvasGeometry,
    selected_nodes: &[GraphNodeId],
    moved_by_group: &std::collections::HashSet<GraphNodeId>,
    snapshot: &ViewSnapshot,
    per_node_delta: &std::collections::HashMap<GraphNodeId, CanvasPoint>,
    shift: CanvasPoint,
    skip_node_extent_clamp: bool,
) {
    let node_origin = snapshot.interaction.node_origin.normalized();
    let mut nodes_sorted = selected_nodes.to_vec();
    nodes_sorted.sort();
    for node_id in nodes_sorted {
        if moved_by_group.contains(&node_id) {
            continue;
        }
        let base = per_node_delta.get(&node_id).copied().unwrap_or_default();
        let delta = CanvasPoint {
            x: base.x + shift.x,
            y: base.y + shift.y,
        };
        let moved = delta.x.abs() > 1.0e-9 || delta.y.abs() > 1.0e-9;
        let Some(node) = g.nodes.get(&node_id) else {
            continue;
        };
        let from = node.pos;
        let mut to = CanvasPoint {
            x: from.x + delta.x,
            y: from.y + delta.y,
        };

        let node_size = if let Some(node_geom) = geom.nodes.get(&node_id) {
            Some(CanvasSize {
                width: node_geom.rect.size.width.0,
                height: node_geom.rect.size.height.0,
            })
        } else {
            node.size
        };

        if let Some(node_size) = node_size {
            let node_w = node_size.width;
            let node_h = node_size.height;

            if moved
                && !skip_node_extent_clamp
                && let Some(extent) = snapshot.interaction.node_extent
            {
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

            if moved && let Some(crate::core::NodeExtent::Rect { rect }) = node.extent {
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

fn clamp_group_delta_to_child_extents(
    g: &Graph,
    geom: &CanvasGeometry,
    group_id: crate::core::GroupId,
    mut delta: CanvasPoint,
) -> CanvasPoint {
    if delta.x.abs() <= 1.0e-9 && delta.y.abs() <= 1.0e-9 {
        return delta;
    }

    let mut min_dx: f32 = f32::NEG_INFINITY;
    let mut max_dx: f32 = f32::INFINITY;
    let mut min_dy: f32 = f32::NEG_INFINITY;
    let mut max_dy: f32 = f32::INFINITY;
    let mut any_x = false;
    let mut any_y = false;

    for (&node_id, node) in &g.nodes {
        if node.parent != Some(group_id) {
            continue;
        }
        let Some(crate::core::NodeExtent::Rect { rect }) = node.extent else {
            continue;
        };

        let node_size = if let Some(node_geom) = geom.nodes.get(&node_id) {
            Some(CanvasSize {
                width: node_geom.rect.size.width.0,
                height: node_geom.rect.size.height.0,
            })
        } else {
            node.size
        };
        let Some(node_size) = node_size else {
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

fn clamp_group_delta_to_global_extent(
    g: &Graph,
    geom: &CanvasGeometry,
    group_id: crate::core::GroupId,
    snapshot: &ViewSnapshot,
    mut delta: CanvasPoint,
) -> CanvasPoint {
    if delta.x.abs() <= 1.0e-9 && delta.y.abs() <= 1.0e-9 {
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

    for (&node_id, node) in &g.nodes {
        if node.parent != Some(group_id) {
            continue;
        }

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

    if !any || !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite()
    {
        return delta;
    }

    let bbox_w = (max_x - min_x).max(0.0);
    let bbox_h = (max_y - min_y).max(0.0);
    let extent_w = extent.size.width.max(0.0);
    let extent_h = extent.size.height.max(0.0);

    if min_x.is_finite()
        && bbox_w.is_finite()
        && extent.origin.x.is_finite()
        && extent_w.is_finite()
    {
        let min_dx = extent.origin.x - min_x;
        let mut max_dx = extent.origin.x + (extent_w - bbox_w).max(0.0) - min_x;
        if !max_dx.is_finite() || max_dx < min_dx {
            max_dx = min_dx;
        }
        delta.x = delta.x.clamp(min_dx, max_dx);
    }

    if min_y.is_finite()
        && bbox_h.is_finite()
        && extent.origin.y.is_finite()
        && extent_h.is_finite()
    {
        let min_dy = extent.origin.y - min_y;
        let mut max_dy = extent.origin.y + (extent_h - bbox_h).max(0.0) - min_y;
        if !max_dy.is_finite() || max_dy < min_dy {
            max_dy = min_dy;
        }
        delta.y = delta.y.clamp(min_dy, max_dy);
    }

    delta
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
