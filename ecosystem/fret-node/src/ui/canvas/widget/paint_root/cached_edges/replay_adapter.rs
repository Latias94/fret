//! Paint-root cached edge replay adapter contract.
//!
//! This module keeps cached edge and edge-label replay scene access behind a named seam. Concrete
//! retained scene sinks live next to the cached edge replay binding.

use fret_core::Scene;
use fret_ui::UiHost;

pub(super) trait PaintRootCachedEdgeReplayCx<H: UiHost> {
    fn paint_root_cached_edge_replay_scene(&mut self) -> &mut Scene;
}

pub(super) fn paint_root_cached_edge_replay_scene<H>(
    cx: &mut impl PaintRootCachedEdgeReplayCx<H>,
) -> &mut Scene
where
    H: UiHost,
{
    cx.paint_root_cached_edge_replay_scene()
}
