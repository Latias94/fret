use fret_core::{AppWindowId, Rect, Scene, UiServices};

use crate::UiHost;
use crate::tree::UiTree;

pub struct UiFrameCx<'a, H: UiHost> {
    pub ui: &'a mut UiTree<H>,
    pub app: &'a mut H,
    pub services: &'a mut dyn UiServices,
    pub window: AppWindowId,
    pub bounds: Rect,
    pub scale_factor: f32,
}

pub type UiFrameContext<'a, H> = UiFrameCx<'a, H>;

impl<'a, H: UiHost> UiFrameCx<'a, H> {
    pub fn new(
        ui: &'a mut UiTree<H>,
        app: &'a mut H,
        services: &'a mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        scale_factor: f32,
    ) -> Self {
        Self {
            ui,
            app,
            services,
            window,
            bounds,
            scale_factor,
        }
    }

    pub fn layout_all(&mut self) {
        let span = tracing::trace_span!(
            "fret_ui.layout_all",
            window = ?self.window,
            frame_id = self.app.frame_id().0,
        );
        let _guard = span.enter();
        self.ui
            .layout_all(self.app, self.services, self.bounds, self.scale_factor);
    }

    pub fn paint_all(&mut self, scene: &mut Scene) {
        let span = tracing::trace_span!(
            "fret_ui.paint_all",
            window = ?self.window,
            frame_id = self.app.frame_id().0,
        );
        let _guard = span.enter();
        self.ui.paint_all(
            self.app,
            self.services,
            self.bounds,
            scene,
            self.scale_factor,
        );
    }
}
