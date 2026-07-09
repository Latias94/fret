use std::path::PathBuf;

use super::diag_bundle_screenshots::{DiagBundleScreenshotCapture, PendingScreenshotReadback};

#[cfg(feature = "diag-screenshots")]
use super::diag_screenshots::{DiagScreenshotCapture, InFlightCapture};
#[cfg(feature = "diag-screenshots")]
use fret_core::AppWindowId;
#[cfg(feature = "diag-screenshots")]
use slotmap::Key as _;

pub(super) struct WindowRedrawBundleScreenshotReadback {
    pending: PendingScreenshotReadback,
    dir: PathBuf,
}

#[cfg(feature = "diag-screenshots")]
pub(super) fn poll_window_redraw_diag_screenshot_requests(
    diag: Option<&mut DiagScreenshotCapture>,
) {
    if let Some(diag) = diag {
        diag.poll();
    }
}

#[cfg(feature = "diag-screenshots")]
#[allow(clippy::too_many_arguments)]
pub(super) fn begin_window_redraw_diag_screenshot_capture(
    diag: Option<&mut DiagScreenshotCapture>,
    app_window: AppWindowId,
    frame_view: Option<&(wgpu::SurfaceTexture, wgpu::TextureView)>,
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    surface_size: (u32, u32),
    cmd_buffers: &mut Vec<wgpu::CommandBuffer>,
) -> Option<InFlightCapture> {
    let (Some(diag), Some((frame, _view))) = (diag, frame_view) else {
        return None;
    };

    let window_ffi = app_window.data().as_ffi();
    let (cmd, inflight) = diag.begin_capture_for_window(
        device,
        window_ffi,
        &frame.texture,
        surface_format,
        surface_size,
    )?;
    cmd_buffers.push(cmd);
    Some(inflight)
}

#[cfg(feature = "diag-screenshots")]
pub(super) fn finish_window_redraw_diag_screenshot_capture(
    diag: Option<&mut DiagScreenshotCapture>,
    device: &wgpu::Device,
    app_window: AppWindowId,
    inflight: Option<InFlightCapture>,
) {
    let (Some(diag), Some(inflight)) = (diag, inflight) else {
        return;
    };

    if let Err(err) = diag.finish_capture(device, inflight) {
        tracing::warn!(
            error = %err,
            window = ?app_window,
            "diag screenshot: capture failed"
        );
    }
}

pub(super) fn begin_window_redraw_bundle_screenshot_readback(
    diag: &DiagBundleScreenshotCapture,
    screenshot_dir: Option<PathBuf>,
    frame_view: Option<&(wgpu::SurfaceTexture, wgpu::TextureView)>,
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    surface_size: (u32, u32),
    cmd_buffers: &mut Vec<wgpu::CommandBuffer>,
) -> Option<WindowRedrawBundleScreenshotReadback> {
    let (Some(dir), Some((frame, _view))) = (screenshot_dir, frame_view) else {
        return None;
    };
    let (pending, copy_cmd) =
        diag.begin_readback(device, &frame.texture, surface_format, surface_size)?;

    cmd_buffers.push(copy_cmd);
    Some(WindowRedrawBundleScreenshotReadback { pending, dir })
}

pub(super) fn finish_window_redraw_bundle_screenshot_readback(
    diag: &DiagBundleScreenshotCapture,
    device: &wgpu::Device,
    pending: Option<WindowRedrawBundleScreenshotReadback>,
    surface_format: wgpu::TextureFormat,
) {
    if let Some(WindowRedrawBundleScreenshotReadback { pending, dir }) = pending {
        let _ = diag.finish_and_write_bmp(device, pending, &dir, surface_format);
    }
}
