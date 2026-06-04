use fret_app::App;
use fret_core::time::Duration;
use fret_core::{AppWindowId, Rect, Scene};
use fret_render::Renderer;

use super::redraw_hitch::{RedrawPhase, measure_redraw_phase};
use super::window_redraw_text_diagnostics::{
    WindowRedrawTextDiagnosticsMode, begin_window_redraw_text_diagnostics_frame,
    window_redraw_text_diagnostics_mode_from_env,
};
use super::{WinitAppDriver, WinitRenderContext};

pub(super) struct WindowRedrawRenderInput<'a, D: WinitAppDriver> {
    pub(super) app: &'a mut App,
    pub(super) driver: &'a mut D,
    pub(super) renderer: &'a mut Renderer,
    pub(super) app_window: AppWindowId,
    pub(super) user: &'a mut D::WindowState,
    pub(super) scene: &'a mut Scene,
    pub(super) bounds: Rect,
    pub(super) scale_factor: f32,
    pub(super) hitch_enabled: bool,
}

pub(super) struct WindowRedrawRender {
    pub(super) text_diagnostics: WindowRedrawTextDiagnosticsMode,
}

pub(super) fn render_window_redraw_frame<D: WinitAppDriver>(
    input: WindowRedrawRenderInput<'_, D>,
) -> (WindowRedrawRender, Option<Duration>) {
    let text_diagnostics = window_redraw_text_diagnostics_mode_from_env();
    let (_, elapsed) = measure_redraw_phase(
        RedrawPhase::Render {
            bounds: input.bounds,
            scale_factor: input.scale_factor,
        },
        input.hitch_enabled,
        || {
            begin_window_redraw_text_diagnostics_frame(input.renderer, text_diagnostics);
            input.driver.render(WinitRenderContext {
                app: input.app,
                services: input.renderer as &mut dyn fret_core::UiServices,
                window: input.app_window,
                state: input.user,
                bounds: input.bounds,
                scale_factor: input.scale_factor,
                scene: input.scene,
            });
        },
    );
    (WindowRedrawRender { text_diagnostics }, elapsed)
}
