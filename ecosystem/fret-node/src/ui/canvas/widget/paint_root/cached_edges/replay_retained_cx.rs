use fret_core::Scene;
use fret_ui::{PaintCx, UiHost};

impl<H: UiHost> super::replay_adapter::PaintRootCachedEdgeReplayCx<H> for PaintCx<'_, H> {
    fn paint_root_cached_edge_replay_scene(&mut self) -> &mut Scene {
        &mut *self.scene
    }
}
