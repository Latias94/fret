use super::*;
use fret_app::App;
use fret_render::{Renderer, WgpuContext};
use winit::event_loop::{EventLoop, EventLoopBuilder};
#[cfg(target_os = "android")]
use winit::platform::android::EventLoopExtAndroid as _;

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

impl<D: WinitAppDriver> WinitRunner<D> {
    pub fn new(config: WinitRunnerConfig, app: App, driver: D) -> Self {
        let mut app = app;
        let now = Instant::now();
        let startup_incoming_open_paths = read_startup_incoming_open_paths_from_args();
        let requested = match app.global::<PlatformCapabilities>().cloned() {
            Some(caps) => caps,
            None => {
                let caps = PlatformCapabilities::default();
                app.set_global(caps.clone());
                caps
            }
        };
        let caps = Self::effective_platform_capabilities(&config, &requested);
        if caps != requested {
            app.set_global(caps.clone());
        }
        tracing::info!(caps = ?caps, "platform capabilities");

        let dispatcher = DesktopDispatcher::new(caps.exec);
        app.set_global::<fret_runtime::DispatcherHandle>(dispatcher.handle());

        let mut runner = Self {
            config,
            app,
            driver,
            dispatcher,
            event_loop_proxy: None,
            proxy_events: Arc::new(Mutex::new(Vec::new())),
            is_suspended: false,
            driver_initialized: false,
            wgpu_init_blocked: false,
            #[cfg(target_os = "android")]
            android_app: None,
            renderdoc: None,
            context: None,
            renderer: None,
            renderer_caps: None,
            system_font_rescan_result: Arc::new(Mutex::new(None)),
            system_font_rescan_in_flight: false,
            system_font_rescan_pending: false,
            last_window_surface_sizes: HashMap::new(),
            last_window_surface_size_changed_at: None,
            no_services: NoUiServices,
            diag_bundle_screenshots: DiagBundleScreenshotCapture::from_env(),
            windows: SlotMap::with_key(),
            window_registry: fret_runner_winit::window_registry::WinitWindowRegistry::default(),
            main_window: None,
            menu_bar: None,
            windows_pending_front: HashMap::new(),
            windows_z_order: Vec::new(),
            saw_left_mouse_release_this_turn: false,
            left_mouse_down: false,
            dock_tearoff_follow: None,
            dock_floating_windows: HashSet::new(),
            dock_drag_pointer_capture: None,
            tick_id: TickId::default(),
            frame_id: FrameId::default(),
            next_environment_poll_at: now,
            #[cfg(target_os = "linux")]
            linux_portal_settings_listener_started: false,
            raf_windows: HashSet::new(),
            timers: HashMap::new(),
            clipboard: NativeClipboard::default(),
            diag_clipboard_force_unavailable_windows: HashSet::new(),
            open_url: NativeOpenUrl,
            file_dialog: NativeFileDialog::default(),
            diag_incoming_open_next_token: 1,
            diag_incoming_open_payloads: HashMap::new(),
            startup_incoming_open_paths,
            startup_incoming_open_delivered: false,
            incoming_open_path_payloads: HashMap::new(),
            #[cfg(target_os = "ios")]
            ios_keyboard: None,
            diag_window_insets_overrides: HashMap::new(),
            diag_cursor_screen_pos_override:
                super::diag_cursor_override::DiagCursorScreenPosOverride::from_env(),
            diag_last_cursor_override_tick: None,
            diag_mouse_buttons_override:
                super::diag_mouse_buttons_override::DiagMouseButtonsOverride::from_env(),
            diag_last_mouse_buttons_override_tick: None,
            diag_mouse_buttons_override_active: false,
            diag_isolate_pointer_input: std::env::var_os("FRET_DIAG_ISOLATE_POINTER_INPUT")
                .is_some_and(|v| {
                    let raw = v.to_string_lossy();
                    let raw = raw.trim();
                    !raw.is_empty() && raw != "0" && !raw.eq_ignore_ascii_case("false")
                }),
            cursor_screen_pos: None,
            #[cfg(target_os = "macos")]
            macos_cursor_transform: MacCursorTransformTable::default(),
            internal_drag_hover_window: None,
            internal_drag_hover_pos: None,
            internal_drag_pointer_id: None,
            external_drop: NativeExternalDrop::default(),
            uploaded_images: HashMap::new(),
            streaming_uploads: StreamingUploadQueue::default(),
            nv12_gpu: None,
            #[cfg(feature = "dev-state")]
            dev_state: super::dev_state::DevStateController::from_env(now),
            #[cfg(feature = "hotpatch-subsecond")]
            hotpatch: hotpatch_trigger_from_env(now),
            #[cfg(feature = "hotpatch-subsecond")]
            hot_reload_generation: 0,
            watch_restart_trigger: super::restart_trigger::RestartTrigger::from_env(now),
            watch_restart_requested: false,

            #[cfg(feature = "diag-screenshots")]
            diag_screenshots: diag_screenshots::DiagScreenshotCapture::from_env(),
        };
        #[cfg(feature = "dev-state")]
        runner.dev_state.install_into_app(&mut runner.app);
        #[cfg(feature = "dev-state")]
        crate::dev_state::DevStateHooks::import_all(&mut runner.app);
        runner.publish_system_font_rescan_state();
        runner
    }
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
            windows_ime_msg_hook_enabled: _windows_ime_msg_hook_enabled,
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
                        _windows_ime_msg_hook_enabled,
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

    fn accessibility_decrement(
        &mut self,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        self.inner
            .accessibility_decrement(app, services, window, state, target);
    }

    fn accessibility_increment(
        &mut self,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        self.inner
            .accessibility_increment(app, services, window, state, target);
    }

    fn accessibility_scroll_by(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        dx: f64,
        dy: f64,
    ) {
        self.inner
            .accessibility_scroll_by(app, window, state, target, dx, dy);
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
