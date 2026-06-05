use fret_app::App;
use fret_core::time::Duration;
use fret_core::{AppWindowId, Scene};
use fret_render::{ClearColor, RenderError, Renderer, SurfaceState, WgpuContext};
use fret_runtime::{FrameId, TickId};

use super::diag_bundle_screenshots::DiagBundleScreenshotCapture;
#[cfg(feature = "diag-screenshots")]
use super::diag_screenshots::DiagScreenshotCapture;
use super::redraw_hitch::{RedrawPhase, measure_redraw_phase};
use super::window_redraw_text_diagnostics::WindowRedrawTextDiagnosticsMode;
use super::{EngineFrameKeepalive, WinitAppDriver};

pub(super) struct WindowRedrawPresentInput<'a, 'window, D: WinitAppDriver> {
    pub(super) app: &'a mut App,
    pub(super) driver: &'a mut D,
    pub(super) renderer: &'a mut Renderer,
    pub(super) context: &'a WgpuContext,
    pub(super) surface: &'a SurfaceState<'window>,
    pub(super) app_window: AppWindowId,
    pub(super) user: &'a mut D::WindowState,
    pub(super) scene: &'a Scene,
    pub(super) tick_id: TickId,
    pub(super) frame_id: &'a mut FrameId,
    pub(super) scale_factor: f32,
    pub(super) clear_color: ClearColor,
    pub(super) engine_command_buffers: Vec<wgpu::CommandBuffer>,
    pub(super) engine_keepalive: Vec<EngineFrameKeepalive>,
    pub(super) text_diagnostics: WindowRedrawTextDiagnosticsMode,
    #[cfg(feature = "diag-screenshots")]
    pub(super) diag_screenshots: &'a mut Option<DiagScreenshotCapture>,
    pub(super) bundle_screenshots: &'a mut DiagBundleScreenshotCapture,
    pub(super) hitch_enabled: bool,
}

pub(super) fn present_window_redraw_frame<D: WinitAppDriver>(
    input: WindowRedrawPresentInput<'_, '_, D>,
) -> (Result<(), RenderError>, Option<Duration>) {
    let frame_id = *input.frame_id;
    measure_redraw_phase(RedrawPhase::Present, input.hitch_enabled, || {
        let frame_view = super::window_redraw_present_target::acquire_window_redraw_present_frame(
            input.surface,
        )?;

        let clear_color = super::window_redraw_clear_color::resolve_window_redraw_clear_color(
            input.app,
            input.app_window,
            input.clear_color,
        );
        let present_target =
            super::window_redraw_present_target::prepare_window_redraw_present_target(
                super::window_redraw_present_target::WindowRedrawPresentTargetInput {
                    context: input.context,
                    surface: input.surface,
                    frame_view,
                },
            );

        let ui_cmd = super::window_redraw_render_scene::record_window_redraw_render_scene(
            super::window_redraw_render_scene::WindowRedrawRenderSceneInput {
                renderer: input.renderer,
                context: input.context,
                surface: input.surface,
                target_view: present_target.target_view(),
                scene: input.scene,
                clear_color,
                scale_factor: input.scale_factor,
            },
        );
        super::window_redraw_post_render_diagnostics::publish_window_redraw_post_render_diagnostics(
            super::window_redraw_post_render_diagnostics::WindowRedrawPostRenderDiagnosticsInput {
                app: input.app,
                driver: input.driver,
                renderer: input.renderer,
                context: input.context,
                app_window: input.app_window,
                user: input.user,
                tick_id: input.tick_id,
                frame_id,
                text_diagnostics: input.text_diagnostics,
            },
        );

        let capture_commands =
            super::window_redraw_present_capture_commands::prepare_window_redraw_present_capture_commands(
                super::window_redraw_present_capture_commands::WindowRedrawPresentCaptureCommandsInput {
                    command_buffers: input.engine_command_buffers,
                    ui_cmd,
                    #[cfg(feature = "diag-screenshots")]
                    diag_screenshots: input.diag_screenshots.as_mut(),
                    bundle_screenshots: input.bundle_screenshots,
                    #[cfg(feature = "diag-screenshots")]
                    app_window: input.app_window,
                    frame_view: present_target.frame_view(),
                    device: &input.context.device,
                    surface_format: input.surface.format(),
                    surface_size: input.surface.size(),
                },
            );

        super::window_redraw_present_submit::submit_window_redraw_present_frame(
            super::window_redraw_present_submit::WindowRedrawPresentSubmitInput {
                context: input.context,
                command_buffers: capture_commands.command_buffers,
                present_target,
            },
        );
        super::window_redraw_present_finish::finish_window_redraw_present_frame(
            super::window_redraw_present_finish::WindowRedrawPresentFinishInput {
                app: input.app,
                frame_id: input.frame_id,
                app_window: input.app_window,
                keepalive: input.engine_keepalive,
                #[cfg(feature = "diag-screenshots")]
                diag_screenshots: input.diag_screenshots.as_mut(),
                bundle_screenshots: input.bundle_screenshots,
                device: &input.context.device,
                #[cfg(feature = "diag-screenshots")]
                screenshot_inflight: capture_commands.screenshot_inflight,
                bundle_screenshot_readback: capture_commands.bundle_screenshot_readback,
                surface_format: input.surface.format(),
            },
        );

        Ok(())
    })
}
