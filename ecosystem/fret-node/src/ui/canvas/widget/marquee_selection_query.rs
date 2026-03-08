use std::collections::BTreeSet;

use fret_core::Point;
use fret_ui::UiHost;

use crate::core::{EdgeId, NodeId as GraphNodeId};
use crate::io::NodeGraphSelectionMode;
use crate::ui::canvas::geometry::CanvasGeometry;
use crate::ui::canvas::state::ViewSnapshot;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn collect_marquee_selection<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    start_pos: Point,
    position: Point,
) -> (Vec<GraphNodeId>, Vec<EdgeId>) {
    let selected_nodes = marquee_selection(canvas, host, snapshot, start_pos, position);
    let selected_edges = selected_edges_for_nodes(canvas, host, snapshot, &selected_nodes);
    (selected_nodes, selected_edges)
}

fn marquee_selection<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    start_pos: Point,
    position: Point,
) -> Vec<GraphNodeId> {
    let (geom, _index) = canvas.canvas_derived(&*host, snapshot);
    let selection = canvas
        .graph
        .read_ref(host, |graph| {
            nodes_in_marquee(
                graph,
                geom.as_ref(),
                start_pos,
                position,
                snapshot.interaction.selection_mode,
            )
        })
        .ok()
        .unwrap_or_default();

    let mut selected = selection;
    selected.sort();
    selected.dedup();
    selected
}

fn selected_edges_for_nodes<H: UiHost, M: NodeGraphCanvasMiddleware>(
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

fn nodes_in_marquee(
    graph: &crate::core::Graph,
    geom: &CanvasGeometry,
    a: Point,
    b: Point,
    mode: NodeGraphSelectionMode,
) -> Vec<GraphNodeId> {
    let rect = super::rect_from_points(a, b);
    geom.nodes
        .iter()
        .filter_map(|(id, ng)| {
            let node = graph.nodes.get(id)?;
            if !node.selectable.unwrap_or(true) {
                return None;
            }
            match mode {
                NodeGraphSelectionMode::Full => {
                    let fully_contained = ng.rect.origin.x.0 >= rect.origin.x.0
                        && ng.rect.origin.y.0 >= rect.origin.y.0
                        && (ng.rect.origin.x.0 + ng.rect.size.width.0)
                            <= (rect.origin.x.0 + rect.size.width.0)
                        && (ng.rect.origin.y.0 + ng.rect.size.height.0)
                            <= (rect.origin.y.0 + rect.size.height.0);
                    fully_contained.then_some(*id)
                }
                NodeGraphSelectionMode::Partial => {
                    super::rects_intersect(rect, ng.rect).then_some(*id)
                }
            }
        })
        .collect()
}
