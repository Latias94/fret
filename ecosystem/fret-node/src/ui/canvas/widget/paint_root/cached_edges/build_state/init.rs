use super::*;

pub(super) fn collect_cached_edge_renders<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &H,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    cull_rect: Rect,
    zoom: f32,
) -> Vec<paint_render_data::EdgeRender> {
    canvas
        .collect_render_data(
            host,
            snapshot,
            Arc::clone(geom),
            Arc::clone(index),
            Some(cull_rect),
            zoom,
            None,
            false,
            false,
            true,
        )
        .edges
}

pub(super) fn init_edges_build_state<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &H,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    clip_rect: Rect,
    cull_rect: Rect,
    zoom: f32,
) -> EdgesBuildState {
    let (ops, edges) = init_cached_edge_build_parts(
        canvas, host, snapshot, geom, index, clip_rect, cull_rect, zoom,
    );
    EdgesBuildState {
        ops,
        edges,
        next_edge: 0,
    }
}

pub(super) fn init_edge_labels_build_state<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &H,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    key: u64,
    clip_rect: Rect,
    cull_rect: Rect,
    zoom: f32,
) -> EdgeLabelsBuildState {
    let (ops, edges) = init_cached_edge_build_parts(
        canvas, host, snapshot, geom, index, clip_rect, cull_rect, zoom,
    );
    EdgeLabelsBuildState {
        key,
        ops,
        edges,
        next_edge: 0,
    }
}

pub(super) fn init_cached_edge_build_parts<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &H,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    clip_rect: Rect,
    cull_rect: Rect,
    zoom: f32,
) -> (Vec<SceneOp>, Vec<paint_render_data::EdgeRender>) {
    (
        super::ops::initial_clip_ops(clip_rect),
        collect_cached_edge_renders(canvas, host, snapshot, geom, index, cull_rect, zoom),
    )
}
