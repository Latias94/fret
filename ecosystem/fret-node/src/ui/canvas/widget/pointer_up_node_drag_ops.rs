use std::collections::{BTreeMap, HashMap};

use fret_core::AppWindowId;
use fret_ui::UiHost;

use crate::core::{GroupId, NodeId as GraphNodeId};
use crate::ops::GraphOp;
use crate::runtime::callbacks::NodeDragEndOutcome;
use crate::ui::canvas::state::NodeDrag;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn end_positions(drag: &NodeDrag) -> HashMap<GraphNodeId, crate::core::CanvasPoint> {
    drag.current_nodes.iter().copied().collect()
}

pub(super) fn group_overrides(drag: &NodeDrag) -> BTreeMap<GroupId, crate::core::CanvasRect> {
    drag.current_groups.iter().copied().collect()
}

pub(super) fn commit_release_ops<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    drag: &NodeDrag,
    end_positions: &HashMap<GraphNodeId, crate::core::CanvasPoint>,
    group_overrides: &BTreeMap<GroupId, crate::core::CanvasRect>,
    parent_changes: &[(GraphNodeId, Option<GroupId>, Option<GroupId>)],
) -> NodeDragEndOutcome {
    let ops = build_release_ops(
        canvas,
        host,
        drag,
        end_positions,
        group_overrides,
        parent_changes,
    );
    if ops.is_empty() {
        return NodeDragEndOutcome::NoOp;
    }

    let label = commit_label(&ops);
    if canvas.commit_ops(host, window, Some(label), ops) {
        NodeDragEndOutcome::Committed
    } else {
        NodeDragEndOutcome::Rejected
    }
}

fn build_release_ops<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    drag: &NodeDrag,
    end_positions: &HashMap<GraphNodeId, crate::core::CanvasPoint>,
    group_overrides: &BTreeMap<GroupId, crate::core::CanvasRect>,
    parent_changes: &[(GraphNodeId, Option<GroupId>, Option<GroupId>)],
) -> Vec<GraphOp> {
    let mut ops = node_position_ops(drag, end_positions);
    push_parent_change_ops(&mut ops, parent_changes);
    ops.extend(group_rect_ops(canvas, host, group_overrides));
    ops
}

fn node_position_ops(
    drag: &NodeDrag,
    end_positions: &HashMap<GraphNodeId, crate::core::CanvasPoint>,
) -> Vec<GraphOp> {
    let mut ops: Vec<GraphOp> = Vec::new();
    for (id, start) in &drag.nodes {
        let Some(end) = end_positions.get(id).copied() else {
            continue;
        };
        if end != *start {
            ops.push(GraphOp::SetNodePos {
                id: *id,
                from: *start,
                to: end,
            });
        }
    }
    ops
}

fn push_parent_change_ops(
    ops: &mut Vec<GraphOp>,
    parent_changes: &[(GraphNodeId, Option<GroupId>, Option<GroupId>)],
) {
    for (node_id, from, to) in parent_changes {
        ops.push(GraphOp::SetNodeParent {
            id: *node_id,
            from: *from,
            to: *to,
        });
    }
}

fn group_rect_ops<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    group_overrides: &BTreeMap<GroupId, crate::core::CanvasRect>,
) -> Vec<GraphOp> {
    canvas
        .graph
        .read_ref(host, |graph| {
            group_overrides
                .iter()
                .filter_map(|(&id, &to)| {
                    let from = graph.groups.get(&id).map(|g| g.rect)?;
                    (from != to).then_some(GraphOp::SetGroupRect { id, from, to })
                })
                .collect()
        })
        .ok()
        .unwrap_or_default()
}

fn commit_label(ops: &[GraphOp]) -> &'static str {
    if ops
        .iter()
        .all(|op| matches!(op, GraphOp::SetNodeParent { .. }))
    {
        if ops.len() == 1 {
            "Set Node Parent"
        } else {
            "Set Node Parents"
        }
    } else if ops.len() == 1 {
        "Move Node"
    } else {
        "Move Nodes"
    }
}
