use std::sync::Arc;

use fret_app::App;
use fret_core::{Event, UiServices, ViewportInputEvent};
use fret_render::{Renderer, WgpuContext};
use fret_runtime::{FrameId, TickId};

use super::{
    EngineFrameUpdate, WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext,
    WinitGlobalContext, WinitHotReloadContext, WinitRenderContext, WinitWindowContext,
};

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
    pub init: Option<FnDriverInitHook<D>>,
    pub gpu_ready: Option<FnDriverGpuReadyHook<D>>,
    pub hot_reload_global: Option<FnDriverHotReloadGlobalHook<D>>,
    pub hot_reload_window: Option<FnDriverHotReloadWindowHook<D, S>>,
    pub gpu_frame_prepare: Option<FnDriverGpuFramePrepareHook<D, S>>,
    pub record_engine_frame: Option<FnDriverRecordEngineFrameHook<D, S>>,
    pub viewport_input: Option<FnDriverViewportInputHook<D>>,
    pub dock_op: Option<FnDriverDockOpHook<D>>,
    pub handle_command: Option<FnDriverHandleCommandHook<D, S>>,
    pub handle_global_command: Option<FnDriverHandleGlobalCommandHook<D>>,
    pub handle_model_changes: Option<FnDriverHandleModelChangesHook<D, S>>,
    pub handle_global_changes: Option<FnDriverHandleGlobalChangesHook<D, S>>,
    pub window_create_spec: Option<FnDriverWindowCreateSpecHook<D>>,
    pub window_created: Option<FnDriverWindowCreatedHook<D>>,
    pub before_close_window: Option<FnDriverBeforeCloseWindowHook<D>>,
    pub accessibility_snapshot: Option<FnDriverAccessibilitySnapshotHook<D, S>>,
    pub accessibility_focus: Option<FnDriverAccessibilityFocusHook<D, S>>,
    pub accessibility_invoke: Option<FnDriverAccessibilityInvokeHook<D, S>>,
    pub accessibility_set_value_text: Option<FnDriverAccessibilitySetValueTextHook<D, S>>,
    pub accessibility_set_value_numeric: Option<FnDriverAccessibilitySetValueNumericHook<D, S>>,
    pub accessibility_set_text_selection: Option<FnDriverAccessibilitySetTextSelectionHook<D, S>>,
    pub accessibility_replace_selected_text:
        Option<FnDriverAccessibilityReplaceSelectedTextHook<D, S>>,
}

pub type FnDriverInitHook<D> = fn(&mut D, &mut App, fret_core::AppWindowId);
pub type FnDriverGpuReadyHook<D> = fn(&mut D, &mut App, &WgpuContext, &mut Renderer);
pub type FnDriverHotReloadGlobalHook<D> = for<'d, 'cx> fn(&'d mut D, WinitGlobalContext<'cx>);
pub type FnDriverHotReloadWindowHook<D, S> =
    for<'d, 'cx> fn(&'d mut D, WinitHotReloadContext<'cx, S>);
pub type FnDriverGpuFramePrepareHook<D, S> = for<'d> fn(
    &'d mut D,
    &mut App,
    fret_core::AppWindowId,
    &mut S,
    &WgpuContext,
    &mut Renderer,
    f32,
);
pub type FnDriverRecordEngineFrameHook<D, S> = for<'d> fn(
    &'d mut D,
    &mut App,
    fret_core::AppWindowId,
    &mut S,
    &WgpuContext,
    &mut Renderer,
    f32,
    TickId,
    FrameId,
) -> EngineFrameUpdate;
pub type FnDriverViewportInputHook<D> = fn(&mut D, &mut App, ViewportInputEvent);
pub type FnDriverDockOpHook<D> = fn(&mut D, &mut App, fret_core::DockOp);
pub type FnDriverHandleCommandHook<D, S> =
    for<'d, 'cx> fn(&'d mut D, WinitCommandContext<'cx, S>, fret_app::CommandId);
pub type FnDriverHandleGlobalCommandHook<D> =
    for<'d, 'cx> fn(&'d mut D, WinitGlobalContext<'cx>, fret_app::CommandId);
pub type FnDriverHandleModelChangesHook<D, S> =
    for<'d, 'cx> fn(&'d mut D, WinitWindowContext<'cx, S>, &'cx [fret_app::ModelId]);
pub type FnDriverHandleGlobalChangesHook<D, S> =
    for<'d, 'cx> fn(&'d mut D, WinitWindowContext<'cx, S>, &'cx [std::any::TypeId]);
pub type FnDriverWindowCreateSpecHook<D> =
    fn(&mut D, &mut App, &fret_app::CreateWindowRequest) -> Option<WindowCreateSpec>;
pub type FnDriverWindowCreatedHook<D> =
    fn(&mut D, &mut App, &fret_app::CreateWindowRequest, fret_core::AppWindowId);
pub type FnDriverBeforeCloseWindowHook<D> = fn(&mut D, &mut App, fret_core::AppWindowId) -> bool;
pub type FnDriverAccessibilitySnapshotHook<D, S> = fn(
    &mut D,
    &mut App,
    fret_core::AppWindowId,
    &mut S,
) -> Option<Arc<fret_core::SemanticsSnapshot>>;
pub type FnDriverAccessibilityFocusHook<D, S> =
    fn(&mut D, &mut App, fret_core::AppWindowId, &mut S, fret_core::NodeId);
pub type FnDriverAccessibilityInvokeHook<D, S> =
    fn(&mut D, &mut App, &mut dyn UiServices, fret_core::AppWindowId, &mut S, fret_core::NodeId);
pub type FnDriverAccessibilitySetValueTextHook<D, S> = fn(
    &mut D,
    &mut App,
    &mut dyn UiServices,
    fret_core::AppWindowId,
    &mut S,
    fret_core::NodeId,
    &str,
);
pub type FnDriverAccessibilitySetValueNumericHook<D, S> = fn(
    &mut D,
    &mut App,
    &mut dyn UiServices,
    fret_core::AppWindowId,
    &mut S,
    fret_core::NodeId,
    f64,
);
pub type FnDriverAccessibilitySetTextSelectionHook<D, S> = fn(
    &mut D,
    &mut App,
    &mut dyn UiServices,
    fret_core::AppWindowId,
    &mut S,
    fret_core::NodeId,
    u32,
    u32,
);
pub type FnDriverAccessibilityReplaceSelectedTextHook<D, S> = fn(
    &mut D,
    &mut App,
    &mut dyn UiServices,
    fret_core::AppWindowId,
    &mut S,
    fret_core::NodeId,
    &str,
);

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

impl<D, S> WinitAppDriver for FnDriver<D, S> {
    type WindowState = S;

    fn init(&mut self, app: &mut App, main_window: fret_core::AppWindowId) {
        if let Some(init) = self.hooks.init {
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(init);
                hot.call((&mut self.driver_state, app, main_window));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                init(&mut self.driver_state, app, main_window);
            }
        }
    }

    fn gpu_ready(&mut self, app: &mut App, context: &WgpuContext, renderer: &mut Renderer) {
        if let Some(gpu_ready) = self.hooks.gpu_ready {
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(gpu_ready);
                hot.call((&mut self.driver_state, app, context, renderer));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                gpu_ready(&mut self.driver_state, app, context, renderer);
            }
        }
    }

    fn hot_reload_global(&mut self, app: &mut App, services: &mut dyn UiServices) {
        if let Some(f) = self.hooks.hot_reload_global {
            let cx = WinitGlobalContext { app, services };

            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((&mut self.driver_state, cx));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                f(&mut self.driver_state, cx);
            }
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
            let cx = WinitHotReloadContext {
                app,
                services,
                window,
                state,
            };

            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((&mut self.driver_state, cx));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                f(&mut self.driver_state, cx);
            }
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
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((
                    &mut self.driver_state,
                    app,
                    window,
                    state,
                    context,
                    renderer,
                    scale_factor,
                ));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
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
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                return hot.call((
                    &mut self.driver_state,
                    app,
                    window,
                    state,
                    context,
                    renderer,
                    scale_factor,
                    tick_id,
                    frame_id,
                ));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
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
        }
        EngineFrameUpdate::default()
    }

    fn viewport_input(&mut self, app: &mut App, event: ViewportInputEvent) {
        if let Some(f) = self.hooks.viewport_input {
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((&mut self.driver_state, app, event));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                f(&mut self.driver_state, app, event);
            }
        }
    }

    fn dock_op(&mut self, app: &mut App, op: fret_core::DockOp) {
        if let Some(f) = self.hooks.dock_op {
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((&mut self.driver_state, app, op));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                f(&mut self.driver_state, app, op);
            }
        }
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: fret_app::CommandId,
    ) {
        if let Some(f) = self.hooks.handle_command {
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((&mut self.driver_state, context, command));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                f(&mut self.driver_state, context, command);
            }
        }
    }

    fn handle_global_command(
        &mut self,
        context: WinitGlobalContext<'_>,
        command: fret_app::CommandId,
    ) {
        if let Some(f) = self.hooks.handle_global_command {
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((&mut self.driver_state, context, command));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                f(&mut self.driver_state, context, command);
            }
        }
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        if let Some(f) = self.hooks.handle_model_changes {
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((&mut self.driver_state, context, changed));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                f(&mut self.driver_state, context, changed);
            }
        }
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        if let Some(f) = self.hooks.handle_global_changes {
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((&mut self.driver_state, context, changed));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                f(&mut self.driver_state, context, changed);
            }
        }
    }

    fn create_window_state(&mut self, app: &mut App, window: fret_core::AppWindowId) -> S {
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
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                return hot.call((&mut self.driver_state, app, request));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                return f(&mut self.driver_state, app, request);
            }
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
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((&mut self.driver_state, app, request, new_window));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                f(&mut self.driver_state, app, request, new_window);
            }
        }
    }

    fn before_close_window(&mut self, app: &mut App, window: fret_core::AppWindowId) -> bool {
        if let Some(f) = self.hooks.before_close_window {
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                return hot.call((&mut self.driver_state, app, window));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                return f(&mut self.driver_state, app, window);
            }
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
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                return hot.call((&mut self.driver_state, app, window, state));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                return f(&mut self.driver_state, app, window, state);
            }
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
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((&mut self.driver_state, app, window, state, target));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                f(&mut self.driver_state, app, window, state, target);
            }
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
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((&mut self.driver_state, app, services, window, state, target));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                f(&mut self.driver_state, app, services, window, state, target);
            }
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
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((
                    &mut self.driver_state,
                    app,
                    services,
                    window,
                    state,
                    target,
                    value,
                ));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
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
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((
                    &mut self.driver_state,
                    app,
                    services,
                    window,
                    state,
                    target,
                    value,
                ));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
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
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((
                    &mut self.driver_state,
                    app,
                    services,
                    window,
                    state,
                    target,
                    anchor,
                    focus,
                ));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
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
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((
                    &mut self.driver_state,
                    app,
                    services,
                    window,
                    state,
                    target,
                    value,
                ));
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
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
}
