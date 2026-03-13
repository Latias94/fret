use crate::ui::canvas::widget::move_ops::*;

use super::types::ModeFlags;

pub(in super::super) fn plan_extent_shift(
    g: &Graph,
    geom: &CanvasGeometry,
    snapshot: &ViewSnapshot,
    moved_nodes: &std::collections::HashSet<GraphNodeId>,
    moved_by_group: &std::collections::HashSet<GraphNodeId>,
    per_group_delta: &std::collections::HashMap<crate::core::GroupId, CanvasPoint>,
    per_node_delta: &std::collections::HashMap<GraphNodeId, CanvasPoint>,
    flags: ModeFlags,
) -> CanvasPoint {
    if !flags.aligns || (!per_group_delta.is_empty() || !per_node_delta.is_empty()) == false {
        return CanvasPoint::default();
    }
    if moved_nodes.len() <= 1 {
        return CanvasPoint::default();
    }
    let Some(extent) = snapshot.interaction.node_extent else {
        return CanvasPoint::default();
    };

    let mut min_x: f32 = f32::INFINITY;
    let mut min_y: f32 = f32::INFINITY;
    let mut max_x: f32 = f32::NEG_INFINITY;
    let mut max_y: f32 = f32::NEG_INFINITY;
    let mut any = false;

    for node_id in moved_nodes.iter().copied() {
        let Some(node_geom) = geom.nodes.get(&node_id) else {
            continue;
        };
        let w = node_geom.rect.size.width.0.max(0.0);
        let h = node_geom.rect.size.height.0.max(0.0);
        if !w.is_finite() || !h.is_finite() {
            continue;
        }

        let base_delta = if moved_by_group.contains(&node_id) {
            g.nodes
                .get(&node_id)
                .and_then(|node| node.parent)
                .and_then(|parent| per_group_delta.get(&parent).copied())
                .unwrap_or_default()
        } else {
            per_node_delta.get(&node_id).copied().unwrap_or_default()
        };

        let x0 = node_geom.rect.origin.x.0 + base_delta.x;
        let y0 = node_geom.rect.origin.y.0 + base_delta.y;
        if !x0.is_finite() || !y0.is_finite() {
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
        return CanvasPoint::default();
    }

    let bbox_w = (max_x - min_x).max(0.0);
    let bbox_h = (max_y - min_y).max(0.0);
    let extent_w = extent.size.width.max(0.0);
    let extent_h = extent.size.height.max(0.0);
    let mut shift = CanvasPoint::default();

    if flags.affects_x && extent.origin.x.is_finite() && extent_w.is_finite() && bbox_w.is_finite()
    {
        let min_dx = extent.origin.x - min_x;
        let mut max_dx = extent.origin.x + (extent_w - bbox_w).max(0.0) - min_x;
        if !max_dx.is_finite() || max_dx < min_dx {
            max_dx = min_dx;
        }
        shift.x = 0.0_f32.clamp(min_dx, max_dx);
    }

    if flags.affects_y && extent.origin.y.is_finite() && extent_h.is_finite() && bbox_h.is_finite()
    {
        let min_dy = extent.origin.y - min_y;
        let mut max_dy = extent.origin.y + (extent_h - bbox_h).max(0.0) - min_y;
        if !max_dy.is_finite() || max_dy < min_dy {
            max_dy = min_dy;
        }
        shift.y = 0.0_f32.clamp(min_dy, max_dy);
    }

    shift
}
