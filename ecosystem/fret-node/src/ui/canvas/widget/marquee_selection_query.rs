mod edges;
mod nodes;

use fret_core::Point;
use fret_ui::UiHost;

use crate::core::{EdgeId, NodeId as GraphNodeId};
use crate::ui::canvas::state::ViewSnapshot;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn collect_marquee_selection<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    start_pos: Point,
    position: Point,
) -> (Vec<GraphNodeId>, Vec<EdgeId>) {
    let selected_nodes = nodes::marquee_selection(canvas, host, snapshot, start_pos, position);
    let selected_edges = edges::selected_edges_for_nodes(canvas, host, snapshot, &selected_nodes);
    (selected_nodes, selected_edges)
}
