use fret_app::App;
use fret_core::{Event, UiServices, ViewportInputEvent};
use fret_render::{Renderer, WgpuContext};
use fret_runtime::{FrameId, TickId};

use super::{
    EngineFrameUpdate, WindowCreateSpec, WinitCommandContext, WinitEventContext,
    WinitGlobalContext, WinitRenderContext, WinitWindowContext,
};

/// Trait-based runner driver integration.
///
/// Prefer `FnDriver` when you want a stable, function-pointer based “hot anchor” surface for dev
/// hotpatch workflows (see ADR 0105). This trait remains supported for compatibility and for
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
