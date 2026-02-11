use std::{sync::Arc, time::Duration};

use fret_render::{ClearColor, WgpuContext};
use winit::dpi::{LogicalSize, Position};
use winit::window::Window;

use crate::RunnerError;

pub struct WinitRunnerConfig {
    pub main_window_title: String,
    pub main_window_size: LogicalSize<f64>,
    pub main_window_position: Option<Position>,
    pub default_window_title: String,
    pub default_window_size: LogicalSize<f64>,
    pub default_window_position: Option<Position>,
    /// Physical pixel offset applied when positioning a new window from an anchor point.
    pub new_window_anchor_offset: (f64, f64),
    /// When the main window requests close, exit the event loop.
    pub exit_on_main_window_close: bool,
    /// Line-based wheel delta unit to logical pixels.
    pub wheel_line_delta_px: f32,
    /// Pixel-based wheel delta scale in logical pixels.
    pub wheel_pixel_delta_scale: f32,
    pub frame_interval: Duration,
    pub clear_color: ClearColor,
    /// Upper bound for total bytes read via `Effect::ExternalDropReadAll` for a single token.
    pub external_drop_max_total_bytes: u64,
    /// Upper bound for a single file read via `Effect::ExternalDropReadAll`.
    pub external_drop_max_file_bytes: u64,
    /// Upper bound for number of files processed per `Effect::ExternalDropReadAll`.
    pub external_drop_max_files: usize,
    /// Upper bound for total bytes read via `Effect::FileDialogReadAll` for a single token.
    pub file_dialog_max_total_bytes: u64,
    /// Upper bound for a single file read via `Effect::FileDialogReadAll`.
    pub file_dialog_max_file_bytes: u64,
    /// Upper bound for number of files processed per `Effect::FileDialogReadAll`.
    pub file_dialog_max_files: usize,
    /// Soft upper bound for total GPU memory used by renderer-internal SVG raster caches.
    ///
    /// This is used for `SceneOp::SvgMaskIcon` and `SceneOp::SvgImage` rasterizations.
    pub svg_raster_budget_bytes: u64,
    /// Soft upper bound for total GPU memory used by renderer-owned intermediate targets.
    ///
    /// This covers internal multi-pass steps such as MSAA resolves, effect intermediates, clip masks,
    /// and post-processing substrates (ADR 0118).
    pub renderer_intermediate_budget_bytes: u64,
    /// MSAA sample count used by the renderer's offscreen path pass.
    ///
    /// Set to `1` to disable MSAA-based AA for paths (more compatible, lower quality).
    pub path_msaa_samples: u32,
    /// Enable platform accessibility integration (AccessKit + winit adapter).
    pub accessibility_enabled: bool,
    /// Optional overrides for the default font family selection used by the text system.
    pub text_font_families: fret_render::TextFontFamilyConfig,
    pub wgpu_init: WgpuInit,
    /// Canvas element id used by the wasm32 backend.
    pub web_canvas_id: String,

    /// Soft upper bound on total CPU->GPU upload bytes per rendered frame (per window) for
    /// streaming image updates (ADR 0121).
    pub streaming_upload_budget_bytes_per_frame: u64,
    /// Soft upper bound on pending streaming update bytes retained for a window (ADR 0121).
    ///
    /// Note: this is a forward-looking knob. The initial streaming update MVP applies coalescing
    /// and per-frame budget at drain points but does not yet maintain a cross-frame pending queue.
    pub streaming_staging_budget_bytes: u64,
    /// When enabled, the runner updates `fret_core::StreamingUploadPerfSnapshot` as an app global.
    pub streaming_perf_snapshot_enabled: bool,
    /// When enabled, the runner may emit `Event::{ImageUpdateApplied,ImageUpdateDropped}` for
    /// streaming image updates (ADR 0124).
    pub streaming_update_ack_enabled: bool,

    /// Enable experimental GPU-assisted NV12 conversion for streaming image updates when supported
    /// by the selected backend/device (ADR 0122).
    pub streaming_nv12_gpu_convert_enabled: bool,
}

pub enum WgpuInit {
    /// Create a `WgpuContext` internally using a surface-compatible adapter.
    CreateDefault,
    /// Use a host-provided GPU context. The runner will create surfaces via `context.instance`
    /// and assumes the adapter/device are compatible with those surfaces.
    Provided(WgpuContext),
    /// Create the GPU context via a host callback given the main window.
    ///
    /// Note: on wasm32 the launcher initializes WGPU asynchronously and does not currently support
    /// user factories.
    Factory(Box<WgpuFactoryFn>),
}

type WgpuFactoryFn = dyn FnOnce(Arc<dyn Window>) -> Result<(WgpuContext, wgpu::Surface<'static>), RunnerError>
    + 'static;

impl Default for WinitRunnerConfig {
    fn default() -> Self {
        Self {
            main_window_title: "fret".to_string(),
            main_window_size: LogicalSize::new(1280.0, 720.0),
            main_window_position: None,
            default_window_title: "fret".to_string(),
            default_window_size: LogicalSize::new(640.0, 480.0),
            default_window_position: None,
            new_window_anchor_offset: (-40.0, -20.0),
            exit_on_main_window_close: true,
            wheel_line_delta_px: 20.0,
            wheel_pixel_delta_scale: 1.0,
            frame_interval: Duration::from_millis(8),
            clear_color: ClearColor::default(),
            external_drop_max_total_bytes: 64 * 1024 * 1024,
            external_drop_max_file_bytes: 32 * 1024 * 1024,
            external_drop_max_files: 128,
            file_dialog_max_total_bytes: 64 * 1024 * 1024,
            file_dialog_max_file_bytes: 32 * 1024 * 1024,
            file_dialog_max_files: 128,
            svg_raster_budget_bytes: 64 * 1024 * 1024,
            renderer_intermediate_budget_bytes: 256 * 1024 * 1024,
            path_msaa_samples: 4,
            accessibility_enabled: true,
            text_font_families: fret_render::TextFontFamilyConfig::default(),
            wgpu_init: WgpuInit::CreateDefault,
            web_canvas_id: "fret-canvas".to_string(),
            streaming_upload_budget_bytes_per_frame: 64 * 1024 * 1024,
            streaming_staging_budget_bytes: 128 * 1024 * 1024,
            streaming_perf_snapshot_enabled: false,
            streaming_update_ack_enabled: false,
            streaming_nv12_gpu_convert_enabled: false,
        }
    }
}

impl WinitRunnerConfig {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn main_window_spec(&self) -> super::WindowCreateSpec {
        let mut spec =
            super::WindowCreateSpec::new(self.main_window_title.clone(), self.main_window_size);
        if let Some(position) = self.main_window_position {
            spec = spec.with_position(position);
        }
        spec
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn default_window_spec(&self) -> super::WindowCreateSpec {
        let mut spec = super::WindowCreateSpec::new(
            self.default_window_title.clone(),
            self.default_window_size,
        );
        if let Some(position) = self.default_window_position {
            spec = spec.with_position(position);
        }
        spec
    }
}
