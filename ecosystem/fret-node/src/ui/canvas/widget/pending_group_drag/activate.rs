use fret_ui::UiHost;

use crate::core::{CanvasPoint, GroupId, NodeId as GraphNodeId};
use crate::ui::canvas::widget::*;

pub(super) fn group_drag_start_nodes<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    group: GroupId,
) -> Vec<(GraphNodeId, CanvasPoint)> {
    canvas
        .graph
        .read_ref(host, |g| {
            g.nodes
                .iter()
                .filter_map(|(id, node)| (node.parent == Some(group)).then_some((*id, node.pos)))
                .collect::<Vec<_>>()
        })
        .ok()
        .unwrap_or_default()
}
