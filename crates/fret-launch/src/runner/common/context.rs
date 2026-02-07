use fret_app::App;
use fret_core::{Rect, Scene, UiServices};

pub struct WinitWindowContext<'a, S> {
    pub app: &'a mut App,
    pub window: fret_core::AppWindowId,
    pub state: &'a mut S,
}

pub struct WinitEventContext<'a, S> {
    pub app: &'a mut App,
    pub services: &'a mut dyn UiServices,
    pub window: fret_core::AppWindowId,
    pub state: &'a mut S,
}

pub struct WinitCommandContext<'a, S> {
    pub app: &'a mut App,
    pub services: &'a mut dyn UiServices,
    pub window: fret_core::AppWindowId,
    pub state: &'a mut S,
}

pub struct WinitRenderContext<'a, S> {
    pub app: &'a mut App,
    pub services: &'a mut dyn UiServices,
    pub window: fret_core::AppWindowId,
    pub state: &'a mut S,
    pub bounds: Rect,
    pub scale_factor: f32,
    pub scene: &'a mut Scene,
}

pub struct WinitGlobalContext<'a> {
    pub app: &'a mut App,
    pub services: &'a mut dyn UiServices,
}

pub struct WinitHotReloadContext<'a, S> {
    pub app: &'a mut App,
    pub services: &'a mut dyn UiServices,
    pub window: fret_core::AppWindowId,
    pub state: &'a mut S,
}
