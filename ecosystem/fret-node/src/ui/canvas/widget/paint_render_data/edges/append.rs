use std::collections::HashSet;
use std::collections::hash_map::Entry;

use super::super::*;
use fret_core::scene::PaintBindingV1;

#[allow(clippy::too_many_arguments)]
pub(super) fn append_edge_render_data<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    graph: &Graph,
    snapshot: &ViewSnapshot,
    geom: &CanvasGeometry,
    presenter: &dyn NodeGraphPresenter,
    selected_edges: &HashSet<EdgeId>,
    hovered_edge: Option<EdgeId>,
    cull: Option<Rect>,
    zoom: f32,
    out: &mut RenderData,
    edge_id: EdgeId,
) {
    let Some(edge) = graph.edges.get(&edge_id) else {
        return;
    };
    if canvas
        .interaction
        .wire_drag
        .as_ref()
        .is_some_and(|wire_drag| {
            NodeGraphCanvasWith::<M>::wire_drag_suppresses_edge(&wire_drag.kind, edge_id)
        })
    {
        return;
    }

    let from = match out.port_centers.entry(edge.from) {
        Entry::Occupied(entry) => *entry.get(),
        Entry::Vacant(entry) => {
            let Some(center) = geom.port_center(edge.from) else {
                return;
            };
            *entry.insert(center)
        }
    };
    let to = match out.port_centers.entry(edge.to) {
        Entry::Occupied(entry) => *entry.get(),
        Entry::Vacant(entry) => {
            let Some(center) = geom.port_center(edge.to) else {
                return;
            };
            *entry.insert(center)
        }
    };

    let selected = selected_edges.contains(&edge_id);
    let hovered = hovered_edge == Some(edge_id);
    let hint = super::hint::resolve_edge_render_hint(canvas, graph, edge_id, selected, hovered);
    let (hint, paint_override) = super::hint::apply_edge_paint_override(canvas, edge_id, hint);

    if cull.is_some_and(|rect| {
        !super::cull::edge_intersects_cull(
            canvas, graph, edge_id, &hint, from, to, rect, snapshot, zoom,
        )
    }) {
        return;
    }

    let mut color = presenter.edge_color(graph, edge_id, &canvas.style);
    if let Some(override_color) = hint.color {
        color = override_color;
    }

    let mut paint: PaintBindingV1 = color.into();
    if let Some(override_paint) =
        paint_override.and_then(|override_state| override_state.stroke_paint)
    {
        paint = override_paint;
    }

    out.edges.push(EdgeRender {
        id: edge_id,
        rank: super::candidate::edge_rank_for_render(geom, edge),
        from,
        to,
        color,
        paint,
        hint,
        selected,
        hovered,
    });
    out.metrics.edge_visible = out.metrics.edge_visible.saturating_add(1);
}
