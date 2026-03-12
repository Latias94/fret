use std::collections::{BTreeMap, HashMap};

use fret_ui::UiHost;

use crate::core::{GroupId, NodeId as GraphNodeId};
use crate::ui::canvas::state::{NodeDrag, ViewSnapshot};
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

use super::{rect, target};

pub(super) fn parent_changes<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    drag: &NodeDrag,
    end_positions: &HashMap<GraphNodeId, crate::core::CanvasPoint>,
    group_overrides: &BTreeMap<GroupId, crate::core::CanvasRect>,
) -> Vec<(GraphNodeId, Option<GroupId>, Option<GroupId>)> {
    let geom = canvas.canvas_geometry(&*host, snapshot);
    canvas
        .graph
        .read_ref(host, |graph| {
            let mut changes: Vec<(GraphNodeId, Option<GroupId>, Option<GroupId>)> = Vec::new();

            for (node_id, _start) in &drag.nodes {
                let Some(node) = graph.nodes.get(node_id) else {
                    continue;
                };
                let Some(node_geom) = geom.nodes.get(node_id) else {
                    continue;
                };
                let Some(pos) = end_positions.get(node_id).copied() else {
                    continue;
                };

                let rect = rect::node_rect(pos, node_geom.rect.size);
                let new_parent = target::best_parent_group(rect, graph, group_overrides);
                if node.parent != new_parent {
                    changes.push((*node_id, node.parent, new_parent));
                }
            }

            changes
        })
        .ok()
        .unwrap_or_default()
}
