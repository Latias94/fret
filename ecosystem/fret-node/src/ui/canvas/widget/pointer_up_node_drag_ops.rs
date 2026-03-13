mod build;
mod commit_label;
mod group_rect;

use std::collections::{BTreeMap, HashMap};

use fret_core::AppWindowId;
use fret_ui::UiHost;

use crate::core::{GroupId, NodeId as GraphNodeId};
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
    let ops = build::build_release_ops(
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

    let label = commit_label::commit_label(&ops);
    if canvas.commit_ops(host, window, Some(label), ops) {
        NodeDragEndOutcome::Committed
    } else {
        NodeDragEndOutcome::Rejected
    }
}
