//! Paint-root frame diagnostics adapter contract.
//!
//! This module keeps path-cache diagnostics recording behind a named seam. Snapshot collection
//! stays in frame cache bookkeeping; retained diagnostics bindings live in a separate module.

use fret_canvas::cache::CacheStats;
use fret_ui::UiHost;

pub(super) trait PaintRootFrameDiagnosticsCx<H: UiHost> {
    fn record_paint_root_path_cache_stats(&mut self, entries: usize, stats: CacheStats);
}

pub(super) fn record_paint_root_path_cache_stats<H>(
    cx: &mut impl PaintRootFrameDiagnosticsCx<H>,
    entries: usize,
    stats: CacheStats,
) where
    H: UiHost,
{
    cx.record_paint_root_path_cache_stats(entries, stats);
}
