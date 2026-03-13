use crate::ui::canvas::widget::move_ops::*;

use super::types::{Elem, ElementId, TargetBounds};

pub(in super::super) fn collect_moved_by_group(
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

pub(in super::super) fn collect_elements(
    g: &Graph,
    geom: &CanvasGeometry,
    selected_nodes: &[GraphNodeId],
    selected_groups: &[crate::core::GroupId],
) -> Vec<Elem> {
    let mut elems = Vec::new();
    for node_id in selected_nodes {
        let Some(node_geom) = geom.nodes.get(node_id) else {
            continue;
        };
        elems.push(Elem {
            id: ElementId::Node(*node_id),
            x: node_geom.rect.origin.x.0,
            y: node_geom.rect.origin.y.0,
            w: node_geom.rect.size.width.0,
            h: node_geom.rect.size.height.0,
        });
    }
    for group_id in selected_groups {
        let Some(group) = g.groups.get(group_id) else {
            continue;
        };
        elems.push(Elem {
            id: ElementId::Group(*group_id),
            x: group.rect.origin.x,
            y: group.rect.origin.y,
            w: group.rect.size.width,
            h: group.rect.size.height,
        });
    }
    elems
}

pub(in super::super) fn compute_target_bounds(elems: &[Elem]) -> Option<TargetBounds> {
    if elems.len() < 2 {
        return None;
    }

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for elem in elems {
        min_x = min_x.min(elem.x);
        min_y = min_y.min(elem.y);
        max_x = max_x.max(elem.x + elem.w);
        max_y = max_y.max(elem.y + elem.h);
    }

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return None;
    }

    Some(TargetBounds {
        left: min_x,
        top: min_y,
        right: max_x,
        bottom: max_y,
        center_x: 0.5 * (min_x + max_x),
        center_y: 0.5 * (min_y + max_y),
    })
}

pub(in super::super) fn collect_moved_nodes(
    selected_nodes: &[GraphNodeId],
    moved_by_group: &std::collections::HashSet<GraphNodeId>,
) -> std::collections::HashSet<GraphNodeId> {
    selected_nodes
        .iter()
        .copied()
        .chain(moved_by_group.iter().copied())
        .collect()
}
