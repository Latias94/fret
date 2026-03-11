use std::collections::BTreeSet;

use fret_ui::UiHost;

use crate::core::{EdgeId, NodeId as GraphNodeId};
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn selected_edges_for_nodes<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    selected: &[GraphNodeId],
) -> Vec<EdgeId> {
    if snapshot.interaction.elements_selectable && snapshot.interaction.edges_selectable {
        let nodes: BTreeSet<GraphNodeId> = selected.iter().copied().collect();
        canvas.box_select_edges_for_nodes(host, &snapshot.interaction, &nodes)
    } else {
        Vec::new()
    }
}
