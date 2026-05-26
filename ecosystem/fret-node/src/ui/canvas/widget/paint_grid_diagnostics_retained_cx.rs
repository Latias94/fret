use crate::ui::canvas::widget::*;

use super::paint_grid_diagnostics_adapter::GridTileCacheStatsSnapshot;

impl<H: UiHost> super::paint_grid_diagnostics_adapter::PaintGridDiagnosticsCx<H>
    for PaintCx<'_, H>
{
    fn record_grid_tile_cache_stats(&mut self, snapshot: GridTileCacheStatsSnapshot) {
        let Some(window) = self.window else {
            return;
        };
        let frame_id = self.app.frame_id().0;
        let tile_key = CanvasCacheKey {
            window: window.data().as_ffi(),
            node: self.node.data().as_ffi(),
            name: "fret-node.canvas.grid_tiles",
        };
        self.app
            .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
                registry.record_scene_op_tile_cache_with_budget(
                    tile_key,
                    frame_id,
                    snapshot.entries,
                    snapshot.requested_tiles,
                    snapshot.budget_limit,
                    snapshot.budget_used,
                    snapshot.skipped_tiles,
                    snapshot.stats,
                );
            });
    }
}
