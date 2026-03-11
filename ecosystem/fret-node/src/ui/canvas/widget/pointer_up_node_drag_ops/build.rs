use std::collections::{BTreeMap, HashMap};

use fret_ui::UiHost;

use crate::core::{GroupId, NodeId as GraphNodeId};
use crate::ops::GraphOp;
use crate::ui::canvas::state::NodeDrag;

use super::group_rect;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn build_release_ops<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    drag: &NodeDrag,
    end_positions: &HashMap<GraphNodeId, crate::core::CanvasPoint>,
    group_overrides: &BTreeMap<GroupId, crate::core::CanvasRect>,
    parent_changes: &[(GraphNodeId, Option<GroupId>, Option<GroupId>)],
) -> Vec<GraphOp> {
    let mut ops = node_position_ops(drag, end_positions);
    push_parent_change_ops(&mut ops, parent_changes);
    ops.extend(group_rect::group_rect_ops(canvas, host, group_overrides));
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
