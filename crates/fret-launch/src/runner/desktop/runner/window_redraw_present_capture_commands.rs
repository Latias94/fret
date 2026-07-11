use super::diag_bundle_screenshots::DiagBundleScreenshotCapture;
#[cfg(feature = "diag-screenshots")]
use super::diag_screenshots::{DiagScreenshotCapture, InFlightCapture};
use super::window_redraw_diag_screenshots::WindowRedrawBundleScreenshotReadback;
#[cfg(feature = "diag-screenshots")]
use fret_core::AppWindowId;

pub(super) struct WindowRedrawPresentCaptureCommandsInput<'a> {
    pub(super) command_buffers: Vec<wgpu::CommandBuffer>,
    pub(super) ui_cmd: wgpu::CommandBuffer,
    #[cfg(feature = "diag-screenshots")]
    pub(super) diag_screenshots: Option<&'a mut DiagScreenshotCapture>,
    pub(super) bundle_screenshots: &'a mut DiagBundleScreenshotCapture,
    #[cfg(feature = "diag-screenshots")]
    pub(super) app_window: AppWindowId,
    pub(super) source_texture: &'a wgpu::Texture,
    pub(super) device: &'a wgpu::Device,
    pub(super) surface_format: wgpu::TextureFormat,
    pub(super) surface_size: (u32, u32),
}

pub(super) struct WindowRedrawPresentCaptureCommands {
    pub(super) command_buffers: Vec<wgpu::CommandBuffer>,
    #[cfg(feature = "diag-screenshots")]
    pub(super) screenshot_inflight: Option<InFlightCapture>,
    pub(super) bundle_screenshot_readback: Option<WindowRedrawBundleScreenshotReadback>,
}

pub(super) fn prepare_window_redraw_present_capture_commands(
    input: WindowRedrawPresentCaptureCommandsInput<'_>,
) -> WindowRedrawPresentCaptureCommands {
    let mut command_buffers = input.command_buffers;
    command_buffers.push(input.ui_cmd);

    #[cfg(feature = "diag-screenshots")]
    let screenshot_inflight =
        super::window_redraw_diag_screenshots::begin_window_redraw_diag_screenshot_capture(
            input.diag_screenshots,
            input.app_window,
            input.source_texture,
            input.device,
            input.surface_format,
            input.surface_size,
            &mut command_buffers,
        );

    let screenshot_dir = input.bundle_screenshots.poll_request_dir();
    let bundle_screenshot_readback =
        super::window_redraw_diag_screenshots::begin_window_redraw_bundle_screenshot_readback(
            input.bundle_screenshots,
            screenshot_dir,
            input.source_texture,
            input.device,
            input.surface_format,
            input.surface_size,
            &mut command_buffers,
        );

    WindowRedrawPresentCaptureCommands {
        command_buffers,
        #[cfg(feature = "diag-screenshots")]
        screenshot_inflight,
        bundle_screenshot_readback,
    }
}
