use fret_core::{Scene, UiServices};
use fret_ui::{PaintCx, UiHost};

impl<H: UiHost> super::cached_static_scene_adapter::PaintRootCachedStaticSceneCx<H>
    for PaintCx<'_, H>
{
    fn paint_root_cached_static_host(&self) -> &H {
        &*self.app
    }

    fn paint_root_cached_static_services(&mut self) -> &mut dyn UiServices {
        self.services
    }

    fn paint_root_cached_static_scale_factor(&self) -> f32 {
        self.scale_factor
    }

    fn paint_root_cached_static_scene(&mut self) -> &mut Scene {
        &mut *self.scene
    }
}
