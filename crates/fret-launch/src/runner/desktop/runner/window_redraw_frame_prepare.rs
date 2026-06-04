use fret_app::App;
use fret_core::time::Duration;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_render::{Renderer, WgpuContext};
use winit::window::Window;

use super::WinitAppDriver;
use super::redraw_hitch::{RedrawPhase, measure_redraw_phase, quantize_logical_px};

pub(super) struct WindowRedrawFramePrepareInput<'a, D: WinitAppDriver> {
    pub(super) app: &'a mut App,
    pub(super) driver: &'a mut D,
    pub(super) app_window: AppWindowId,
    pub(super) user: &'a mut D::WindowState,
    pub(super) platform: &'a mut fret_runner_winit::WinitPlatform,
    pub(super) window: &'a dyn Window,
    pub(super) context: &'a WgpuContext,
    pub(super) renderer: &'a mut Renderer,
    pub(super) hitch_enabled: bool,
}

pub(super) struct WindowRedrawFramePrepare {
    pub(super) scale_factor: f32,
    pub(super) bounds: Rect,
}

pub(super) fn prepare_window_redraw_frame<D: WinitAppDriver>(
    input: WindowRedrawFramePrepareInput<'_, D>,
) -> (WindowRedrawFramePrepare, Option<Duration>) {
    measure_redraw_phase(RedrawPhase::Prepare, input.hitch_enabled, || {
        // Apply any pending window-side state (IME/cursor) once per frame,
        // similar to Dear ImGui's backend `prepare_frame` pattern.
        input.platform.prepare_frame(input.window);

        let scale_factor = input.window.scale_factor() as f32;
        let bounds = window_redraw_frame_bounds(input.window);

        input.driver.gpu_frame_prepare(
            input.app,
            input.app_window,
            input.user,
            input.context,
            input.renderer,
            scale_factor,
        );

        WindowRedrawFramePrepare {
            scale_factor,
            bounds,
        }
    })
}

fn window_redraw_frame_bounds(window: &dyn Window) -> Rect {
    let physical = window.surface_size();
    let logical: winit::dpi::LogicalSize<f32> = physical.to_logical(window.scale_factor());
    let logical_width = quantize_logical_px(logical.width);
    let logical_height = quantize_logical_px(logical.height);

    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(logical_width), Px(logical_height)),
    )
}
