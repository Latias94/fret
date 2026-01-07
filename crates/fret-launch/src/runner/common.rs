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

pub struct WinitHotReloadContext<'a, S> {
    pub app: &'a mut App,
    pub services: &'a mut dyn UiServices,
    pub window: fret_core::AppWindowId,
    pub state: &'a mut S,
}

/// A function-pointer based `WinitAppDriver`.
///
/// This is intended as an ergonomic and hotpatch-friendly alternative to implementing
/// the `WinitAppDriver` trait directly.
pub struct FnDriver<D, S> {
    driver_state: D,
    create_window_state: fn(&mut D, &mut App, fret_core::AppWindowId) -> S,
    handle_event: for<'d, 'cx, 'e> fn(&'d mut D, WinitEventContext<'cx, S>, &'e Event),
    render: for<'d, 'cx> fn(&'d mut D, WinitRenderContext<'cx, S>),
    hooks: FnDriverHooks<D, S>,
}

impl<D, S> FnDriver<D, S> {
    pub fn new(
        driver_state: D,
        create_window_state: fn(&mut D, &mut App, fret_core::AppWindowId) -> S,
        handle_event: for<'d, 'cx, 'e> fn(&'d mut D, WinitEventContext<'cx, S>, &'e Event),
        render: for<'d, 'cx> fn(&'d mut D, WinitRenderContext<'cx, S>),
    ) -> Self {
        Self {
            driver_state,
            create_window_state,
            handle_event,
            render,
            hooks: FnDriverHooks::default(),
        }
    }

    pub fn with_init(mut self, init: fn(&mut D, &mut App, fret_core::AppWindowId)) -> Self {
        self.hooks.init = Some(init);
        self
    }

    pub fn with_gpu_ready(
        mut self,
        gpu_ready: fn(&mut D, &mut App, &WgpuContext, &mut Renderer),
    ) -> Self {
        self.hooks.gpu_ready = Some(gpu_ready);
        self
    }

    pub fn with_hooks(mut self, f: impl FnOnce(&mut FnDriverHooks<D, S>)) -> Self {
        f(&mut self.hooks);
        self
    }

    pub fn hooks(&self) -> &FnDriverHooks<D, S> {
        &self.hooks
    }

    pub fn hooks_mut(&mut self) -> &mut FnDriverHooks<D, S> {
        &mut self.hooks
    }

    pub fn driver_state(&self) -> &D {
        &self.driver_state
    }

    pub fn driver_state_mut(&mut self) -> &mut D {
        &mut self.driver_state
    }
}

pub struct FnDriverHooks<D, S> {
    pub init: Option<fn(&mut D, &mut App, fret_core::AppWindowId)>,
    pub gpu_ready: Option<fn(&mut D, &mut App, &WgpuContext, &mut Renderer)>,
    pub hot_reload_global: Option<for<'d, 'cx> fn(&'d mut D, WinitGlobalContext<'cx>)>,
    pub hot_reload_window: Option<for<'d, 'cx> fn(&'d mut D, WinitHotReloadContext<'cx, S>)>,
    pub gpu_frame_prepare: Option<
        for<'d> fn(
            &'d mut D,
            &mut App,
            fret_core::AppWindowId,
            &mut S,
            &WgpuContext,
            &mut Renderer,
            f32,
        ),
    >,
    pub record_engine_frame: Option<
        for<'d> fn(
            &'d mut D,
            &mut App,
            fret_core::AppWindowId,
            &mut S,
            &WgpuContext,
            &mut Renderer,
            f32,
            TickId,
            FrameId,
        ) -> EngineFrameUpdate,
    >,
    pub viewport_input: Option<fn(&mut D, &mut App, ViewportInputEvent)>,
    pub dock_op: Option<fn(&mut D, &mut App, fret_core::DockOp)>,
    pub handle_command:
        Option<for<'d, 'cx> fn(&'d mut D, WinitCommandContext<'cx, S>, fret_app::CommandId)>,
    pub handle_global_command:
        Option<for<'d, 'cx> fn(&'d mut D, WinitGlobalContext<'cx>, fret_app::CommandId)>,
    pub handle_model_changes:
        Option<for<'d, 'cx> fn(&'d mut D, WinitWindowContext<'cx, S>, &'cx [fret_app::ModelId])>,
    pub handle_global_changes:
        Option<for<'d, 'cx> fn(&'d mut D, WinitWindowContext<'cx, S>, &'cx [std::any::TypeId])>,
    pub window_create_spec:
        Option<fn(&mut D, &mut App, &fret_app::CreateWindowRequest) -> Option<WindowCreateSpec>>,
    pub window_created:
        Option<fn(&mut D, &mut App, &fret_app::CreateWindowRequest, fret_core::AppWindowId)>,
    pub before_close_window: Option<fn(&mut D, &mut App, fret_core::AppWindowId) -> bool>,
    pub accessibility_snapshot: Option<
        fn(
            &mut D,
            &mut App,
            fret_core::AppWindowId,
            &mut S,
        ) -> Option<Arc<fret_core::SemanticsSnapshot>>,
    >,
    pub accessibility_focus:
        Option<fn(&mut D, &mut App, fret_core::AppWindowId, &mut S, fret_core::NodeId)>,
    pub accessibility_invoke: Option<
        fn(
            &mut D,
            &mut App,
            &mut dyn UiServices,
            fret_core::AppWindowId,
            &mut S,
            fret_core::NodeId,
        ),
    >,
    pub accessibility_set_value_text: Option<
        fn(
            &mut D,
            &mut App,
            &mut dyn UiServices,
            fret_core::AppWindowId,
            &mut S,
            fret_core::NodeId,
            &str,
        ),
    >,
    pub accessibility_set_value_numeric: Option<
        fn(
            &mut D,
            &mut App,
            &mut dyn UiServices,
            fret_core::AppWindowId,
            &mut S,
            fret_core::NodeId,
            f64,
        ),
    >,
    pub accessibility_set_text_selection: Option<
        fn(
            &mut D,
            &mut App,
            &mut dyn UiServices,
            fret_core::AppWindowId,
            &mut S,
            fret_core::NodeId,
            u32,
            u32,
        ),
    >,
    pub accessibility_replace_selected_text: Option<
        fn(
            &mut D,
            &mut App,
            &mut dyn UiServices,
            fret_core::AppWindowId,
            &mut S,
            fret_core::NodeId,
            &str,
        ),
    >,
}

impl<D, S> Default for FnDriverHooks<D, S> {
    fn default() -> Self {
        Self {
            init: None,
            gpu_ready: None,
            hot_reload_global: None,
            hot_reload_window: None,
            gpu_frame_prepare: None,
            record_engine_frame: None,
            viewport_input: None,
            dock_op: None,
            handle_command: None,
            handle_global_command: None,
            handle_model_changes: None,
            handle_global_changes: None,
            window_create_spec: None,
            window_created: None,
            before_close_window: None,
            accessibility_snapshot: None,
            accessibility_focus: None,
            accessibility_invoke: None,
            accessibility_set_value_text: None,
            accessibility_set_value_numeric: None,
            accessibility_set_text_selection: None,
            accessibility_replace_selected_text: None,
        }
    }
}

/// Trait-based runner driver integration.
///
/// Prefer `FnDriver` when you want a stable, function-pointer based “hot anchor” surface for dev
/// hotpatch workflows (see ADR 0107). This trait remains supported for compatibility and for
/// drivers that benefit from trait-based struct organization.
///
/// TODO: Once `FnDriver` covers all required hooks and in-tree call sites have migrated, remove
/// `WinitAppDriver` from the public surface to make `FnDriver` the single supported entrypoint.
pub trait WinitAppDriver {
    type WindowState;

    fn init(&mut self, _app: &mut App, _main_window: fret_core::AppWindowId) {}

    fn gpu_ready(&mut self, _app: &mut App, _context: &WgpuContext, _renderer: &mut Renderer) {}

    /// Dev-only hot reload hook.
    ///
    /// Runner callers should guarantee a "hard reset" boundary that does not require preserving
    /// any previously-registered callbacks. This is intentionally a best-effort hook that allows
    /// app code to reset retained UI runtime state without rebuilding app models.
    fn hot_reload_global(&mut self, _app: &mut App, _services: &mut dyn UiServices) {}

    /// Dev-only hot reload hook.
    ///
    /// Default behavior is a no-op to keep production behavior unchanged. App code can implement
    /// this to reset per-window retained UI runtime state (e.g. `UiTree`) while preserving models.
    fn hot_reload_window(
        &mut self,
        _app: &mut App,
        _services: &mut dyn UiServices,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
    ) {
    }

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

impl<D, S> WinitAppDriver for FnDriver<D, S> {
    type WindowState = S;

    fn init(&mut self, app: &mut App, main_window: fret_core::AppWindowId) {
        if let Some(init) = self.hooks.init {
            init(&mut self.driver_state, app, main_window);
        }
    }

    fn gpu_ready(&mut self, app: &mut App, context: &WgpuContext, renderer: &mut Renderer) {
        if let Some(gpu_ready) = self.hooks.gpu_ready {
            gpu_ready(&mut self.driver_state, app, context, renderer);
        }
    }

    fn hot_reload_global(&mut self, app: &mut App, services: &mut dyn UiServices) {
        if let Some(f) = self.hooks.hot_reload_global {
            f(&mut self.driver_state, WinitGlobalContext { app, services });
        }
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
    ) {
        if let Some(f) = self.hooks.hot_reload_window {
            f(
                &mut self.driver_state,
                WinitHotReloadContext {
                    app,
                    services,
                    window,
                    state,
                },
            );
        }
    }

    fn gpu_frame_prepare(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        context: &WgpuContext,
        renderer: &mut Renderer,
        scale_factor: f32,
    ) {
        if let Some(f) = self.hooks.gpu_frame_prepare {
            f(
                &mut self.driver_state,
                app,
                window,
                state,
                context,
                renderer,
                scale_factor,
            );
        }
    }

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
        if let Some(f) = self.hooks.record_engine_frame {
            return f(
                &mut self.driver_state,
                app,
                window,
                state,
                context,
                renderer,
                scale_factor,
                tick_id,
                frame_id,
            );
        }
        EngineFrameUpdate::default()
    }

    fn viewport_input(&mut self, app: &mut App, event: ViewportInputEvent) {
        if let Some(f) = self.hooks.viewport_input {
            f(&mut self.driver_state, app, event);
        }
    }

    fn dock_op(&mut self, app: &mut App, op: fret_core::DockOp) {
        if let Some(f) = self.hooks.dock_op {
            f(&mut self.driver_state, app, op);
        }
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: fret_app::CommandId,
    ) {
        if let Some(f) = self.hooks.handle_command {
            f(&mut self.driver_state, context, command);
        }
    }

    fn handle_global_command(
        &mut self,
        context: WinitGlobalContext<'_>,
        command: fret_app::CommandId,
    ) {
        if let Some(f) = self.hooks.handle_global_command {
            f(&mut self.driver_state, context, command);
        }
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        if let Some(f) = self.hooks.handle_model_changes {
            f(&mut self.driver_state, context, changed);
        }
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        if let Some(f) = self.hooks.handle_global_changes {
            f(&mut self.driver_state, context, changed);
        }
    }

    fn create_window_state(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
    ) -> Self::WindowState {
        (self.create_window_state)(&mut self.driver_state, app, window)
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        (self.handle_event)(&mut self.driver_state, context, event)
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        (self.render)(&mut self.driver_state, context)
    }

    fn window_create_spec(
        &mut self,
        app: &mut App,
        request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        if let Some(f) = self.hooks.window_create_spec {
            return f(&mut self.driver_state, app, request);
        }
        None
    }

    fn window_created(
        &mut self,
        app: &mut App,
        request: &fret_app::CreateWindowRequest,
        new_window: fret_core::AppWindowId,
    ) {
        if let Some(f) = self.hooks.window_created {
            f(&mut self.driver_state, app, request, new_window);
        }
    }

    fn before_close_window(&mut self, app: &mut App, window: fret_core::AppWindowId) -> bool {
        if let Some(f) = self.hooks.before_close_window {
            return f(&mut self.driver_state, app, window);
        }
        true
    }

    fn accessibility_snapshot(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
    ) -> Option<Arc<fret_core::SemanticsSnapshot>> {
        if let Some(f) = self.hooks.accessibility_snapshot {
            return f(&mut self.driver_state, app, window, state);
        }
        None
    }

    fn accessibility_focus(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        if let Some(f) = self.hooks.accessibility_focus {
            f(&mut self.driver_state, app, window, state, target);
        }
    }

    fn accessibility_invoke(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        if let Some(f) = self.hooks.accessibility_invoke {
            f(&mut self.driver_state, app, services, window, state, target);
        }
    }

    fn accessibility_set_value_text(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        if let Some(f) = self.hooks.accessibility_set_value_text {
            f(
                &mut self.driver_state,
                app,
                services,
                window,
                state,
                target,
                value,
            );
        }
    }

    fn accessibility_set_value_numeric(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: f64,
    ) {
        if let Some(f) = self.hooks.accessibility_set_value_numeric {
            f(
                &mut self.driver_state,
                app,
                services,
                window,
                state,
                target,
                value,
            );
        }
    }

    fn accessibility_set_text_selection(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        anchor: u32,
        focus: u32,
    ) {
        if let Some(f) = self.hooks.accessibility_set_text_selection {
            f(
                &mut self.driver_state,
                app,
                services,
                window,
                state,
                target,
                anchor,
                focus,
            );
        }
    }

    fn accessibility_replace_selected_text(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        if let Some(f) = self.hooks.accessibility_replace_selected_text {
            f(
                &mut self.driver_state,
                app,
                services,
                window,
                state,
                target,
                value,
            );
        }
    }
}
