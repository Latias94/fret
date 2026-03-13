mod collect;
mod rect;
mod target;

use std::collections::{BTreeMap, HashMap};

use fret_ui::UiHost;

use crate::core::{GroupId, NodeId as GraphNodeId};
use crate::ui::canvas::state::{NodeDrag, ViewSnapshot};

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn parent_changes<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    drag: &NodeDrag,
    end_positions: &HashMap<GraphNodeId, crate::core::CanvasPoint>,
    group_overrides: &BTreeMap<GroupId, crate::core::CanvasRect>,
) -> Vec<(GraphNodeId, Option<GroupId>, Option<GroupId>)> {
    collect::parent_changes(canvas, host, snapshot, drag, end_positions, group_overrides)
}
