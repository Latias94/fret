use crate::ui::canvas::widget::paint_render_data::RenderData;

use super::*;

fn clear_edge_cache_build_states<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
) {
    canvas.edges_build_states.clear();
    canvas.edge_labels_build_states.clear();
    canvas.edge_labels_build_state = None;
}

pub(super) fn paint_root_edges_without_static_cache<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    hovered_edge: Option<EdgeId>,
    render_cull_rect: Option<Rect>,
    zoom: f32,
    view_interacting: bool,
) {
    clear_edge_cache_build_states(canvas);
    let render_edges: RenderData = canvas.collect_render_data(
        &*cx.app,
        snapshot,
        Arc::clone(geom),
        Arc::clone(index),
        render_cull_rect,
        zoom,
        hovered_edge,
        false,
        false,
        true,
    );
    canvas.paint_edges(cx, snapshot, &render_edges, geom, zoom, view_interacting);
}
