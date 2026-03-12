use crate::ui::canvas::widget::paint_render_data::EdgeRender;
use crate::ui::canvas::widget::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_edges_cached_budgeted<M: NodeGraphCanvasMiddleware, H: UiHost>(
    canvas: &mut NodeGraphCanvasWith<M>,
    tmp: &mut fret_core::Scene,
    host: &H,
    services: &mut dyn fret_core::UiServices,
    edges: &[EdgeRender],
    zoom: f32,
    scale_factor: f32,
    next_edge: usize,
    wire_budget: &mut WorkBudget,
    marker_budget: &mut WorkBudget,
) -> (usize, bool) {
    let custom_paths = canvas.collect_custom_edge_paths(host, edges, zoom);
    let mut next_edge = next_edge.min(edges.len());

    for edge in edges.iter().skip(next_edge) {
        let width = canvas.style.geometry.wire_width * edge.hint.width_mul.max(0.0);
        let (stop, _marker_skipped) = if let Some(custom) = custom_paths.get(&edge.id) {
            let fallback = Point::new(
                Px(edge.to.x.0 - edge.from.x.0),
                Px(edge.to.y.0 - edge.from.y.0),
            );
            let (t0, t1) =
                path_start_end_tangents(&custom.commands).unwrap_or((fallback, fallback));
            canvas.push_edge_custom_wire_and_markers_budgeted(
                tmp,
                services,
                custom.cache_key,
                &custom.commands,
                t0,
                t1,
                zoom,
                scale_factor,
                edge.from,
                edge.to,
                edge.paint,
                edge.color,
                width,
                edge.hint.dash,
                None,
                edge.hint.start_marker.as_ref(),
                edge.hint.end_marker.as_ref(),
                wire_budget,
                marker_budget,
                true,
            )
        } else {
            canvas.push_edge_wire_and_markers_budgeted(
                tmp,
                services,
                zoom,
                scale_factor,
                edge.hint.route,
                edge.from,
                edge.to,
                edge.paint,
                edge.color,
                width,
                edge.hint.dash,
                None,
                edge.hint.start_marker.as_ref(),
                edge.hint.end_marker.as_ref(),
                wire_budget,
                marker_budget,
                true,
            )
        };

        if stop {
            return (next_edge, true);
        }
        next_edge = next_edge.saturating_add(1);
    }

    (next_edge, false)
}
