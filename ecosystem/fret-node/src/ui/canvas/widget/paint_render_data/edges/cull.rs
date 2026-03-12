use super::super::*;

pub(super) fn edge_cull_pad<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    edge_id: EdgeId,
    hint: &EdgeRenderHint,
    zoom: f32,
) -> f32 {
    let interaction_width_px = canvas
        .geometry_overrides
        .as_ref()
        .and_then(|o| o.edge_geometry_override(edge_id).interaction_width_px)
        .unwrap_or(snapshot.interaction.edge_interaction_width);
    (interaction_width_px
        .max(
            canvas.style.geometry.wire_width
                * hint.width_mul
                * canvas.style.paint.wire_width_selected_mul,
        )
        .max(
            canvas.style.geometry.wire_width
                * hint.width_mul
                * canvas.style.paint.wire_width_hover_mul,
        ))
        / zoom
}

#[allow(clippy::too_many_arguments)]
pub(super) fn edge_intersects_cull<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    graph: &Graph,
    edge_id: EdgeId,
    hint: &EdgeRenderHint,
    from: Point,
    to: Point,
    cull: Rect,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool {
    let pad = edge_cull_pad(canvas, snapshot, edge_id, hint, zoom);
    let bounds = if let Some(custom) = canvas.edge_custom_path(graph, edge_id, hint, from, to, zoom)
    {
        path_bounds_rect(&custom.commands)
            .map(|rect| inflate_rect(rect, pad))
            .unwrap_or_else(|| edge_bounds_rect(hint.route, from, to, zoom, pad))
    } else {
        edge_bounds_rect(hint.route, from, to, zoom, pad)
    };
    rects_intersect(bounds, cull)
}
