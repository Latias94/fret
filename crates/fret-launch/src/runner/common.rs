use std::{fmt, sync::Arc, time::Duration};

use fret_app::App;
use fret_core::{Event, Rect, Scene, UiServices, ViewportInputEvent};
use fret_render::{ClearColor, Renderer, WgpuContext};
use fret_runtime::{FrameId, TickId};
use winit::dpi::{LogicalSize, Position};
use winit::window::Window;

use crate::RunnerError;

pub enum RenderTargetUpdate {
    Update {
        id: fret_core::RenderTargetId,
        desc: fret_render::RenderTargetDescriptor,
    },
    Unregister {
        id: fret_core::RenderTargetId,
    },
}

impl fmt::Debug for RenderTargetUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Update { id, desc } => f
                .debug_struct("Update")
                .field("id", id)
                .field("size", &desc.size)
                .field("format", &desc.format)
                .field("color_space", &desc.color_space)
                .field("view", &"<wgpu::TextureView>")
                .finish(),
            Self::Unregister { id } => f.debug_struct("Unregister").field("id", id).finish(),
        }
    }
}

#[derive(Default)]
pub struct EngineFrameUpdate {
    pub target_updates: Vec<RenderTargetUpdate>,
    pub command_buffers: Vec<wgpu::CommandBuffer>,
}

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
            path_msaa_samples: 4,
            accessibility_enabled: true,
            text_font_families: fret_render::TextFontFamilyConfig::default(),
            wgpu_init: WgpuInit::CreateDefault,
            web_canvas_id: "fret-canvas".to_string(),
        }
    }
}

impl WinitRunnerConfig {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn main_window_spec(&self) -> WindowCreateSpec {
        let mut spec = WindowCreateSpec::new(self.main_window_title.clone(), self.main_window_size);
        if let Some(position) = self.main_window_position {
            spec = spec.with_position(position);
        }
        spec
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn default_window_spec(&self) -> WindowCreateSpec {
        let mut spec =
            WindowCreateSpec::new(self.default_window_title.clone(), self.default_window_size);
        if let Some(position) = self.default_window_position {
            spec = spec.with_position(position);
        }
        spec
    }
}

#[derive(Debug, Clone)]
pub struct WindowCreateSpec {
    pub title: String,
    pub size: LogicalSize<f64>,
    pub position: Option<Position>,
    pub visible: bool,
}

impl WindowCreateSpec {
    pub fn new(title: impl Into<String>, size: LogicalSize<f64>) -> Self {
        Self {
            title: title.into(),
            size,
            position: None,
            visible: true,
        }
    }

    pub fn with_position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

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

pub trait WinitAppDriver {
    type WindowState;

    fn init(&mut self, _app: &mut App, _main_window: fret_core::AppWindowId) {}

    fn gpu_ready(&mut self, _app: &mut App, _context: &WgpuContext, _renderer: &mut Renderer) {}

    #[allow(clippy::too_many_arguments)]
    fn gpu_frame_prepare(
        &mut self,
        _app: &mut App,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
        _context: &WgpuContext,
        _renderer: &mut Renderer,
        _scale_factor: f32,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    fn record_engine_frame(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        context: &WgpuContext,
        renderer: &mut Renderer,
        scale_factor: f32,
        tick_id: TickId,
        frame_id: FrameId,
    ) -> EngineFrameUpdate {
        EngineFrameUpdate {
            target_updates: Vec::new(),
            command_buffers: self.record_engine_commands(
                app,
                window,
                state,
                context,
                renderer,
                scale_factor,
                tick_id,
                frame_id,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn record_engine_commands(
        &mut self,
        _app: &mut App,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
        _context: &WgpuContext,
        _renderer: &mut Renderer,
        _scale_factor: f32,
        _tick_id: TickId,
        _frame_id: FrameId,
    ) -> Vec<wgpu::CommandBuffer> {
        Vec::new()
    }

    fn viewport_input(&mut self, _app: &mut App, _event: ViewportInputEvent) {}

    fn dock_op(&mut self, _app: &mut App, _op: fret_core::DockOp) {}

    fn handle_command(
        &mut self,
        _context: WinitCommandContext<'_, Self::WindowState>,
        _command: fret_app::CommandId,
    ) {
    }

    fn handle_global_command(
        &mut self,
        _context: WinitGlobalContext<'_>,
        _command: fret_app::CommandId,
    ) {
    }

    fn handle_model_changes(
        &mut self,
        _context: WinitWindowContext<'_, Self::WindowState>,
        _changed: &[fret_app::ModelId],
    ) {
    }

    fn handle_global_changes(
        &mut self,
        _context: WinitWindowContext<'_, Self::WindowState>,
        _changed: &[std::any::TypeId],
    ) {
    }

    fn create_window_state(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
    ) -> Self::WindowState;

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event);

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>);

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: fret_core::AppWindowId,
    ) {
    }

    fn before_close_window(&mut self, _app: &mut App, _window: fret_core::AppWindowId) -> bool {
        true
    }

    fn accessibility_snapshot(
        &mut self,
        _app: &mut App,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
    ) -> Option<std::sync::Arc<fret_core::SemanticsSnapshot>> {
        None
    }

    fn accessibility_focus(
        &mut self,
        _app: &mut App,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
        _target: fret_core::NodeId,
    ) {
    }

    fn accessibility_invoke(
        &mut self,
        _app: &mut App,
        _services: &mut dyn UiServices,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
        _target: fret_core::NodeId,
    ) {
    }

    fn accessibility_set_value_text(
        &mut self,
        _app: &mut App,
        _services: &mut dyn UiServices,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
        _target: fret_core::NodeId,
        _value: &str,
    ) {
    }

    fn accessibility_set_value_numeric(
        &mut self,
        _app: &mut App,
        _services: &mut dyn UiServices,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
        _target: fret_core::NodeId,
        _value: f64,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    fn accessibility_set_text_selection(
        &mut self,
        _app: &mut App,
        _services: &mut dyn UiServices,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
        _target: fret_core::NodeId,
        _anchor: u32,
        _focus: u32,
    ) {
    }

    fn accessibility_replace_selected_text(
        &mut self,
        _app: &mut App,
        _services: &mut dyn UiServices,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
        _target: fret_core::NodeId,
        _value: &str,
    ) {
    }
}
