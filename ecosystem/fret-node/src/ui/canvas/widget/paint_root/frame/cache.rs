use super::*;

pub(super) fn begin_paint_root_caches<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
) {
    canvas.paint_cache.begin_frame();
    canvas.groups_scene_cache.begin_frame();
    canvas.nodes_scene_cache.begin_frame();
    canvas.edges_scene_cache.begin_frame();
    canvas.edge_labels_scene_cache.begin_frame();
}

pub(super) fn record_path_cache_stats<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
) {
    let (entries, stats) = canvas.paint_cache.diagnostics_path_cache_snapshot();
    super::super::frame_diagnostics_adapter::record_paint_root_path_cache_stats(cx, entries, stats);
}
