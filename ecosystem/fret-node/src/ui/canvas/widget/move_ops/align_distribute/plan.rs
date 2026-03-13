use crate::ui::canvas::widget::move_ops::*;

use super::support::{
    ModeFlags, append_group_ops, append_node_ops, collect_elements, collect_moved_by_group,
    collect_moved_nodes, compute_target_bounds, plan_deltas, plan_extent_shift,
};

pub(super) fn plan_ops(
    g: &Graph,
    geom: &CanvasGeometry,
    selected_nodes: &Vec<GraphNodeId>,
    selected_groups: &Vec<crate::core::GroupId>,
    snapshot: &ViewSnapshot,
    mode: AlignDistributeMode,
) -> Vec<GraphOp> {
    let selected_groups_set: std::collections::HashSet<crate::core::GroupId> =
        selected_groups.iter().copied().collect();
    let moved_by_group = collect_moved_by_group(g, &selected_groups_set);
    let elems = collect_elements(g, geom, selected_nodes, selected_groups);
    let Some(targets) = compute_target_bounds(&elems) else {
        return Vec::new();
    };
    let Some(delta_plan) = plan_deltas(&elems, &targets, mode) else {
        return Vec::new();
    };

    let flags = ModeFlags::for_mode(mode);
    let moved_nodes = collect_moved_nodes(selected_nodes, &moved_by_group);
    let multi_move = moved_nodes.len() > 1;
    let skip_node_extent_clamp = flags.aligns && multi_move;
    let shift = plan_extent_shift(
        g,
        geom,
        snapshot,
        &moved_nodes,
        &moved_by_group,
        &delta_plan.per_group_delta,
        &delta_plan.per_node_delta,
        flags,
    );

    let mut ops = Vec::new();
    append_group_ops(
        &mut ops,
        g,
        geom,
        selected_groups,
        snapshot,
        &delta_plan.per_group_delta,
        shift,
        flags.aligns,
    );
    append_node_ops(
        &mut ops,
        g,
        geom,
        selected_nodes,
        &moved_by_group,
        snapshot,
        &delta_plan.per_node_delta,
        shift,
        skip_node_extent_clamp,
    );
    ops
}
