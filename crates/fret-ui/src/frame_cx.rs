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
        self.ui
            .layout_all(self.app, self.services, self.bounds, self.scale_factor);
    }

    pub fn paint_all(&mut self, scene: &mut Scene) {
        self.ui.paint_all(
            self.app,
            self.services,
            self.bounds,
            scene,
            self.scale_factor,
        );
    }
}
