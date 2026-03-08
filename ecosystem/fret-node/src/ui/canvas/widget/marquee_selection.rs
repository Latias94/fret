use std::collections::BTreeSet;

use fret_core::Point;
use fret_ui::UiHost;

use crate::core::NodeId as GraphNodeId;
use crate::io::NodeGraphSelectionMode;
use crate::ui::canvas::state::{MarqueeDrag, ViewSnapshot};

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn update_active_marquee<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
) -> bool {
    let Some(mut marquee) = canvas.interaction.marquee.take() else {
        return false;
    };

    marquee.pos = position;
    let selection = marquee_selection(canvas, cx.app, snapshot, marquee.start_pos, marquee.pos);
    let selected_edges = selected_edges_for_nodes(canvas, cx.app, snapshot, &selection);

    canvas.interaction.marquee = Some(marquee);
    canvas.interaction.focused_edge = None;
    apply_marquee_selection(canvas, cx.app, selection, selected_edges);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

pub(super) fn activate_pending_marquee<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    start_pos: Point,
    position: Point,
) -> bool {
    canvas.interaction.pending_marquee = None;
    let marquee = MarqueeDrag {
        start_pos,
        pos: position,
    };
    canvas.interaction.marquee = Some(marquee.clone());

    let selection = marquee_selection(canvas, cx.app, snapshot, marquee.start_pos, marquee.pos);
    let selected_edges = selected_edges_for_nodes(canvas, cx.app, snapshot, &selection);
    apply_marquee_selection(canvas, cx.app, selection, selected_edges);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
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
) -> Vec<crate::core::EdgeId> {
    if snapshot.interaction.elements_selectable && snapshot.interaction.edges_selectable {
        let nodes: BTreeSet<GraphNodeId> = selected.iter().copied().collect();
        canvas.box_select_edges_for_nodes(host, &snapshot.interaction, &nodes)
    } else {
        Vec::new()
    }
}

fn apply_marquee_selection<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    selected: Vec<GraphNodeId>,
    selected_edges: Vec<crate::core::EdgeId>,
) {
    canvas.update_view_state(host, |state| {
        state.selected_edges.clear();
        state.selected_groups.clear();
        state.selected_nodes = selected;
        state.selected_edges = selected_edges;
    });
}

fn nodes_in_marquee(
    graph: &crate::core::Graph,
    geom: &crate::ui::canvas::geometry::CanvasGeometry,
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
