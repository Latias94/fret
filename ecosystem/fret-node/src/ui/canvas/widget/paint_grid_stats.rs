use super::*;

pub(super) fn record_grid_tile_cache_stats<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    warmup: &super::paint_grid_cache::GridTileWarmupStats,
) {
    let Some(window) = cx.window else {
        return;
    };
    let frame_id = cx.app.frame_id().0;
    let tile_entries = canvas.grid_scene_cache.entries_len();
    let tile_stats = canvas.grid_scene_cache.stats();
    let requested_tiles = canvas.grid_tiles_scratch.len();
    let tile_key = CanvasCacheKey {
        window: window.data().as_ffi(),
        node: cx.node.data().as_ffi(),
        name: "fret-node.canvas.grid_tiles",
    };
    cx.app
        .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
            registry.record_scene_op_tile_cache_with_budget(
                tile_key,
                frame_id,
                tile_entries,
                requested_tiles,
                warmup.tile_budget_limit,
                warmup.tile_budget_used,
                warmup.skipped_tiles,
                tile_stats,
            );
        });
}
