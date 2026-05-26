use crate::ui::canvas::widget::*;

impl<H: UiHost> super::frame_diagnostics_adapter::PaintRootFrameDiagnosticsCx<H>
    for PaintCx<'_, H>
{
    fn record_paint_root_path_cache_stats(
        &mut self,
        entries: usize,
        stats: fret_canvas::cache::CacheStats,
    ) {
        let Some(window) = self.window else {
            return;
        };
        let frame_id = self.app.frame_id().0;
        let key = CanvasCacheKey {
            window: window.data().as_ffi(),
            node: self.node.data().as_ffi(),
            name: "fret-node.canvas.paths",
        };
        self.app
            .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
                registry.record_path_cache(key, frame_id, entries, stats);
            });
    }
}
