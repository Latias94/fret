use crate::ui::canvas::widget::move_ops::*;

#[derive(Clone, Copy)]
pub(super) enum ElementId {
    Node(GraphNodeId),
    Group(crate::core::GroupId),
}

#[derive(Clone, Copy)]
pub(super) struct Elem {
    pub(super) id: ElementId,
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) w: f32,
    pub(super) h: f32,
}

pub(super) struct TargetBounds {
    pub(super) left: f32,
    pub(super) top: f32,
    pub(super) right: f32,
    pub(super) bottom: f32,
    pub(super) center_x: f32,
    pub(super) center_y: f32,
}

pub(super) struct DeltaPlan {
    pub(super) per_group_delta: std::collections::HashMap<crate::core::GroupId, CanvasPoint>,
    pub(super) per_node_delta: std::collections::HashMap<GraphNodeId, CanvasPoint>,
}

#[derive(Clone, Copy)]
pub(super) struct ModeFlags {
    pub(super) aligns: bool,
    pub(super) affects_x: bool,
    pub(super) affects_y: bool,
}

impl ModeFlags {
    pub(super) fn for_mode(mode: AlignDistributeMode) -> Self {
        Self {
            aligns: matches!(
                mode,
                AlignDistributeMode::AlignLeft
                    | AlignDistributeMode::AlignRight
                    | AlignDistributeMode::AlignTop
                    | AlignDistributeMode::AlignBottom
                    | AlignDistributeMode::AlignCenterX
                    | AlignDistributeMode::AlignCenterY
            ),
            affects_x: matches!(
                mode,
                AlignDistributeMode::AlignLeft
                    | AlignDistributeMode::AlignRight
                    | AlignDistributeMode::AlignCenterX
            ),
            affects_y: matches!(
                mode,
                AlignDistributeMode::AlignTop
                    | AlignDistributeMode::AlignBottom
                    | AlignDistributeMode::AlignCenterY
            ),
        }
    }
}

pub(super) fn collect_moved_by_group(
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

pub(super) fn collect_elements(
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

pub(super) fn compute_target_bounds(elems: &[Elem]) -> Option<TargetBounds> {
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

pub(super) fn collect_moved_nodes(
    selected_nodes: &[GraphNodeId],
    moved_by_group: &std::collections::HashSet<GraphNodeId>,
) -> std::collections::HashSet<GraphNodeId> {
    selected_nodes
        .iter()
        .copied()
        .chain(moved_by_group.iter().copied())
        .collect()
}

pub(super) fn plan_deltas(
    elems: &[Elem],
    targets: &TargetBounds,
    mode: AlignDistributeMode,
) -> Option<DeltaPlan> {
    let mut per_group_delta = std::collections::HashMap::new();
    let mut per_node_delta = std::collections::HashMap::new();

    match mode {
        AlignDistributeMode::AlignLeft => {
            for elem in elems {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: targets.left - elem.x,
                        y: 0.0,
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::AlignRight => {
            for elem in elems {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: (targets.right - elem.w) - elem.x,
                        y: 0.0,
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::AlignTop => {
            for elem in elems {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: 0.0,
                        y: targets.top - elem.y,
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::AlignBottom => {
            for elem in elems {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: 0.0,
                        y: (targets.bottom - elem.h) - elem.y,
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::AlignCenterX => {
            for elem in elems {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: targets.center_x - (elem.x + 0.5 * elem.w),
                        y: 0.0,
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::AlignCenterY => {
            for elem in elems {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: 0.0,
                        y: targets.center_y - (elem.y + 0.5 * elem.h),
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::DistributeX => {
            if elems.len() < 3 {
                return None;
            }
            let mut sorted = elems.to_vec();
            sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
            let first = sorted.first().copied().unwrap();
            let last = sorted.last().copied().unwrap();
            let c0 = first.x + 0.5 * first.w;
            let c1 = last.x + 0.5 * last.w;
            let span = c1 - c0;
            if !span.is_finite() || span.abs() <= 1.0e-6 {
                return None;
            }
            let step = span / (sorted.len() as f32 - 1.0);
            for (index, elem) in sorted.iter().enumerate().skip(1).take(sorted.len() - 2) {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: (c0 + (index as f32) * step) - (elem.x + 0.5 * elem.w),
                        y: 0.0,
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::DistributeY => {
            if elems.len() < 3 {
                return None;
            }
            let mut sorted = elems.to_vec();
            sorted.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
            let first = sorted.first().copied().unwrap();
            let last = sorted.last().copied().unwrap();
            let c0 = first.y + 0.5 * first.h;
            let c1 = last.y + 0.5 * last.h;
            let span = c1 - c0;
            if !span.is_finite() || span.abs() <= 1.0e-6 {
                return None;
            }
            let step = span / (sorted.len() as f32 - 1.0);
            for (index, elem) in sorted.iter().enumerate().skip(1).take(sorted.len() - 2) {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: 0.0,
                        y: (c0 + (index as f32) * step) - (elem.y + 0.5 * elem.h),
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
    }

    Some(DeltaPlan {
        per_group_delta,
        per_node_delta,
    })
}

pub(super) fn plan_extent_shift(
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

fn assign_delta(
    element_id: ElementId,
    delta: CanvasPoint,
    per_group_delta: &mut std::collections::HashMap<crate::core::GroupId, CanvasPoint>,
    per_node_delta: &mut std::collections::HashMap<GraphNodeId, CanvasPoint>,
) {
    if delta.x.abs() <= 1.0e-9 && delta.y.abs() <= 1.0e-9 {
        return;
    }
    match element_id {
        ElementId::Group(id) => {
            per_group_delta.insert(id, delta);
        }
        ElementId::Node(id) => {
            per_node_delta.insert(id, delta);
        }
    }
}

pub(super) fn append_group_ops(
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

pub(super) fn append_node_ops(
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
