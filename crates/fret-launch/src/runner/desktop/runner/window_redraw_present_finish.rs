use fret_app::App;
use fret_core::AppWindowId;
use fret_runtime::FrameId;

use super::EngineFrameKeepalive;
use super::diag_bundle_screenshots::DiagBundleScreenshotCapture;
#[cfg(feature = "diag-screenshots")]
use super::diag_screenshots::{DiagScreenshotCapture, InFlightCapture};
use super::window_redraw_diag_screenshots::WindowRedrawBundleScreenshotReadback;

pub(super) struct WindowRedrawPresentFinishInput<'a> {
    pub(super) app: &'a mut App,
    pub(super) frame_id: &'a mut FrameId,
    pub(super) app_window: AppWindowId,
    pub(super) keepalive: Vec<EngineFrameKeepalive>,
    #[cfg(feature = "diag-screenshots")]
    pub(super) diag_screenshots: Option<&'a mut DiagScreenshotCapture>,
    pub(super) bundle_screenshots: &'a DiagBundleScreenshotCapture,
    pub(super) device: &'a wgpu::Device,
    #[cfg(feature = "diag-screenshots")]
    pub(super) screenshot_inflight: Option<InFlightCapture>,
    pub(super) bundle_screenshot_readback: Option<WindowRedrawBundleScreenshotReadback>,
    pub(super) surface_format: wgpu::TextureFormat,
}

pub(super) fn finish_window_redraw_present_frame(input: WindowRedrawPresentFinishInput<'_>) {
    super::scheduling_diagnostics::commit_presented_frame_for_window(
        input.app,
        input.frame_id,
        input.app_window,
    );
    drop(input.keepalive);

    #[cfg(feature = "diag-screenshots")]
    super::window_redraw_diag_screenshots::finish_window_redraw_diag_screenshot_capture(
        input.diag_screenshots,
        input.device,
        input.app_window,
        input.screenshot_inflight,
    );

    super::window_redraw_diag_screenshots::finish_window_redraw_bundle_screenshot_readback(
        input.bundle_screenshots,
        input.device,
        input.bundle_screenshot_readback,
        input.surface_format,
    );
}
