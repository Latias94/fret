use crate::ui::canvas::widget::paint_render_data::RenderData;

use super::*;

pub(super) fn paint_root_edges_uncached<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    render_cull_rect: Option<Rect>,
    hovered_edge: Option<EdgeId>,
    zoom: f32,
    view_interacting: bool,
) {
    canvas.edges_build_states.clear();
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
