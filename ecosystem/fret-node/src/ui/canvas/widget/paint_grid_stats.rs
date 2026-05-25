use super::*;

pub(super) fn record_grid_tile_cache_stats<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::paint_grid_diagnostics_adapter::PaintGridDiagnosticsCx<H>,
    warmup: &super::paint_grid_cache::GridTileWarmupStats,
) {
    let snapshot = super::paint_grid_diagnostics_adapter::GridTileCacheStatsSnapshot {
        entries: canvas.grid_scene_cache.entries_len(),
        requested_tiles: canvas.grid_tiles_scratch.len(),
        budget_limit: warmup.tile_budget_limit,
        budget_used: warmup.tile_budget_used,
        skipped_tiles: warmup.skipped_tiles,
        stats: canvas.grid_scene_cache.stats(),
    };
    super::paint_grid_diagnostics_adapter::record_grid_tile_cache_stats(cx, snapshot);
}
