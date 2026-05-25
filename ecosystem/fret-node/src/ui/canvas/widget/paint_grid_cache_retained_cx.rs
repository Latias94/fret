use fret_core::Scene;
use fret_ui::{PaintCx, UiHost};

impl<H: UiHost> super::paint_grid_cache_adapter::PaintGridTileCacheCx<H> for PaintCx<'_, H> {
    fn paint_grid_scene(&mut self) -> &mut Scene {
        &mut *self.scene
    }
}
