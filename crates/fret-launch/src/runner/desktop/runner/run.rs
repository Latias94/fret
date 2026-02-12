use fret_app::App;
use fret_render::{Renderer, WgpuContext};
use winit::event_loop::{EventLoop, EventLoopBuilder};

use crate::RunnerError;

type OnMainWindowCreatedHook = dyn FnOnce(&mut App, fret_core::AppWindowId) + 'static;
type OnGpuReadyHook = dyn FnOnce(&mut App, &WgpuContext, &mut Renderer) + 'static;
type EventLoopBuilderHook = dyn FnOnce(&mut EventLoopBuilder) + 'static;

pub fn run_app<D: super::WinitAppDriver + 'static>(
    config: super::WinitRunnerConfig,
    app: App,
    driver: D,
) -> Result<(), RunnerError> {
    run_app_with_event_loop(EventLoop::new()?, config, app, driver)
}

pub fn run_app_with_event_loop<D: super::WinitAppDriver + 'static>(
    event_loop: EventLoop,
    config: super::WinitRunnerConfig,
    app: App,
    driver: D,
) -> Result<(), RunnerError> {
    crate::configure_stacksafe_from_env();
    let mut runner = super::WinitRunner::new_app(config, app, driver);
    #[cfg(target_os = "android")]
    runner.set_android_app(event_loop.android_app().clone());
    runner.set_event_loop_proxy(event_loop.create_proxy());
    event_loop.run_app(runner)?;
    Ok(())
}

pub struct WinitAppBuilder<D: super::WinitAppDriver> {
    config: super::WinitRunnerConfig,
    app: App,
    driver: D,
    windows_ime_msg_hook_enabled: bool,
    on_main_window_created: Option<Box<OnMainWindowCreatedHook>>,
    on_gpu_ready: Option<Box<OnGpuReadyHook>>,
    event_loop_builder_hook: Option<Box<EventLoopBuilderHook>>,
    event_loop: Option<EventLoop>,
}

impl<D: super::WinitAppDriver + 'static> WinitAppBuilder<D> {
    pub fn new(app: App, driver: D) -> Self {
        Self {
            config: super::WinitRunnerConfig::default(),
            app,
            driver,
            windows_ime_msg_hook_enabled: cfg!(windows),
            on_main_window_created: None,
            on_gpu_ready: None,
            event_loop_builder_hook: None,
            event_loop: None,
        }
    }

    pub fn configure(mut self, f: impl FnOnce(&mut super::WinitRunnerConfig)) -> Self {
        f(&mut self.config);
        self
    }

    pub fn init_app(mut self, f: impl FnOnce(&mut App)) -> Self {
        f(&mut self.app);
        self
    }

    pub fn on_main_window_created(
        mut self,
        f: impl FnOnce(&mut App, fret_core::AppWindowId) + 'static,
    ) -> Self {
        self.on_main_window_created = Some(Box::new(f));
        self
    }

    pub fn on_gpu_ready(
        mut self,
        f: impl FnOnce(&mut App, &WgpuContext, &mut Renderer) + 'static,
    ) -> Self {
        self.on_gpu_ready = Some(Box::new(f));
        self
    }

    pub fn with_config(mut self, config: super::WinitRunnerConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_event_loop(mut self, event_loop: EventLoop) -> Self {
        self.event_loop = Some(event_loop);
        self
    }

    pub fn with_event_loop_builder_hook(
        mut self,
        hook: impl FnOnce(&mut EventLoopBuilder) + 'static,
    ) -> Self {
        self.event_loop_builder_hook = Some(Box::new(hook));
        self
    }

    pub fn enable_windows_ime_msg_hook(self) -> Self {
        #[cfg(windows)]
        {
            Self {
                windows_ime_msg_hook_enabled: true,
                ..self
            }
        }
        #[cfg(not(windows))]
        self
    }

    pub fn disable_windows_ime_msg_hook(self) -> Self {
        #[cfg(windows)]
        {
            Self {
                windows_ime_msg_hook_enabled: false,
                ..self
            }
        }
        #[cfg(not(windows))]
        self
    }

    pub fn run(self) -> Result<(), RunnerError> {
        let WinitAppBuilder {
            config,
            app,
            driver,
            windows_ime_msg_hook_enabled,
            on_main_window_created,
            on_gpu_ready,
            event_loop_builder_hook,
            event_loop,
        } = self;

        let driver = HookedDriver {
            inner: driver,
            on_main_window_created,
            on_gpu_ready,
        };

        match event_loop {
            Some(event_loop) => run_app_with_event_loop(event_loop, config, app, driver),
            None => {
                let mut builder = EventLoop::builder();
                if let Some(hook) = event_loop_builder_hook {
                    hook(&mut builder);
                }

                #[cfg(windows)]
                {
                    use winit::platform::windows::EventLoopBuilderExtWindows as _;
                    super::event_loop::set_windows_ime_msg_hook_enabled(
                        windows_ime_msg_hook_enabled,
                    );
                    builder.with_msg_hook(super::windows_msg_hook);
                }

                let event_loop = builder.build()?;
                run_app_with_event_loop(event_loop, config, app, driver)
            }
        }
    }
}

struct HookedDriver<D> {
    inner: D,
    on_main_window_created: Option<Box<OnMainWindowCreatedHook>>,
    on_gpu_ready: Option<Box<OnGpuReadyHook>>,
}

impl<D: super::WinitAppDriver> super::WinitAppDriver for HookedDriver<D> {
    type WindowState = D::WindowState;

    fn init(&mut self, app: &mut App, main_window: fret_core::AppWindowId) {
        if let Some(hook) = self.on_main_window_created.take() {
            hook(app, main_window);
        }
        self.inner.init(app, main_window);
    }

    fn gpu_ready(&mut self, app: &mut App, context: &WgpuContext, renderer: &mut Renderer) {
        let diag_renderer_perf =
            std::env::var_os("FRET_DIAG_RENDERER_PERF").is_some_and(|v| !v.is_empty());
        if diag_renderer_perf {
            renderer.set_perf_enabled(true);
        }
        if let Some(hook) = self.on_gpu_ready.take() {
            hook(app, context, renderer);
        }
        self.inner.gpu_ready(app, context, renderer);
    }

    fn hot_reload_global(&mut self, app: &mut App, services: &mut dyn fret_core::UiServices) {
        self.inner.hot_reload_global(app, services);
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
    ) {
        self.inner.hot_reload_window(app, services, window, state);
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
        self.inner
            .gpu_frame_prepare(app, window, state, context, renderer, scale_factor);
    }

    fn record_engine_frame(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        context: &WgpuContext,
        renderer: &mut Renderer,
        scale_factor: f32,
        tick_id: fret_runtime::TickId,
        frame_id: fret_runtime::FrameId,
    ) -> super::EngineFrameUpdate {
        self.inner.record_engine_frame(
            app,
            window,
            state,
            context,
            renderer,
            scale_factor,
            tick_id,
            frame_id,
        )
    }

    fn record_engine_commands(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        context: &WgpuContext,
        renderer: &mut Renderer,
        scale_factor: f32,
        tick_id: fret_runtime::TickId,
        frame_id: fret_runtime::FrameId,
    ) -> Vec<wgpu::CommandBuffer> {
        self.inner.record_engine_commands(
            app,
            window,
            state,
            context,
            renderer,
            scale_factor,
            tick_id,
            frame_id,
        )
    }

    fn viewport_input(&mut self, app: &mut App, event: fret_core::ViewportInputEvent) {
        self.inner.viewport_input(app, event);
    }

    fn dock_op(&mut self, app: &mut App, op: fret_core::DockOp) {
        self.inner.dock_op(app, op);
    }

    fn handle_command(
        &mut self,
        context: super::WinitCommandContext<'_, Self::WindowState>,
        command: fret_app::CommandId,
    ) {
        self.inner.handle_command(context, command);
    }

    fn handle_global_command(
        &mut self,
        context: super::WinitGlobalContext<'_>,
        command: fret_app::CommandId,
    ) {
        self.inner.handle_global_command(context, command);
    }

    fn handle_model_changes(
        &mut self,
        context: super::WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        self.inner.handle_model_changes(context, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: super::WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        self.inner.handle_global_changes(context, changed);
    }

    fn create_window_state(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
    ) -> Self::WindowState {
        self.inner.create_window_state(app, window)
    }

    fn handle_event(
        &mut self,
        context: super::WinitEventContext<'_, Self::WindowState>,
        event: &fret_core::Event,
    ) {
        self.inner.handle_event(context, event);
    }

    fn render(&mut self, context: super::WinitRenderContext<'_, Self::WindowState>) {
        self.inner.render(context);
    }

    fn window_create_spec(
        &mut self,
        app: &mut App,
        request: &fret_app::CreateWindowRequest,
    ) -> Option<super::WindowCreateSpec> {
        self.inner.window_create_spec(app, request)
    }

    fn window_created(
        &mut self,
        app: &mut App,
        request: &fret_app::CreateWindowRequest,
        new_window: fret_core::AppWindowId,
    ) {
        self.inner.window_created(app, request, new_window);
    }

    fn before_close_window(&mut self, app: &mut App, window: fret_core::AppWindowId) -> bool {
        self.inner.before_close_window(app, window)
    }

    fn accessibility_snapshot(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
    ) -> Option<std::sync::Arc<fret_core::SemanticsSnapshot>> {
        self.inner.accessibility_snapshot(app, window, state)
    }

    fn accessibility_focus(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        self.inner.accessibility_focus(app, window, state, target);
    }

    fn accessibility_invoke(
        &mut self,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        self.inner
            .accessibility_invoke(app, services, window, state, target);
    }

    fn accessibility_set_value_text(
        &mut self,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        self.inner
            .accessibility_set_value_text(app, services, window, state, target, value);
    }

    fn accessibility_set_value_numeric(
        &mut self,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: f64,
    ) {
        self.inner
            .accessibility_set_value_numeric(app, services, window, state, target, value);
    }

    fn accessibility_set_text_selection(
        &mut self,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        anchor: u32,
        focus: u32,
    ) {
        self.inner
            .accessibility_set_text_selection(app, services, window, state, target, anchor, focus);
    }

    fn accessibility_replace_selected_text(
        &mut self,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        self.inner
            .accessibility_replace_selected_text(app, services, window, state, target, value);
    }
}
