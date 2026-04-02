use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

use fret_render::{ClearColor, WgpuContext};
use fret_runtime::WindowStyleRequest;

#[cfg(not(target_arch = "wasm32"))]
use crate::RunnerError;

pub struct WinitRunnerConfig {
    pub main_window_title: String,
    pub main_window_size: super::WindowLogicalSize,
    /// Optional minimum logical surface size applied to the main OS window.
    pub main_window_min_size: Option<super::WindowLogicalSize>,
    /// Optional maximum logical surface size applied to the main OS window.
    pub main_window_max_size: Option<super::WindowLogicalSize>,
    /// Optional logical surface resize increments applied to the main OS window.
    pub main_window_resize_increments: Option<super::WindowLogicalSize>,
    pub main_window_position: Option<super::WindowPosition>,
    /// Create-time style request for the main OS window.
    ///
    /// Notes:
    /// - Some facets are only honored at window creation (e.g. `decorations`, `resizable`).
    /// - Runners should treat unsupported facets as best-effort and clamp via capabilities.
    pub main_window_style: WindowStyleRequest,
    pub default_window_title: String,
    pub default_window_size: super::WindowLogicalSize,
    /// Optional minimum logical surface size applied to fallback-created OS windows.
    pub default_window_min_size: Option<super::WindowLogicalSize>,
    /// Optional maximum logical surface size applied to fallback-created OS windows.
    pub default_window_max_size: Option<super::WindowLogicalSize>,
    /// Optional logical surface resize increments applied to fallback-created OS windows.
    pub default_window_resize_increments: Option<super::WindowLogicalSize>,
    pub default_window_position: Option<super::WindowPosition>,
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
    #[cfg(not(target_arch = "wasm32"))]
    /// Create the GPU context via a host callback given the main window.
    ///
    /// Note: on wasm32 the launcher initializes WGPU asynchronously and does not currently support
    /// user factories.
    Factory(Box<WgpuFactoryFn>),
}

#[cfg(not(target_arch = "wasm32"))]
type WgpuFactoryFn = dyn FnOnce(
        Arc<dyn winit::window::Window>,
    ) -> Result<(WgpuContext, wgpu::Surface<'static>), RunnerError>
    + 'static;

impl Default for WinitRunnerConfig {
    fn default() -> Self {
        Self {
            main_window_title: "fret".to_string(),
            main_window_size: super::WindowLogicalSize::new(1280.0, 720.0),
            main_window_min_size: None,
            main_window_max_size: None,
            main_window_resize_increments: None,
            main_window_position: None,
            main_window_style: WindowStyleRequest::default(),
            default_window_title: "fret".to_string(),
            default_window_size: super::WindowLogicalSize::new(640.0, 480.0),
            default_window_min_size: None,
            default_window_max_size: None,
            default_window_resize_increments: None,
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
    #[doc(hidden)]
    pub fn main_window_spec(&self) -> super::WindowCreateSpec {
        let mut spec =
            super::WindowCreateSpec::new(self.main_window_title.clone(), self.main_window_size);
        if let Some(min_size) = self.main_window_min_size {
            spec = spec.with_min_size(min_size);
        }
        if let Some(max_size) = self.main_window_max_size {
            spec = spec.with_max_size(max_size);
        }
        if let Some(resize_increments) = self.main_window_resize_increments {
            spec = spec.with_resize_increments(resize_increments);
        }
        if let Some(position) = self.main_window_position {
            spec = spec.with_position(position);
        }
        spec.normalize_size_constraints();
        spec
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[doc(hidden)]
    pub fn default_window_spec(&self) -> super::WindowCreateSpec {
        let mut spec = super::WindowCreateSpec::new(
            self.default_window_title.clone(),
            self.default_window_size,
        );
        if let Some(min_size) = self.default_window_min_size {
            spec = spec.with_min_size(min_size);
        }
        if let Some(max_size) = self.default_window_max_size {
            spec = spec.with_max_size(max_size);
        }
        if let Some(resize_increments) = self.default_window_resize_increments {
            spec = spec.with_resize_increments(resize_increments);
        }
        if let Some(position) = self.default_window_position {
            spec = spec.with_position(position);
        }
        spec.normalize_size_constraints();
        spec
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::WindowLogicalSize;

    #[test]
    fn main_window_spec_carries_size_constraints_and_clamps_initial_size() {
        let config = WinitRunnerConfig {
            main_window_size: WindowLogicalSize::new(320.0, 240.0),
            main_window_min_size: Some(WindowLogicalSize::new(420.0, 560.0)),
            main_window_max_size: Some(WindowLogicalSize::new(900.0, 700.0)),
            main_window_resize_increments: Some(WindowLogicalSize::new(24.0, 16.0)),
            ..Default::default()
        };

        let spec = config.main_window_spec();

        assert_eq!(spec.size, WindowLogicalSize::new(420.0, 560.0));
        assert_eq!(spec.min_size, Some(WindowLogicalSize::new(420.0, 560.0)));
        assert_eq!(spec.max_size, Some(WindowLogicalSize::new(900.0, 700.0)));
        assert_eq!(
            spec.resize_increments,
            Some(WindowLogicalSize::new(24.0, 16.0))
        );
    }

    #[test]
    fn default_window_spec_repairs_inverted_constraints() {
        let config = WinitRunnerConfig {
            default_window_size: WindowLogicalSize::new(1200.0, 900.0),
            default_window_min_size: Some(WindowLogicalSize::new(460.0, 360.0)),
            default_window_max_size: Some(WindowLogicalSize::new(420.0, 320.0)),
            default_window_resize_increments: Some(WindowLogicalSize::new(18.0, 18.0)),
            ..Default::default()
        };

        let spec = config.default_window_spec();

        assert_eq!(spec.size, WindowLogicalSize::new(460.0, 360.0));
        assert_eq!(spec.min_size, Some(WindowLogicalSize::new(460.0, 360.0)));
        assert_eq!(spec.max_size, Some(WindowLogicalSize::new(460.0, 360.0)));
        assert_eq!(
            spec.resize_increments,
            Some(WindowLogicalSize::new(18.0, 18.0))
        );
    }
}
