//! Paint grid diagnostics adapter contract.
//!
//! This module keeps grid tile cache diagnostics recording behind a named seam. Snapshot collection
//! stays in grid stats bookkeeping; retained diagnostics bindings live in a separate module.

use fret_canvas::cache::SceneOpTileCacheStats;
use fret_ui::UiHost;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct GridTileCacheStatsSnapshot {
    pub(super) entries: usize,
    pub(super) requested_tiles: usize,
    pub(super) budget_limit: u32,
    pub(super) budget_used: u32,
    pub(super) skipped_tiles: u32,
    pub(super) stats: SceneOpTileCacheStats,
}

pub(super) trait PaintGridDiagnosticsCx<H: UiHost> {
    fn record_grid_tile_cache_stats(&mut self, snapshot: GridTileCacheStatsSnapshot);
}

pub(super) fn record_grid_tile_cache_stats<H>(
    cx: &mut impl PaintGridDiagnosticsCx<H>,
    snapshot: GridTileCacheStatsSnapshot,
) where
    H: UiHost,
{
    cx.record_grid_tile_cache_stats(snapshot);
}
