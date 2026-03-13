use fret_core::Point;
use fret_ui::UiHost;

use crate::core::NodeId as GraphNodeId;
use crate::io::NodeGraphSelectionMode;
use crate::ui::canvas::geometry::CanvasGeometry;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn marquee_selection<H: UiHost, M: NodeGraphCanvasMiddleware>(
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

fn nodes_in_marquee(
    graph: &crate::core::Graph,
    geom: &CanvasGeometry,
    a: Point,
    b: Point,
    mode: NodeGraphSelectionMode,
) -> Vec<GraphNodeId> {
    let rect = crate::ui::canvas::widget::rect_from_points(a, b);
    geom.nodes
        .iter()
        .filter_map(|(id, ng)| {
            let node = graph.nodes.get(id)?;
            if !node.selectable.unwrap_or(true) {
                return None;
            }
            match mode {
                NodeGraphSelectionMode::Full => node_fully_contained(rect, ng.rect).then_some(*id),
                NodeGraphSelectionMode::Partial => {
                    crate::ui::canvas::widget::rects_intersect(rect, ng.rect).then_some(*id)
                }
            }
        })
        .collect()
}

fn node_fully_contained(rect: fret_core::Rect, node_rect: fret_core::Rect) -> bool {
    node_rect.origin.x.0 >= rect.origin.x.0
        && node_rect.origin.y.0 >= rect.origin.y.0
        && (node_rect.origin.x.0 + node_rect.size.width.0) <= (rect.origin.x.0 + rect.size.width.0)
        && (node_rect.origin.y.0 + node_rect.size.height.0)
            <= (rect.origin.y.0 + rect.size.height.0)
}
