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
    let Some(window) = cx.window else {
        return;
    };
    let (entries, stats) = canvas.paint_cache.diagnostics_path_cache_snapshot();
    let frame_id = cx.app.frame_id().0;
    let key = CanvasCacheKey {
        window: window.data().as_ffi(),
        node: cx.node.data().as_ffi(),
        name: "fret-node.canvas.paths",
    };
    cx.app
        .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
            registry.record_path_cache(key, frame_id, entries, stats);
        });
}
