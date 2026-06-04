use fret_app::App;
use fret_core::AppWindowId;
use fret_runtime::{FrameId, TickId};

use super::WinitAppDriver;
use super::window_redraw_text_diagnostics::WindowRedrawTextDiagnosticsMode;

pub(super) struct WindowRedrawPostRenderDiagnosticsInput<'a, D: WinitAppDriver> {
    pub(super) app: &'a mut App,
    pub(super) driver: &'a mut D,
    pub(super) renderer: &'a mut fret_render::Renderer,
    pub(super) context: &'a fret_render::WgpuContext,
    pub(super) app_window: AppWindowId,
    pub(super) user: &'a mut D::WindowState,
    pub(super) tick_id: TickId,
    pub(super) frame_id: FrameId,
    pub(super) text_diagnostics: WindowRedrawTextDiagnosticsMode,
}

pub(super) fn publish_window_redraw_post_render_diagnostics<D: WinitAppDriver>(
    input: WindowRedrawPostRenderDiagnosticsInput<'_, D>,
) {
    super::window_redraw_text_diagnostics::publish_window_redraw_text_diagnostics(
        input.app,
        input.renderer,
        input.frame_id,
        input.text_diagnostics,
    );

    super::window_redraw_renderer_perf::maybe_publish_window_redraw_renderer_perf_sample(
        input.app,
        input.driver,
        input.renderer,
        input.app_window,
        input.user,
        input.tick_id.0,
        input.frame_id.0,
    );

    super::window_redraw_wgpu_report::maybe_record_window_redraw_wgpu_hub_report(
        input.app,
        input.context,
        input.app_window,
        input.tick_id.0,
        input.frame_id.0,
    );

    super::window_redraw_wgpu_allocator_report::maybe_record_window_redraw_wgpu_allocator_report(
        input.app,
        input.context,
        input.app_window,
        input.tick_id.0,
        input.frame_id.0,
    );
}
