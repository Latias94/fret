use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    fmt,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use fret_app::{App, CreateWindowKind, CreateWindowRequest, Effect, WindowRequest};
use fret_core::{
    Event, ExternalDragEvent, ExternalDragFile, ExternalDragFiles, ExternalDragKind,
    InternalDragEvent, InternalDragKind, Point, Px, Rect, Scene, Size, UiServices,
    ViewportInputEvent, WindowMetricsService,
};
use fret_platform_native::clipboard::NativeClipboard;
use fret_platform_native::external_drop::NativeExternalDrop;
use fret_platform_native::file_dialog::NativeFileDialog;
use fret_platform_native::open_url::NativeOpenUrl;
use fret_render::{ClearColor, Renderer, SurfaceState, WgpuContext};
use fret_runner_winit::accessibility;
use fret_runtime::{
    ExternalDragPayloadKind, ExternalDragPositionQuality, FrameId, PlatformCapabilities,
    PlatformCompletion, TickId,
};
use slotmap::SlotMap;
use tracing::error;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, Position},
    event::{DeviceEvent, ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{Window, WindowId, WindowLevel},
};

use crate::error::RunnerError;
use fret_platform::clipboard::Clipboard as _;
use fret_platform::external_drop::ExternalDropProvider as _;
use fret_platform::file_dialog::FileDialogProvider as _;
use fret_platform::open_url::OpenUrl as _;

type WindowAnchor = fret_core::WindowAnchor;

mod app_handler;
mod no_services;

use no_services::NoUiServices;

#[cfg(windows)]
pub fn ime_msg_hook(msg: *const std::ffi::c_void) -> bool {
    fret_runner_winit::windows_ime::msg_hook(msg)
}

#[derive(Debug, Clone)]
pub enum RunnerUserEvent {
    PlatformCompletion {
        window: fret_core::AppWindowId,
        completion: PlatformCompletion,
    },
}

pub fn run_app<D: WinitAppDriver + 'static>(
    config: WinitRunnerConfig,
    app: App,
    driver: D,
) -> Result<(), RunnerError> {
    run_app_with_event_loop(EventLoop::new()?, config, app, driver)
}

pub fn run_app_with_event_loop<D: WinitAppDriver + 'static>(
    event_loop: EventLoop,
    config: WinitRunnerConfig,
    app: App,
    driver: D,
) -> Result<(), RunnerError> {
    let mut runner = WinitRunner::new_app(config, app, driver);
    runner.set_event_loop_proxy(event_loop.create_proxy());
    event_loop.run_app(runner)?;
    Ok(())
}

type OnMainWindowCreatedHook = dyn FnOnce(&mut App, fret_core::AppWindowId) + 'static;
type OnGpuReadyHook = dyn FnOnce(&mut App, &WgpuContext, &mut Renderer) + 'static;
type EventLoopBuilderHook = dyn FnOnce(&mut EventLoopBuilder) + 'static;

pub struct WinitAppBuilder<D: WinitAppDriver> {
    config: WinitRunnerConfig,
    app: App,
    driver: D,
    windows_ime_msg_hook_enabled: bool,
    on_main_window_created: Option<Box<OnMainWindowCreatedHook>>,
    on_gpu_ready: Option<Box<OnGpuReadyHook>>,
    event_loop_builder_hook: Option<Box<EventLoopBuilderHook>>,
    event_loop: Option<EventLoop>,
}

impl<D: WinitAppDriver + 'static> WinitAppBuilder<D> {
    pub fn new(app: App, driver: D) -> Self {
        Self {
            config: WinitRunnerConfig::default(),
            app,
            driver,
            windows_ime_msg_hook_enabled: cfg!(windows),
            on_main_window_created: None,
            on_gpu_ready: None,
            event_loop_builder_hook: None,
            event_loop: None,
        }
    }

    pub fn configure(mut self, f: impl FnOnce(&mut WinitRunnerConfig)) -> Self {
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

    pub fn with_config(mut self, config: WinitRunnerConfig) -> Self {
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
                    if windows_ime_msg_hook_enabled {
                        use winit::platform::windows::EventLoopBuilderExtWindows as _;
                        builder.with_msg_hook(ime_msg_hook);
                    }
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

impl<D: WinitAppDriver> WinitAppDriver for HookedDriver<D> {
    type WindowState = D::WindowState;

    fn init(&mut self, app: &mut App, main_window: fret_core::AppWindowId) {
        if let Some(hook) = self.on_main_window_created.take() {
            hook(app, main_window);
        }
        self.inner.init(app, main_window);
    }

    fn gpu_ready(&mut self, app: &mut App, context: &WgpuContext, renderer: &mut Renderer) {
        if let Some(hook) = self.on_gpu_ready.take() {
            hook(app, context, renderer);
        }
        self.inner.gpu_ready(app, context, renderer);
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
        tick_id: TickId,
        frame_id: FrameId,
    ) -> EngineFrameUpdate {
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
        tick_id: TickId,
        frame_id: FrameId,
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

    fn viewport_input(&mut self, app: &mut App, event: ViewportInputEvent) {
        self.inner.viewport_input(app, event);
    }

    fn dock_op(&mut self, app: &mut App, op: fret_core::DockOp) {
        self.inner.dock_op(app, op);
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: fret_app::CommandId,
    ) {
        self.inner.handle_command(context, command);
    }

    fn handle_global_command(
        &mut self,
        context: WinitGlobalContext<'_>,
        command: fret_app::CommandId,
    ) {
        self.inner.handle_global_command(context, command);
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        self.inner.handle_model_changes(context, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[TypeId],
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

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        self.inner.handle_event(context, event);
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        self.inner.render(context);
    }

    fn window_create_spec(
        &mut self,
        app: &mut App,
        request: &CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        self.inner.window_create_spec(app, request)
    }

    fn window_created(
        &mut self,
        app: &mut App,
        request: &CreateWindowRequest,
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
        services: &mut dyn UiServices,
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
        services: &mut dyn UiServices,
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
        services: &mut dyn UiServices,
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
        services: &mut dyn UiServices,
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
        services: &mut dyn UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        self.inner
            .accessibility_replace_selected_text(app, services, window, state, target, value);
    }
}

fn validate_scene_if_enabled(scene: &Scene) {
    if std::env::var_os("FRET_VALIDATE_SCENE").is_none() {
        return;
    }

    if let Err(err) = scene.validate() {
        error!(
            index = err.index,
            op = ?err.op,
            kind = ?err.kind,
            error = %err,
            "scene validation failed (set FRET_VALIDATE_SCENE_PANIC=1 to panic)"
        );

        if std::env::var_os("FRET_VALIDATE_SCENE_PANIC").is_some() {
            panic!("scene validation failed: {err}");
        }
    }
}

#[cfg(target_os = "windows")]
mod win32 {
    use super::MonitorRectF64;
    use winit::dpi::PhysicalPosition;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct MonitorInfo {
        cb_size: u32,
        rc_monitor: Rect,
        rc_work: Rect,
        dw_flags: u32,
    }

    const MONITOR_DEFAULTTONEAREST: u32 = 2;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetCursorPos(lpPoint: *mut Point) -> i32;
        fn MonitorFromPoint(pt: Point, dwFlags: u32) -> isize;
        fn GetMonitorInfoW(hMonitor: isize, lpmi: *mut MonitorInfo) -> i32;
    }

    pub fn cursor_pos_physical() -> Option<PhysicalPosition<f64>> {
        let mut p = Point::default();
        let ok = unsafe { GetCursorPos(&mut p) };
        if ok == 0 {
            return None;
        }
        Some(PhysicalPosition::new(p.x as f64, p.y as f64))
    }

    pub fn monitor_work_area_for_point(point: PhysicalPosition<f64>) -> Option<MonitorRectF64> {
        let pt = Point {
            x: point.x.round() as i32,
            y: point.y.round() as i32,
        };
        let hmon = unsafe { MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST) };
        if hmon == 0 {
            return None;
        }

        let mut info = MonitorInfo {
            cb_size: std::mem::size_of::<MonitorInfo>() as u32,
            ..Default::default()
        };
        let ok = unsafe { GetMonitorInfoW(hmon, &mut info) };
        if ok == 0 {
            return None;
        }

        Some(MonitorRectF64 {
            min_x: info.rc_work.left as f64,
            min_y: info.rc_work.top as f64,
            max_x: info.rc_work.right as f64,
            max_y: info.rc_work.bottom as f64,
        })
    }
}

fn macos_window_log(_args: fmt::Arguments<'_>) {
    #[cfg(target_os = "macos")]
    {
        use std::{
            io::Write,
            sync::{Mutex, OnceLock},
        };

        if std::env::var_os("FRET_MACOS_WINDOW_LOG").is_none() {
            return;
        }

        static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

        let file = LOG_FILE.get_or_init(|| {
            let _ = std::fs::create_dir_all("target");
            let path = std::path::Path::new("target").join("fret-macos-window.log");
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
                .expect("open fret-macos-window.log");
            let _ = writeln!(
                file,
                "[session] pid={} time={:?}",
                std::process::id(),
                std::time::SystemTime::now()
            );
            Mutex::new(file)
        });

        let Ok(mut file) = file.lock() else {
            return;
        };

        let _ = writeln!(file, "{}", _args);
    }
}

fn dock_tearoff_log(_args: fmt::Arguments<'_>) {
    #[cfg(target_os = "macos")]
    {
        use std::{
            io::Write,
            sync::{Mutex, OnceLock},
        };

        if std::env::var_os("FRET_DOCK_TEAROFF_LOG").is_none() {
            return;
        }

        static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

        let file = LOG_FILE.get_or_init(|| {
            let _ = std::fs::create_dir_all("target");
            let path = std::path::Path::new("target").join("fret-dock-tearoff.log");
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
                .expect("open fret-dock-tearoff.log");
            let _ = writeln!(
                file,
                "[session] pid={} time={:?}",
                std::process::id(),
                std::time::SystemTime::now()
            );
            Mutex::new(file)
        });

        let Ok(mut file) = file.lock() else {
            return;
        };

        let _ = writeln!(file, "{}", _args);
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn macos_is_left_mouse_down() -> bool {
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};
    unsafe {
        let Some(class) = Class::get("NSEvent") else {
            return false;
        };
        let buttons: u64 = msg_send![class, pressedMouseButtons];
        (buttons & 1) != 0
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn bring_window_to_front(window: &Window, sender: Option<&Window>) -> bool {
    use cocoa::{
        appkit::{NSApp, NSApplication, NSWindow},
        base::{id, nil},
    };
    use objc::runtime::YES;
    use objc::{msg_send, sel, sel_impl};
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    unsafe fn ns_window_id(window: &Window) -> Option<id> {
        let handle = window.window_handle().ok()?;
        let RawWindowHandle::AppKit(h) = handle.as_raw() else {
            return None;
        };
        let ns_view: id = h.ns_view.as_ptr() as id;
        if ns_view == nil {
            return None;
        }
        let ns_window: id = msg_send![ns_view, window];
        (ns_window != nil).then_some(ns_window)
    }

    unsafe {
        // If the window was created hidden (we do this for DockFloating to avoid the initial flash
        // behind the source window), ensure it is visible before we query/raise the NSWindow.
        // Otherwise `ns_view.window` can be nil and the raise attempt becomes a no-op.
        window.set_visible(true);

        // macOS often keeps newly created windows behind the current key window unless we
        // explicitly activate the app and order the window front.
        let app = NSApp();
        app.activateIgnoringOtherApps_(YES);

        // winit exposes an `NSView*` via raw-window-handle; resolve `NSWindow*` from it.
        let Some(ns_window) = ns_window_id(window) else {
            macos_window_log(format_args!(
                "[raise] ns_window=nil winit={:?}",
                window.id()
            ));
            return false;
        };

        // Passing a sender can help macOS accept the focus change while the source window is
        // still finishing an interaction (ImGui does this in its macOS backend).
        let sender_window = sender.and_then(|w| ns_window_id(w)).unwrap_or(nil);

        let sender_level: i64 = if sender_window != nil {
            msg_send![sender_window, level]
        } else {
            -1
        };
        let sender_number: i32 = if sender_window != nil {
            msg_send![sender_window, windowNumber]
        } else {
            0
        };
        let sender_ordered_index: i32 = if sender_window != nil {
            msg_send![sender_window, orderedIndex]
        } else {
            -1
        };
        let sender_occlusion: u64 = if sender_window != nil {
            msg_send![sender_window, occlusionState]
        } else {
            0
        };

        let key_window: id = msg_send![app, keyWindow];
        let main_window: id = msg_send![app, mainWindow];
        let is_key: bool = msg_send![ns_window, isKeyWindow];
        let is_main: bool = msg_send![ns_window, isMainWindow];
        let is_visible: bool = msg_send![ns_window, isVisible];
        let occlusion: u64 = msg_send![ns_window, occlusionState];
        let level: i64 = msg_send![ns_window, level];
        let ordered_index: i32 = msg_send![ns_window, orderedIndex];
        let window_number: i32 = msg_send![ns_window, windowNumber];
        macos_window_log(format_args!(
            "[raise-before] target={:p} sender={:p} sender_level={} sender_num={} sender_ordered_index={} sender_occl=0x{:x} key={:p} main={:p} is_key={} is_main={} visible={} occl=0x{:x} level={} ordered_index={} win_num={} winit={:?}",
            ns_window as *const std::ffi::c_void,
            sender_window as *const std::ffi::c_void,
            sender_level,
            sender_number,
            sender_ordered_index,
            sender_occlusion,
            key_window as *const std::ffi::c_void,
            main_window as *const std::ffi::c_void,
            is_key,
            is_main,
            is_visible,
            occlusion,
            level,
            ordered_index,
            window_number,
            window.id(),
        ));

        ns_window.makeKeyAndOrderFront_(sender_window);
        if sender_window != nil {
            // Ensure we are ordered above the source window even if macOS keeps the relative
            // ordering unchanged (e.g. due to window levels or active interactions).
            // NSWindowAbove = 1
            let _: () = msg_send![ns_window, orderWindow: 1 relativeTo: sender_number];
        }
        let _: () = msg_send![ns_window, orderFrontRegardless];

        // Keep winit’s internal focus bookkeeping aligned; in practice this also improves the
        // success rate of the ordering change when the source window is in a tracked interaction.
        window.focus_window();

        let key_window_after: id = msg_send![app, keyWindow];
        let main_window_after: id = msg_send![app, mainWindow];
        let is_key_after: bool = msg_send![ns_window, isKeyWindow];
        let is_main_after: bool = msg_send![ns_window, isMainWindow];
        let is_visible_after: bool = msg_send![ns_window, isVisible];
        let occlusion_after: u64 = msg_send![ns_window, occlusionState];
        let level_after: i64 = msg_send![ns_window, level];
        let ordered_index_after: i32 = msg_send![ns_window, orderedIndex];
        let window_number_after: i32 = msg_send![ns_window, windowNumber];
        macos_window_log(format_args!(
            "[raise-after]  target={:p} sender={:p} sender_level={} sender_num={} sender_ordered_index={} sender_occl=0x{:x} key={:p} main={:p} is_key={} is_main={} visible={} occl=0x{:x} level={} ordered_index={} win_num={} winit={:?}",
            ns_window as *const std::ffi::c_void,
            sender_window as *const std::ffi::c_void,
            sender_level,
            sender_number,
            sender_ordered_index,
            sender_occlusion,
            key_window_after as *const std::ffi::c_void,
            main_window_after as *const std::ffi::c_void,
            is_key_after,
            is_main_after,
            is_visible_after,
            occlusion_after,
            level_after,
            ordered_index_after,
            window_number_after,
            window.id(),
        ));
        true
    }
}

#[cfg(not(target_os = "macos"))]
fn bring_window_to_front(window: &dyn Window, _sender: Option<&dyn Window>) -> bool {
    window.focus_window();
    true
}

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
}

pub enum WgpuInit {
    /// Create a `WgpuContext` internally using a surface-compatible adapter.
    CreateDefault,
    /// Use a host-provided GPU context. The runner will create surfaces via `context.instance`
    /// and assumes the adapter/device are compatible with those surfaces.
    Provided(WgpuContext),
    /// Create the GPU context via a host callback given the main window.
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
        }
    }
}

impl WinitRunnerConfig {
    fn main_window_spec(&self) -> WindowCreateSpec {
        let mut spec = WindowCreateSpec::new(self.main_window_title.clone(), self.main_window_size);
        if let Some(position) = self.main_window_position {
            spec = spec.with_position(position);
        }
        spec
    }

    fn default_window_spec(&self) -> WindowCreateSpec {
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
        _changed: &[TypeId],
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
        _request: &CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &CreateWindowRequest,
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

pub struct WinitAppDriverAdapter<D> {
    inner: D,
}

impl<D> WinitAppDriverAdapter<D> {
    pub fn new(inner: D) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> D {
        self.inner
    }

    pub fn inner(&self) -> &D {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }
}

pub trait WinitDriver {
    type WindowState;

    fn init(&mut self, _app: &mut App, _main_window: fret_core::AppWindowId) {}

    fn gpu_ready(&mut self, _app: &mut App, _context: &WgpuContext, _renderer: &mut Renderer) {}

    #[allow(clippy::too_many_arguments)]
    /// Prepare GPU resources needed for the upcoming UI render.
    ///
    /// This runs on the render thread right before `render(...)`, and exists to support workflows
    /// like SVG rasterization + texture registration that require `Device/Queue/Renderer` access.
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
        _app: &mut App,
        _services: &mut dyn UiServices,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
        _command: fret_app::CommandId,
    ) {
    }

    fn handle_global_command(
        &mut self,
        _app: &mut App,
        _services: &mut dyn UiServices,
        _command: fret_app::CommandId,
    ) {
    }

    fn handle_model_changes(
        &mut self,
        _app: &mut App,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
        _changed: &[fret_app::ModelId],
    ) {
    }

    fn handle_global_changes(
        &mut self,
        _app: &mut App,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
        _changed: &[TypeId],
    ) {
    }

    fn create_window_state(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
    ) -> Self::WindowState;

    fn handle_event(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        event: &Event,
    );

    #[allow(clippy::too_many_arguments)]
    /// Render the UI into `scene`.
    ///
    /// Notes:
    /// - `scene` is **not cleared by the runner**; drivers should clear it before recording
    ///   the current frame.
    /// - This allows drivers to ingest the previous frame's recorded ops for replay caching.
    fn render(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        bounds: Rect,
        scale_factor: f32,
        services: &mut dyn fret_core::UiServices,
        scene: &mut Scene,
    );

    fn window_create_spec(
        &mut self,
        app: &mut App,
        request: &CreateWindowRequest,
    ) -> Option<WindowCreateSpec>;

    fn window_created(
        &mut self,
        app: &mut App,
        request: &CreateWindowRequest,
        new_window: fret_core::AppWindowId,
    );

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

struct WindowRuntime<S> {
    window: Arc<dyn Window>,
    accessibility: Option<accessibility::WinitAccessibility>,
    last_accessibility_snapshot: Option<std::sync::Arc<fret_core::SemanticsSnapshot>>,
    surface: SurfaceState<'static>,
    scene: Scene,
    platform: fret_runner_winit::WinitPlatform,
    is_focused: bool,
    external_drag_files: Vec<std::path::PathBuf>,
    external_drag_token: Option<fret_runtime::ExternalDropToken>,
    user: S,
}

#[derive(Debug, Clone)]
struct PendingFrontRequest {
    source_window: Option<fret_core::AppWindowId>,
    panel: Option<fret_core::PanelKey>,
    created_at: Instant,
    next_attempt_at: Instant,
    attempts_left: u8,
}

pub struct WinitRunner<D: WinitDriver> {
    pub config: WinitRunnerConfig,
    pub app: App,
    pub driver: D,
    event_loop_proxy: Option<EventLoopProxy>,
    proxy_events: Arc<Mutex<Vec<RunnerUserEvent>>>,

    context: Option<WgpuContext>,
    renderer: Option<Renderer>,
    no_services: NoUiServices,

    windows: SlotMap<fret_core::AppWindowId, WindowRuntime<D::WindowState>>,
    window_registry: fret_runner_winit::window_registry::WinitWindowRegistry,
    main_window: Option<fret_core::AppWindowId>,
    windows_pending_front: HashMap<fret_core::AppWindowId, PendingFrontRequest>,

    /// True if this event-loop turn already observed a left mouse release via `WindowEvent`.
    /// On macOS we may also see the same release as a `DeviceEvent`, so this prevents double-drop.
    saw_left_mouse_release_this_turn: bool,
    left_mouse_down: bool,
    dock_tearoff_follow: Option<DockTearoffFollow>,

    tick_id: TickId,
    frame_id: FrameId,

    raf_windows: HashSet<fret_core::AppWindowId>,
    timers: HashMap<fret_runtime::TimerToken, TimerEntry>,
    clipboard: NativeClipboard,
    open_url: NativeOpenUrl,
    file_dialog: NativeFileDialog,
    cursor_screen_pos: Option<PhysicalPosition<f64>>,
    internal_drag_hover_window: Option<fret_core::AppWindowId>,
    internal_drag_hover_pos: Option<Point>,

    external_drop: NativeExternalDrop,
}

#[derive(Debug, Clone)]
struct TimerEntry {
    window: Option<fret_core::AppWindowId>,
    deadline: Instant,
    repeat: Option<Duration>,
}

#[derive(Debug, Clone, Copy)]
struct DockTearoffFollow {
    window: fret_core::AppWindowId,
    source_window: fret_core::AppWindowId,
    grab_offset: Point,
    manual_follow: bool,
    last_outer_pos: Option<PhysicalPosition<i32>>,
}

#[derive(Clone, Copy, Debug)]
struct MonitorRectF64 {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

#[derive(Clone, Copy, Debug)]
struct RectF64 {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl<D: WinitDriver> WinitRunner<D> {
    const WINDOW_VISIBILITY_PADDING_PX: f64 = 40.0;

    fn virtual_desktop_bounds(window: &dyn Window) -> Option<MonitorRectF64> {
        let mut monitors = window.available_monitors();
        let first = monitors.next()?;

        let first_pos = first.position()?;
        let first_size = first.current_video_mode()?.size();
        let mut min_x = first_pos.x as f64;
        let mut min_y = first_pos.y as f64;
        let mut max_x = first_pos.x as f64 + first_size.width as f64;
        let mut max_y = first_pos.y as f64 + first_size.height as f64;

        for monitor in monitors {
            let Some(pos) = monitor.position() else {
                continue;
            };
            let Some(mode) = monitor.current_video_mode() else {
                continue;
            };
            let size = mode.size();
            min_x = min_x.min(pos.x as f64);
            min_y = min_y.min(pos.y as f64);
            max_x = max_x.max(pos.x as f64 + size.width as f64);
            max_y = max_y.max(pos.y as f64 + size.height as f64);
        }

        Some(MonitorRectF64 {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    fn monitor_rects_physical(window: &dyn Window) -> Vec<MonitorRectF64> {
        window
            .available_monitors()
            .filter_map(|m| {
                let pos = m.position()?;
                let size = m.current_video_mode()?.size();
                Some(MonitorRectF64 {
                    min_x: pos.x as f64,
                    min_y: pos.y as f64,
                    max_x: pos.x as f64 + size.width as f64,
                    max_y: pos.y as f64 + size.height as f64,
                })
            })
            .collect()
    }

    fn find_monitor_for_point(
        monitors: &[MonitorRectF64],
        point: PhysicalPosition<f64>,
    ) -> Option<usize> {
        if monitors.is_empty() {
            return None;
        }

        let mut best = 0usize;
        let mut best_dist2 = f64::INFINITY;
        for (i, m) in monitors.iter().enumerate() {
            let dx = if point.x < m.min_x {
                m.min_x - point.x
            } else if point.x > m.max_x {
                point.x - m.max_x
            } else {
                0.0
            };
            let dy = if point.y < m.min_y {
                m.min_y - point.y
            } else if point.y > m.max_y {
                point.y - m.max_y
            } else {
                0.0
            };
            let dist2 = dx * dx + dy * dy;
            if dist2 < best_dist2 {
                best_dist2 = dist2;
                best = i;
            }
            if dist2 == 0.0 {
                return Some(i);
            }
        }

        Some(best)
    }

    fn find_monitor_for_rect(monitors: &[MonitorRectF64], rect: RectF64) -> Option<usize> {
        if monitors.is_empty() {
            return None;
        }
        if monitors.len() == 1 {
            return Some(0);
        }

        let mut best = 0usize;
        let mut best_area = -1.0f64;
        for (i, m) in monitors.iter().enumerate() {
            let ix0 = rect.min_x.max(m.min_x);
            let iy0 = rect.min_y.max(m.min_y);
            let ix1 = rect.max_x.min(m.max_x);
            let iy1 = rect.max_y.min(m.max_y);
            let iw = (ix1 - ix0).max(0.0);
            let ih = (iy1 - iy0).max(0.0);
            let area = iw * ih;
            if area > best_area {
                best_area = area;
                best = i;
            }
        }
        Some(best)
    }

    fn clamp_window_outer_pos_to_monitor(
        desired_outer_x: f64,
        desired_outer_y: f64,
        outer_size: winit::dpi::PhysicalSize<u32>,
        monitor: MonitorRectF64,
        padding: f64,
    ) -> (f64, f64) {
        let w = outer_size.width as f64;
        let h = outer_size.height as f64;

        let pad_x = padding.min(w).max(0.0);
        let pad_y = padding.min(h).max(0.0);

        // Keep at least `pad` pixels of the window visible within the monitor bounds.
        let min_x = monitor.min_x - (w - pad_x);
        let max_x = monitor.max_x - pad_x;
        let min_y = monitor.min_y - (h - pad_y);
        let max_y = monitor.max_y - pad_y;

        let clamped_x = desired_outer_x.clamp(min_x, max_x.max(min_x));
        let clamped_y = desired_outer_y.clamp(min_y, max_y.max(min_y));
        (clamped_x, clamped_y)
    }

    fn settle_window_outer_position(
        &self,
        window: &dyn Window,
        cursor_screen_pos: Option<PhysicalPosition<f64>>,
    ) -> Option<PhysicalPosition<i32>> {
        let outer_pos = window.outer_position().ok()?;
        let outer_size = window.outer_size();

        let desired_x = outer_pos.x as f64;
        let desired_y = outer_pos.y as f64;

        #[cfg(target_os = "windows")]
        if let Some(cursor) = cursor_screen_pos
            && let Some(work) = win32::monitor_work_area_for_point(cursor)
        {
            let (x, y) = Self::clamp_window_outer_pos_to_monitor(
                desired_x,
                desired_y,
                outer_size,
                work,
                Self::WINDOW_VISIBILITY_PADDING_PX,
            );
            let target = PhysicalPosition::new(x.round() as i32, y.round() as i32);
            return (target != outer_pos).then_some(target);
        }

        let monitors = Self::monitor_rects_physical(window);
        let monitor = if let Some(cursor) = cursor_screen_pos
            && let Some(idx) = Self::find_monitor_for_point(&monitors, cursor)
            && let Some(m) = monitors.get(idx).copied()
        {
            Some(m)
        } else {
            let rect = RectF64 {
                min_x: desired_x,
                min_y: desired_y,
                max_x: desired_x + outer_size.width as f64,
                max_y: desired_y + outer_size.height as f64,
            };
            let idx = Self::find_monitor_for_rect(&monitors, rect);
            idx.and_then(|i| monitors.get(i).copied())
        };

        let monitor = monitor.or_else(|| Self::virtual_desktop_bounds(window));
        let monitor = monitor?;

        let (x, y) = Self::clamp_window_outer_pos_to_monitor(
            desired_x,
            desired_y,
            outer_size,
            monitor,
            Self::WINDOW_VISIBILITY_PADDING_PX,
        );

        let target = PhysicalPosition::new(x.round() as i32, y.round() as i32);
        if target == outer_pos {
            None
        } else {
            Some(target)
        }
    }

    pub fn new(config: WinitRunnerConfig, app: App, driver: D) -> Self {
        let mut app = app;
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

        Self {
            config,
            app,
            driver,
            event_loop_proxy: None,
            proxy_events: Arc::new(Mutex::new(Vec::new())),
            context: None,
            renderer: None,
            no_services: NoUiServices,
            windows: SlotMap::with_key(),
            window_registry: fret_runner_winit::window_registry::WinitWindowRegistry::default(),
            main_window: None,
            windows_pending_front: HashMap::new(),
            saw_left_mouse_release_this_turn: false,
            left_mouse_down: false,
            dock_tearoff_follow: None,
            tick_id: TickId::default(),
            frame_id: FrameId::default(),
            raf_windows: HashSet::new(),
            timers: HashMap::new(),
            clipboard: NativeClipboard::default(),
            open_url: NativeOpenUrl,
            file_dialog: NativeFileDialog::default(),
            cursor_screen_pos: None,
            internal_drag_hover_window: None,
            internal_drag_hover_pos: None,
            external_drop: NativeExternalDrop::default(),
        }
    }

    fn backend_platform_capabilities(_config: &WinitRunnerConfig) -> PlatformCapabilities {
        let mut caps = PlatformCapabilities::default();

        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            caps.ui.multi_window = true;
            caps.ui.window_tear_off = true;
            caps.ui.cursor_icons = true;

            caps.clipboard.text = true;
            caps.clipboard.files = false;

            caps.dnd.external = true;
            // The portable external drag contract is token-based (ADR 0053).
            caps.dnd.external_payload = ExternalDragPayloadKind::FileToken;
            caps.dnd.external_position = ExternalDragPositionQuality::Continuous;

            // winit on macOS does not reliably provide continuous drag-over cursor positions for
            // external file drags (see `docs/known-issues.md`).
            #[cfg(target_os = "macos")]
            {
                caps.dnd.external_position = ExternalDragPositionQuality::BestEffort;
            }

            caps.ime.enabled = true;
            caps.ime.set_cursor_area = true;

            caps.fs.real_paths = true;
            caps.fs.file_dialogs = true;

            caps.shell.open_url = true;

            caps.gfx.native_gpu = true;
            caps.gfx.webgpu = false;
        }

        #[cfg(target_arch = "wasm32")]
        {
            caps.ui.multi_window = false;
            caps.ui.window_tear_off = false;
            caps.ui.cursor_icons = false;

            caps.clipboard.text = false;
            caps.clipboard.files = false;

            caps.dnd.external = false;
            caps.dnd.external_payload = ExternalDragPayloadKind::None;
            caps.dnd.external_position = ExternalDragPositionQuality::None;

            caps.ime.enabled = true;
            caps.ime.set_cursor_area = false;

            caps.fs.real_paths = false;
            caps.fs.file_dialogs = false;

            caps.shell.open_url = true;

            caps.gfx.native_gpu = false;
            caps.gfx.webgpu = true;
        }

        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            caps.ui.multi_window = false;
            caps.ui.window_tear_off = false;
            caps.ui.cursor_icons = false;

            caps.clipboard.text = true;
            caps.clipboard.files = false;

            caps.dnd.external = false;
            caps.dnd.external_payload = ExternalDragPayloadKind::None;
            caps.dnd.external_position = ExternalDragPositionQuality::None;

            caps.ime.enabled = true;
            caps.ime.set_cursor_area = true;

            caps.fs.real_paths = false;
            caps.fs.file_dialogs = false;

            caps.shell.open_url = true;

            caps.gfx.native_gpu = true;
            caps.gfx.webgpu = false;
        }

        caps
    }

    fn effective_platform_capabilities(
        config: &WinitRunnerConfig,
        requested: &PlatformCapabilities,
    ) -> PlatformCapabilities {
        let available = Self::backend_platform_capabilities(config);
        let mut caps = requested.clone();

        caps.ui.multi_window &= available.ui.multi_window;
        caps.ui.window_tear_off &= available.ui.window_tear_off;
        caps.ui.cursor_icons &= available.ui.cursor_icons;

        caps.clipboard.text &= available.clipboard.text;
        caps.clipboard.files &= available.clipboard.files;

        caps.dnd.external &= available.dnd.external;
        caps.dnd.external_payload =
            match (caps.dnd.external_payload, available.dnd.external_payload) {
                (ExternalDragPayloadKind::None, _) => ExternalDragPayloadKind::None,
                (_, ExternalDragPayloadKind::None) => ExternalDragPayloadKind::None,
                (requested, available) if requested == available => requested,
                // Narrow to the backend's portable contract if the requested mode isn't supported.
                (_, available) => available,
            };
        caps.dnd.external_position = if caps.dnd.external {
            caps.dnd
                .external_position
                .clamp_to_available(available.dnd.external_position)
        } else {
            ExternalDragPositionQuality::None
        };

        caps.ime.enabled &= available.ime.enabled;
        caps.ime.set_cursor_area &= available.ime.set_cursor_area;

        caps.fs.real_paths &= available.fs.real_paths;
        caps.fs.file_dialogs &= available.fs.file_dialogs;

        caps.shell.open_url &= available.shell.open_url;

        caps.gfx.native_gpu &= available.gfx.native_gpu;
        caps.gfx.webgpu &= available.gfx.webgpu;

        caps
    }

    /// Sets the event-loop proxy used to deliver asynchronous platform completions back into the
    /// window event stream.
    ///
    /// Without a proxy, the runner falls back to synchronous delivery for platform effects.
    pub fn set_event_loop_proxy(&mut self, proxy: EventLoopProxy) {
        self.event_loop_proxy = Some(proxy);
    }

    fn spawn_platform_completion_task<F>(&self, window: fret_core::AppWindowId, task: F) -> bool
    where
        F: FnOnce() -> PlatformCompletion + Send + 'static,
    {
        let Some(proxy) = self.event_loop_proxy.clone() else {
            return false;
        };
        let events = self.proxy_events.clone();

        std::thread::spawn(move || {
            let completion = task();
            if let Ok(mut queue) = events.lock() {
                queue.push(RunnerUserEvent::PlatformCompletion { window, completion });
            }
            proxy.wake_up();
        });

        true
    }

    fn deliver_window_event_now(&mut self, window: fret_core::AppWindowId, event: &Event) {
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
        self.driver
            .handle_event(&mut self.app, services, window, &mut state.user, event);
    }

    fn deliver_platform_completion_now(
        &mut self,
        window: fret_core::AppWindowId,
        completion: PlatformCompletion,
    ) {
        match completion {
            PlatformCompletion::ClipboardText { token, text } => {
                self.deliver_window_event_now(window, &Event::ClipboardText { token, text });
            }
            PlatformCompletion::ClipboardTextUnavailable { token } => {
                self.deliver_window_event_now(window, &Event::ClipboardTextUnavailable { token });
            }
            PlatformCompletion::ExternalDropData(data) => {
                self.deliver_window_event_now(window, &Event::ExternalDropData(data));
            }
            PlatformCompletion::FileDialogSelection(selection) => {
                self.deliver_window_event_now(window, &Event::FileDialogSelection(selection));
            }
            PlatformCompletion::FileDialogData(data) => {
                self.deliver_window_event_now(window, &Event::FileDialogData(data));
            }
            PlatformCompletion::FileDialogCanceled => {
                self.deliver_window_event_now(window, &Event::FileDialogCanceled);
            }
        }
    }

    fn external_drag_files(
        token: fret_runtime::ExternalDropToken,
        paths: &[std::path::PathBuf],
    ) -> ExternalDragFiles {
        let files = paths
            .iter()
            .map(|p| ExternalDragFile {
                name: p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| p.to_string_lossy().to_string()),
            })
            .collect();
        ExternalDragFiles { token, files }
    }

    fn ui_services_mut<'a>(
        renderer: &'a mut Option<Renderer>,
        no_services: &'a mut NoUiServices,
    ) -> &'a mut dyn UiServices {
        match renderer.as_mut() {
            Some(renderer) => renderer as &mut dyn UiServices,
            None => no_services as &mut dyn UiServices,
        }
    }

    fn create_os_window(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        spec: WindowCreateSpec,
    ) -> Result<(Arc<dyn Window>, Option<accessibility::WinitAccessibility>), RunnerError> {
        let mut attrs = winit::window::WindowAttributes::default()
            .with_title(spec.title)
            .with_surface_size(spec.size)
            .with_visible(if self.config.accessibility_enabled {
                false
            } else {
                spec.visible
            });
        if let Some(position) = spec.position {
            attrs = attrs.with_position(position);
        }
        let window = Arc::<dyn Window>::from(
            event_loop
                .create_window(attrs)
                .map_err(|source| RunnerError::CreateWindowFailed { source })?,
        );

        macos_window_log(format_args!("[create] winit={:?}", window.id()));

        let accessibility = self
            .config
            .accessibility_enabled
            .then(|| accessibility::WinitAccessibility::new(event_loop, window.as_ref()));

        if self.config.accessibility_enabled && spec.visible {
            window.set_visible(true);
        }

        Ok((window, accessibility))
    }

    fn insert_window(
        &mut self,
        window: Arc<dyn Window>,
        accessibility: Option<accessibility::WinitAccessibility>,
        surface: wgpu::Surface<'static>,
    ) -> Result<fret_core::AppWindowId, RunnerError> {
        let Some(context) = self.context.as_ref() else {
            return Err(RunnerError::WgpuNotInitialized);
        };

        let size = window.surface_size();
        let surface = SurfaceState::new(
            &context.adapter,
            &context.device,
            surface,
            size.width,
            size.height,
        )?;

        let id = self.windows.insert_with_key(|id| {
            let user = self.driver.create_window_state(&mut self.app, id);
            WindowRuntime {
                window,
                accessibility,
                last_accessibility_snapshot: None,
                surface,
                scene: Scene::default(),
                platform: fret_runner_winit::WinitPlatform {
                    wheel: fret_runner_winit::WheelConfig {
                        line_delta_px: self.config.wheel_line_delta_px,
                        pixel_delta_scale: self.config.wheel_pixel_delta_scale,
                    },
                    ..Default::default()
                },
                is_focused: false,
                external_drag_files: Vec::new(),
                external_drag_token: None,
                user,
            }
        });

        if let Some(state) = self.windows.get(id) {
            let size_phys = state.window.surface_size();
            let size_logical: winit::dpi::LogicalSize<f32> =
                size_phys.to_logical(state.window.scale_factor());
            self.app
                .with_global_mut(WindowMetricsService::default, |svc, _app| {
                    svc.set_inner_size(
                        id,
                        Size::new(Px(size_logical.width), Px(size_logical.height)),
                    );
                });
        }

        let winit_id = self.windows[id].window.id();
        self.window_registry.insert(winit_id, id);

        // Ensure the window draws at least one frame after creation.
        //
        // Important: `WindowEvent::RedrawRequested` is keyed by the winit `WindowId`, so we must
        // install the `WindowId -> AppWindowId` mapping *before* requesting the redraw. Otherwise, the first
        // redraw can be dropped and the window may appear blank until another event arrives.
        if let Some(state) = self.windows.get(id) {
            state.window.request_redraw();
        }
        Ok(id)
    }

    fn resize_surface(&mut self, window: fret_core::AppWindowId, width: u32, height: u32) {
        let Some(context) = self.context.as_ref() else {
            return;
        };
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        state.surface.resize(&context.device, width, height);
    }

    fn close_window(&mut self, window: fret_core::AppWindowId) {
        let should_close = self.driver.before_close_window(&mut self.app, window);
        if !should_close {
            return;
        }

        if let Some(state) = self.windows.remove(window) {
            self.window_registry.remove(state.window.id());
        }
        self.app
            .with_global_mut(WindowMetricsService::default, |svc, _app| {
                svc.remove(window);
            });
        if Some(window) == self.main_window {
            self.main_window = None;
        }
    }

    fn compute_window_position_from_anchor(&self, anchor: WindowAnchor) -> Option<Position> {
        let anchor_state = self.windows.get(anchor.window)?;
        // `WindowAnchor::position` is in surface-local logical coordinates (matching pointer
        // events), so start from the surface origin in desktop coordinates.
        let outer = anchor_state.window.outer_position().ok()?;
        let surface = anchor_state.window.surface_position();
        let scale = anchor_state.window.scale_factor();

        let (ox, oy) = self.config.new_window_anchor_offset;
        let mut x = outer.x as f64 + surface.x as f64 + anchor.position.x.0 as f64 * scale + ox;
        let mut y = outer.y as f64 + surface.y as f64 + anchor.position.y.0 as f64 * scale + oy;

        // Best-effort clamping: avoid creating "off-screen" floating windows due to
        // platform-specific coordinate spaces and DPI conversions.
        if let Some(monitor) = anchor_state.window.current_monitor()
            && let (Some(pos), Some(mode)) = (monitor.position(), monitor.current_video_mode())
        {
            let size = mode.size();
            let min_x = pos.x as f64;
            let min_y = pos.y as f64;
            // Leave a small margin so the window stays reachable even if its size is larger
            // than the monitor work area.
            let max_x = min_x + size.width as f64 - 40.0;
            let max_y = min_y + size.height as f64 - 40.0;

            x = x.clamp(min_x, max_x);
            y = y.clamp(min_y, max_y);
        }

        Some(PhysicalPosition::new(x.round() as i32, y.round() as i32).into())
    }

    fn compute_window_position_from_cursor(
        &self,
        reference_window: fret_core::AppWindowId,
    ) -> Option<Position> {
        let screen_pos = self.cursor_screen_pos?;
        let ref_state = self.windows.get(reference_window)?;
        let (ox, oy) = self.config.new_window_anchor_offset;
        let mut x = screen_pos.x + ox;
        let mut y = screen_pos.y + oy;

        if let Some(monitor) = ref_state.window.current_monitor()
            && let (Some(pos), Some(mode)) = (monitor.position(), monitor.current_video_mode())
        {
            let size = mode.size();
            let min_x = pos.x as f64;
            let min_y = pos.y as f64;
            let max_x = min_x + size.width as f64 - 40.0;
            let max_y = min_y + size.height as f64 - 40.0;

            x = x.clamp(min_x, max_x);
            y = y.clamp(min_y, max_y);
        }

        Some(PhysicalPosition::new(x.round() as i32, y.round() as i32).into())
    }

    fn compute_window_outer_position_from_cursor_grab(
        &self,
        target_window: fret_core::AppWindowId,
        grab_offset_logical: Point,
    ) -> Option<Position> {
        let screen_pos = self.cursor_screen_pos?;
        let state = self.windows.get(target_window)?;
        let scale = state.window.scale_factor();

        if !grab_offset_logical.x.0.is_finite()
            || !grab_offset_logical.y.0.is_finite()
            || grab_offset_logical.x.0 < 0.0
            || grab_offset_logical.y.0 < 0.0
        {
            return None;
        }

        // Clamp the grab point to the target window's current client size. During tear-off, the
        // grab offset comes from the source window's client coordinates; if the new floating
        // window is smaller, keeping the original offset would place the cursor outside the new
        // window (visible as a fixed offset between cursor and window).
        let target_inner = state.window.surface_size();
        let target_inner_logical: winit::dpi::LogicalSize<f32> = target_inner.to_logical(scale);
        let max_x = target_inner_logical.width;
        let max_y = target_inner_logical.height;
        let mut grab_x = grab_offset_logical.x.0;
        let mut grab_y = grab_offset_logical.y.0;
        if max_x.is_finite() && max_x > 0.0 {
            grab_x = grab_x.min(max_x).max(0.0);
        } else {
            grab_x = 0.0;
        }
        if max_y.is_finite() && max_y > 0.0 {
            grab_y = grab_y.min(max_y).max(0.0);
        } else {
            grab_y = 0.0;
        }

        // Match ImGui's platform contract:
        // - viewport pos is client/inner screen position (logical)
        // - winit expects outer position
        // - therefore: outer = desired_client - decoration_offset(window)
        // See `repo-ref/dear-imgui-rs/backends/dear-imgui-winit/src/multi_viewport.rs:winit_set_window_pos`.
        let deco_offset = state.window.surface_position();

        let grab_client_x = grab_x as f64 * scale;
        let grab_client_y = grab_y as f64 * scale;
        let grab_outer_x = deco_offset.x as f64 + grab_client_x;
        let grab_outer_y = deco_offset.y as f64 + grab_client_y;

        let mut x = screen_pos.x - grab_outer_x;
        let mut y = screen_pos.y - grab_outer_y;

        // Align with ImGui docking/multi-viewport behavior:
        // - platform backend sets the window pos as requested
        // - visibility/reachability constraints are based on the *target monitor*, not the window's
        //   current monitor (which can pin the window at monitor edges).
        let outer_size = state.window.outer_size();

        #[cfg(target_os = "windows")]
        if let Some(work) = win32::monitor_work_area_for_point(screen_pos) {
            (x, y) = Self::clamp_window_outer_pos_to_monitor(
                x,
                y,
                outer_size,
                work,
                Self::WINDOW_VISIBILITY_PADDING_PX,
            );
        } else {
            let monitors = Self::monitor_rects_physical(state.window.as_ref());
            if let Some(idx) = Self::find_monitor_for_point(&monitors, screen_pos)
                && let Some(monitor) = monitors.get(idx).copied()
            {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            } else if let Some(monitor) = Self::virtual_desktop_bounds(state.window.as_ref()) {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let monitors = Self::monitor_rects_physical(state.window.as_ref());
            if let Some(idx) = Self::find_monitor_for_point(&monitors, screen_pos)
                && let Some(monitor) = monitors.get(idx).copied()
            {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            } else if let Some(monitor) = Self::virtual_desktop_bounds(state.window.as_ref()) {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            }
        }

        Some(PhysicalPosition::new(x.round() as i32, y.round() as i32).into())
    }

    fn create_window_from_request(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        request: &CreateWindowRequest,
    ) -> Result<fret_core::AppWindowId, RunnerError> {
        let mut spec = self
            .driver
            .window_create_spec(&mut self.app, request)
            .unwrap_or_else(|| self.config.default_window_spec());

        if spec.position.is_none() {
            // For dock tear-off, initially place near the cursor; we will refine the position
            // after the OS window exists using its own decoration offset (ImGui-style).
            if let CreateWindowKind::DockFloating { source_window, .. } = request.kind {
                spec.position = self.compute_window_position_from_cursor(source_window);
            }

            if spec.position.is_none()
                && let Some(anchor) = request.anchor
            {
                spec.position = self.compute_window_position_from_anchor(anchor);
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Avoid the "flash behind the source window" when tearing off a dock panel by
            // creating the new OS window hidden, then letting the deferred raise show it.
            if let CreateWindowKind::DockFloating { source_window, .. } = request.kind
                && !self.is_left_mouse_down_for_window(source_window)
            {
                spec.visible = false;
            }
        }

        let (window, accessibility) = self.create_os_window(event_loop, spec)?;
        let surface = {
            let Some(context) = self.context.as_ref() else {
                return Err(RunnerError::WgpuNotInitialized);
            };
            context.create_surface(window.clone())?
        };
        self.insert_window(window, accessibility, surface)
    }

    fn schedule_timer(&mut self, now: Instant, effect: &Effect) {
        let Effect::SetTimer {
            window,
            token,
            after,
            repeat,
        } = effect
        else {
            return;
        };
        self.timers.insert(
            *token,
            TimerEntry {
                window: *window,
                deadline: now + *after,
                repeat: *repeat,
            },
        );
    }

    fn fire_due_timers(&mut self, now: Instant) -> bool {
        let mut fired_any = false;
        let mut due: Vec<fret_runtime::TimerToken> = Vec::new();
        for (token, entry) in &self.timers {
            if entry.deadline <= now {
                due.push(*token);
            }
        }

        for token in due {
            let Some(entry) = self.timers.get(&token).cloned() else {
                continue;
            };
            fired_any = true;

            let target = entry
                .window
                .or(self.main_window)
                .and_then(|w| self.windows.contains_key(w).then_some(w));

            if let Some(window) = target {
                let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                if let Some(state) = self.windows.get_mut(window) {
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        window,
                        &mut state.user,
                        &Event::Timer { token },
                    );
                }
            }

            match entry.repeat {
                Some(interval) => {
                    if let Some(e) = self.timers.get_mut(&token) {
                        e.deadline = now + interval;
                    }
                }
                None => {
                    self.timers.remove(&token);
                }
            }
        }

        fired_any
    }

    fn drain_effects(&mut self, event_loop: &dyn ActiveEventLoop) {
        const MAX_EFFECT_DRAIN_TURNS: usize = 8;

        for _ in 0..MAX_EFFECT_DRAIN_TURNS {
            let now = Instant::now();
            let effects = self.app.flush_effects();

            let mut did_work = !effects.is_empty();
            let mut window_state_dirty: HashSet<fret_core::AppWindowId> = HashSet::new();

            for effect in effects {
                match effect {
                    Effect::Redraw(window) => {
                        if let Some(state) = self.windows.get(window) {
                            state.window.request_redraw();
                        }
                    }
                    Effect::ImeAllow { window, enabled } => {
                        if let Some(state) = self.windows.get_mut(window) {
                            if state.platform.set_ime_allowed(enabled) {
                                window_state_dirty.insert(window);
                            }
                        }
                    }
                    Effect::ImeSetCursorArea { window, rect } => {
                        if let Some(state) = self.windows.get_mut(window) {
                            if std::env::var_os("FRET_IME_DEBUG").is_some_and(|v| !v.is_empty()) {
                                tracing::info!(
                                    "IME_DEBUG effect: ImeSetCursorArea window={:?} rect=({:.1},{:.1} {:.1}x{:.1})",
                                    window,
                                    rect.origin.x.0,
                                    rect.origin.y.0,
                                    rect.size.width.0,
                                    rect.size.height.0
                                );
                            }
                            if state.platform.set_ime_cursor_area(rect) {
                                window_state_dirty.insert(window);
                            }
                        }
                    }
                    Effect::CursorSetIcon { window, icon } => {
                        let Some(state) = self.windows.get_mut(window) else {
                            continue;
                        };
                        if state.platform.set_cursor_icon(icon) {
                            window_state_dirty.insert(window);
                        }
                    }
                    Effect::RequestAnimationFrame(window) => {
                        self.raf_windows.insert(window);
                        if let Some(state) = self.windows.get(window) {
                            state.window.request_redraw();
                        }
                    }
                    Effect::SetTimer { .. } => {
                        self.schedule_timer(now, &effect);
                    }
                    Effect::CancelTimer { token } => {
                        self.timers.remove(&token);
                    }
                    Effect::Command { window, command } => match window {
                        Some(window) => {
                            if let Some(state) = self.windows.get_mut(window) {
                                let services = Self::ui_services_mut(
                                    &mut self.renderer,
                                    &mut self.no_services,
                                );
                                self.driver.handle_command(
                                    &mut self.app,
                                    services,
                                    window,
                                    &mut state.user,
                                    command,
                                );
                            }
                        }
                        None => {
                            let services =
                                Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                            self.driver
                                .handle_global_command(&mut self.app, services, command);
                        }
                    },
                    Effect::ClipboardSetText { text } => {
                        if let Err(err) = self.clipboard.set_text(&text) {
                            tracing::debug!(?err, "failed to set clipboard text");
                        }
                    }
                    Effect::ClipboardGetText { window, token } => match self.clipboard.get_text() {
                        Ok(Some(text)) => self.deliver_window_event_now(
                            window,
                            &Event::ClipboardText { token, text },
                        ),
                        Ok(None) | Err(_) => self.deliver_window_event_now(
                            window,
                            &Event::ClipboardTextUnavailable { token },
                        ),
                    },
                    Effect::ExternalDropReadAll { window, token } => {
                        let limits = fret_platform::external_drop::ExternalDropReadLimits {
                            max_total_bytes: self.config.external_drop_max_total_bytes,
                            max_file_bytes: self.config.external_drop_max_file_bytes,
                            max_files: self.config.external_drop_max_files,
                        };

                        if let Some(paths) = self.external_drop.paths(token).map(|p| p.to_vec())
                            && self.spawn_platform_completion_task(window, move || {
                                let event = NativeExternalDrop::read_paths(token, paths, limits);
                                PlatformCompletion::ExternalDropData(event)
                            })
                        {
                            continue;
                        }

                        let Some(event) = self.external_drop.read_all(token, limits) else {
                            continue;
                        };
                        self.deliver_window_event_now(window, &Event::ExternalDropData(event));
                    }
                    Effect::ExternalDropReadAllWithLimits {
                        window,
                        token,
                        limits,
                    } => {
                        let cap = fret_platform::external_drop::ExternalDropReadLimits {
                            max_total_bytes: self.config.external_drop_max_total_bytes,
                            max_file_bytes: self.config.external_drop_max_file_bytes,
                            max_files: self.config.external_drop_max_files,
                        };
                        let limits = limits.capped_by(cap);

                        if let Some(paths) = self.external_drop.paths(token).map(|p| p.to_vec())
                            && self.spawn_platform_completion_task(window, move || {
                                let event = NativeExternalDrop::read_paths(token, paths, limits);
                                PlatformCompletion::ExternalDropData(event)
                            })
                        {
                            continue;
                        }

                        let Some(event) = self.external_drop.read_all(token, limits) else {
                            continue;
                        };
                        self.deliver_window_event_now(window, &Event::ExternalDropData(event));
                    }
                    Effect::ExternalDropRelease { token } => {
                        self.external_drop.release(token);
                    }
                    Effect::OpenUrl { url } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.shell.open_url {
                            continue;
                        }
                        if let Err(err) = self.open_url.open_url(&url) {
                            tracing::debug!(?err, url = %url, "failed to open url");
                        }
                    }
                    Effect::FileDialogOpen { window, options } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.fs.file_dialogs {
                            continue;
                        }
                        match self.file_dialog.open_files(&options) {
                            Ok(Some(selection)) => {
                                self.deliver_platform_completion_now(
                                    window,
                                    PlatformCompletion::FileDialogSelection(selection),
                                );
                            }
                            Ok(None) => {
                                self.deliver_platform_completion_now(
                                    window,
                                    PlatformCompletion::FileDialogCanceled,
                                );
                            }
                            Err(err) => {
                                tracing::debug!(?err, "file dialog open failed");
                            }
                        }
                    }
                    Effect::FileDialogReadAll { window, token } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.fs.file_dialogs {
                            continue;
                        }
                        let limits = fret_platform::external_drop::ExternalDropReadLimits {
                            max_total_bytes: self.config.file_dialog_max_total_bytes,
                            max_file_bytes: self.config.file_dialog_max_file_bytes,
                            max_files: self.config.file_dialog_max_files,
                        };

                        if let Some(paths) = self.file_dialog.paths(token).map(|p| p.to_vec())
                            && self.spawn_platform_completion_task(window, move || {
                                let data = NativeFileDialog::read_paths(token, paths, limits);
                                PlatformCompletion::FileDialogData(data)
                            })
                        {
                            continue;
                        }

                        let Some(data) = self.file_dialog.read_all(token, limits) else {
                            continue;
                        };
                        self.deliver_platform_completion_now(
                            window,
                            PlatformCompletion::FileDialogData(data),
                        );
                    }
                    Effect::FileDialogReadAllWithLimits {
                        window,
                        token,
                        limits,
                    } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.fs.file_dialogs {
                            continue;
                        }
                        let cap = fret_platform::external_drop::ExternalDropReadLimits {
                            max_total_bytes: self.config.file_dialog_max_total_bytes,
                            max_file_bytes: self.config.file_dialog_max_file_bytes,
                            max_files: self.config.file_dialog_max_files,
                        };
                        let limits = limits.capped_by(cap);

                        if let Some(paths) = self.file_dialog.paths(token).map(|p| p.to_vec())
                            && self.spawn_platform_completion_task(window, move || {
                                let data = NativeFileDialog::read_paths(token, paths, limits);
                                PlatformCompletion::FileDialogData(data)
                            })
                        {
                            continue;
                        }

                        let Some(data) = self.file_dialog.read_all(token, limits) else {
                            continue;
                        };
                        self.deliver_platform_completion_now(
                            window,
                            PlatformCompletion::FileDialogData(data),
                        );
                    }
                    Effect::FileDialogRelease { token } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.fs.file_dialogs {
                            continue;
                        }
                        self.file_dialog.release(token);
                    }
                    Effect::TextAddFonts { fonts } => {
                        let Some(renderer) = self.renderer.as_mut() else {
                            continue;
                        };

                        let added = renderer.add_fonts(fonts);
                        if added == 0 {
                            continue;
                        }

                        let families = renderer.all_font_names();
                        let _update = fret_runtime::apply_font_catalog_update(
                            &mut self.app,
                            families,
                            fret_runtime::FontFamilyDefaultsPolicy::None,
                        );

                        for (_id, state) in self.windows.iter() {
                            state.window.request_redraw();
                        }
                    }
                    Effect::ImageRegisterRgba8 {
                        window,
                        token,
                        width,
                        height,
                        bytes,
                        color_space,
                    } => {
                        let Some(context) = self.context.as_ref() else {
                            self.deliver_window_event_now(
                                window,
                                &Event::ImageRegisterFailed {
                                    token,
                                    message: "wgpu not initialized".to_string(),
                                },
                            );
                            continue;
                        };
                        let Some(renderer) = self.renderer.as_mut() else {
                            self.deliver_window_event_now(
                                window,
                                &Event::ImageRegisterFailed {
                                    token,
                                    message: "renderer not initialized".to_string(),
                                },
                            );
                            continue;
                        };

                        if width == 0 || height == 0 {
                            self.deliver_window_event_now(
                                window,
                                &Event::ImageRegisterFailed {
                                    token,
                                    message: format!("invalid image size: {width}x{height}"),
                                },
                            );
                            continue;
                        }

                        let expected_len = (width as usize)
                            .saturating_mul(height as usize)
                            .saturating_mul(4);
                        if bytes.len() != expected_len {
                            self.deliver_window_event_now(
                                window,
                                &Event::ImageRegisterFailed {
                                    token,
                                    message: format!(
                                        "invalid rgba8 byte length: got {} expected {}",
                                        bytes.len(),
                                        expected_len
                                    ),
                                },
                            );
                            continue;
                        }

                        let color_space = match color_space {
                            fret_runtime::ImageColorSpace::Srgb => {
                                fret_render::ImageColorSpace::Srgb
                            }
                            fret_runtime::ImageColorSpace::Linear => {
                                fret_render::ImageColorSpace::Linear
                            }
                        };

                        let fret_render::UploadedRgba8Image {
                            view,
                            size,
                            format,
                            color_space,
                            ..
                        } = fret_render::upload_rgba8_image(
                            &context.device,
                            &context.queue,
                            (width, height),
                            &bytes,
                            color_space,
                        );

                        let image = renderer.register_image(fret_render::ImageDescriptor {
                            view,
                            size,
                            format,
                            color_space,
                        });

                        self.deliver_window_event_now(
                            window,
                            &Event::ImageRegistered {
                                token,
                                image,
                                width,
                                height,
                            },
                        );
                        if let Some(state) = self.windows.get(window) {
                            state.window.request_redraw();
                        }
                    }
                    Effect::ImageUnregister { image } => {
                        let Some(renderer) = self.renderer.as_mut() else {
                            continue;
                        };

                        if !renderer.unregister_image(image) {
                            continue;
                        }

                        for (_id, state) in self.windows.iter() {
                            state.window.request_redraw();
                        }
                    }
                    Effect::ViewportInput(event) => {
                        self.driver.viewport_input(&mut self.app, event);
                    }
                    Effect::Dock(op) => {
                        if matches!(op, fret_core::DockOp::RequestFloatPanelToNewWindow { .. }) {
                            dock_tearoff_log(format_args!("[effect-dock] {:?}", op));
                        }
                        self.driver.dock_op(&mut self.app, op);
                    }
                    Effect::Window(req) => match req {
                        WindowRequest::Close(window) => {
                            let is_main = Some(window) == self.main_window;
                            if is_main && self.config.exit_on_main_window_close {
                                event_loop.exit();
                                return;
                            }
                            self.close_window(window);
                            if is_main && self.windows.is_empty() {
                                event_loop.exit();
                                return;
                            }
                        }
                        WindowRequest::Create(create) => {
                            if matches!(create.kind, CreateWindowKind::DockFloating { .. }) {
                                dock_tearoff_log(format_args!(
                                    "[effect-window-create] kind={:?} anchor={:?}",
                                    create.kind, create.anchor
                                ));
                            }
                            let new_window =
                                match self.create_window_from_request(event_loop, &create) {
                                    Ok(id) => id,
                                    Err(e) => {
                                        error!(error = ?e, "failed to create window from request");
                                        continue;
                                    }
                                };

                            if let CreateWindowKind::DockFloating { source_window, .. } =
                                &create.kind
                            {
                                #[cfg(target_os = "macos")]
                                {
                                    // When tearing off during an active drag, macOS may create the
                                    // new window behind the source window. Bring it to front
                                    // immediately so the subsequent `drag_window()` (if used)
                                    // behaves like ImGui's multi-viewport UX.
                                    let sender =
                                        self.windows.get(*source_window).map(|w| w.window.as_ref());
                                    if let Some(state) = self.windows.get(new_window) {
                                        let _ =
                                            bring_window_to_front(state.window.as_ref(), sender);
                                    }
                                }

                                if let Some(anchor) = create.anchor
                                    && let Some(state) = self.windows.get(new_window)
                                    && let Some(pos) = self
                                        .compute_window_outer_position_from_cursor_grab(
                                            new_window,
                                            anchor.position,
                                        )
                                {
                                    state.window.set_outer_position(pos);
                                }

                                if self.is_left_mouse_down_for_window(*source_window) {
                                    let grab_offset = create
                                        .anchor
                                        .map(|a| a.position)
                                        .unwrap_or(Point::new(Px(40.0), Px(20.0)));
                                    if let Some(state) = self.windows.get(new_window) {
                                        state.window.set_window_level(WindowLevel::AlwaysOnTop);
                                    }
                                    self.dock_tearoff_follow = Some(DockTearoffFollow {
                                        window: new_window,
                                        source_window: *source_window,
                                        grab_offset,
                                        manual_follow: true,
                                        last_outer_pos: None,
                                    });
                                    // Do not call `drag_window()` here. ImGui drives multi-viewport
                                    // window movement by updating the platform window position in
                                    // response to mouse motion; native OS dragging tends to
                                    // introduce a fixed cursor offset and prevents reliable
                                    // hit-testing of other windows under the moving viewport.
                                }
                                let panel = match &create.kind {
                                    CreateWindowKind::DockFloating { panel, .. } => Some(panel),
                                    _ => None,
                                };
                                self.enqueue_window_front(
                                    new_window,
                                    Some(*source_window),
                                    panel.cloned(),
                                    now,
                                );
                            }

                            self.driver
                                .window_created(&mut self.app, &create, new_window);

                            // If this window was created as part of an active dock tear-off drag,
                            // update the drag session's source window so dropping over another
                            // window emits `DockOp::MovePanel` with the correct `source_window`.
                            if let CreateWindowKind::DockFloating { source_window, .. } =
                                create.kind
                                && let Some(drag) = self.app.drag_mut()
                                && drag.kind == fret_app::DragKind::DockPanel
                                && drag.source_window == source_window
                            {
                                drag.source_window = new_window;
                                drag.current_window = new_window;
                            }

                            self.app.request_redraw(new_window);
                        }
                        WindowRequest::Raise {
                            window,
                            sender: sender_id,
                        } => {
                            let sender_window = sender_id
                                .and_then(|id| self.windows.get(id))
                                .map(|w| w.window.as_ref());
                            if let Some(state) = self.windows.get(window) {
                                let _ = bring_window_to_front(state.window.as_ref(), sender_window);
                                state.window.request_redraw();
                            }
                            #[cfg(target_os = "macos")]
                            {
                                if self.windows.contains_key(window) {
                                    self.enqueue_window_front(window, sender_id, None, now);
                                }
                            }
                        }
                    },
                }
            }

            for window in window_state_dirty {
                if let Some(state) = self.windows.get_mut(window) {
                    state.platform.prepare_frame(state.window.as_ref());
                }
            }

            did_work |= self.fire_due_timers(now);
            did_work |= self.clear_internal_drag_hover_if_needed();
            did_work |= self.propagate_model_changes();
            did_work |= self.propagate_global_changes();

            if !did_work {
                break;
            }
        }
    }

    fn propagate_model_changes(&mut self) -> bool {
        let changed = self.app.take_changed_models();
        if changed.is_empty() {
            return false;
        }

        for (window, runtime) in self.windows.iter_mut() {
            self.driver
                .handle_model_changes(&mut self.app, window, &mut runtime.user, &changed);
        }
        true
    }

    fn propagate_global_changes(&mut self) -> bool {
        let changed = self.app.take_changed_globals();
        if changed.is_empty() {
            return false;
        }

        if changed.contains(&TypeId::of::<fret_core::TextFontFamilyConfig>())
            && let (Some(renderer), Some(config)) = (
                self.renderer.as_mut(),
                self.app.global::<fret_core::TextFontFamilyConfig>(),
            )
            && renderer.set_text_font_families(config)
        {
            for (_id, state) in self.windows.iter() {
                state.window.request_redraw();
            }
        }

        for (window, runtime) in self.windows.iter_mut() {
            self.driver
                .handle_global_changes(&mut self.app, window, &mut runtime.user, &changed);
        }
        true
    }

    fn enqueue_window_front(
        &mut self,
        window: fret_core::AppWindowId,
        source_window: Option<fret_core::AppWindowId>,
        panel: Option<fret_core::PanelKey>,
        now: Instant,
    ) {
        macos_window_log(format_args!(
            "[enqueue-front] target={:?} source={:?} now={:?}",
            window, source_window, now
        ));

        // macOS may ignore focus changes during an active interaction in the source window.
        // Retry a few times over subsequent event-loop turns (and stop once the window reports
        // `Focused(true)`).
        self.windows_pending_front.insert(
            window,
            PendingFrontRequest {
                source_window,
                panel,
                created_at: now,
                // Defer the first raise to `about_to_wait` (Godot uses `call_deferred`); this
                // avoids fighting the platform while a tracked interaction is still active.
                next_attempt_at: now,
                attempts_left: 10,
            },
        );
    }

    fn process_pending_front_requests(&mut self, now: Instant) -> bool {
        if self.windows_pending_front.is_empty() {
            return false;
        }

        let pending = std::mem::take(&mut self.windows_pending_front);
        let mut kept: HashMap<fret_core::AppWindowId, PendingFrontRequest> = HashMap::new();
        let mut did_work = false;

        for (window, mut req) in pending {
            let Some(state) = self.windows.get(window) else {
                continue;
            };

            if state.is_focused && req.attempts_left > 2 {
                // Even after winit reports the window focused, the window ordering can still lag
                // behind when the float was initiated from a tracked menu / drag sequence.
                // Keep a couple more retries to ensure it actually surfaces.
                req.attempts_left = 2;
            }

            if req.attempts_left == 0 {
                macos_window_log(format_args!(
                    "[front-done] target={:?} panel={:?} focused={} age_ms={} now={:?}",
                    window,
                    req.panel.as_ref().map(|p| &p.kind.0),
                    state.is_focused,
                    now.saturating_duration_since(req.created_at).as_millis(),
                    now,
                ));
                continue;
            }

            if now >= req.next_attempt_at {
                macos_window_log(format_args!(
                    "[front-try] target={:?} panel={:?} source={:?} focused={} attempts_left={} age_ms={} now={:?}",
                    window,
                    req.panel.as_ref().map(|p| &p.kind.0),
                    req.source_window,
                    state.is_focused,
                    req.attempts_left,
                    now.saturating_duration_since(req.created_at).as_millis(),
                    now,
                ));
                let sender = req
                    .source_window
                    .and_then(|id| self.windows.get(id))
                    .map(|w| w.window.as_ref());
                let _ = bring_window_to_front(state.window.as_ref(), sender);
                state.window.request_redraw();
                req.attempts_left = req.attempts_left.saturating_sub(1);
                req.next_attempt_at = now + Duration::from_millis(60);
                did_work = true;
            }

            kept.insert(window, req);
        }

        self.windows_pending_front = kept;
        did_work
    }

    fn next_pending_front_deadline(&self) -> Option<Instant> {
        self.windows_pending_front
            .values()
            .filter(|r| r.attempts_left > 0)
            .map(|r| r.next_attempt_at)
            .min()
    }

    #[cfg(target_os = "macos")]
    fn maybe_finish_dock_drag_released_outside(&mut self) -> bool {
        let (source_window, current_window, dragging) = {
            let Some(drag) = self.app.drag() else {
                return false;
            };
            if !drag.cross_window_hover
                || drag.kind != fret_app::DragKind::DockPanel
                || macos_is_left_mouse_down()
                || self.saw_left_mouse_release_this_turn
            {
                return false;
            }
            (drag.source_window, drag.current_window, drag.dragging)
        };

        dock_tearoff_log(format_args!(
            "[poll-up] source={:?} current={:?} screen_pos={:?} dragging={}",
            source_window, current_window, self.cursor_screen_pos, dragging
        ));

        // If the mouse was released outside any window, winit may not deliver a `MouseInput`
        // event to any window. Use the regular cursor-based drop routing so docking back into an
        // existing window still works (ImGui-style).
        if let Some(d) = self.app.drag_mut()
            && d.kind == fret_app::DragKind::DockPanel
        {
            d.dragging = true;
        }

        self.route_internal_drag_drop_from_cursor();
        dock_tearoff_log(format_args!(
            "[poll-drop] dispatched target={:?}",
            source_window
        ));

        if self.app.drag().is_some_and(|d| d.cross_window_hover) {
            self.app.cancel_drag();
            let _ = self.clear_internal_drag_hover_if_needed();
        }

        true
    }

    fn dispatch_internal_drag_event(
        &mut self,
        window: fret_core::AppWindowId,
        kind: InternalDragKind,
        position: Point,
    ) {
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        let modifiers = state.platform.input.modifiers;
        let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
        self.driver.handle_event(
            &mut self.app,
            services,
            window,
            &mut state.user,
            &Event::InternalDrag(InternalDragEvent {
                position,
                kind,
                modifiers,
            }),
        );
    }

    fn clear_internal_drag_hover_if_needed(&mut self) -> bool {
        let Some(window) = self.internal_drag_hover_window else {
            return false;
        };
        if self.app.drag().is_some_and(|d| d.cross_window_hover) {
            return false;
        }
        self.internal_drag_hover_window = None;
        let pos = self.internal_drag_hover_pos.take().unwrap_or_default();
        self.dispatch_internal_drag_event(window, InternalDragKind::Cancel, pos);
        true
    }

    fn route_internal_drag_hover_from_cursor(&mut self) -> bool {
        let Some(drag) = self.app.drag() else {
            return self.clear_internal_drag_hover_if_needed();
        };
        if !drag.cross_window_hover {
            return self.clear_internal_drag_hover_if_needed();
        }

        let Some(screen_pos) = self.cursor_screen_pos else {
            return false;
        };

        // When a dock tear-off window is following the cursor, the cursor is always "inside" that
        // moving window. Prefer other windows under the cursor so we can dock back into the main
        // window (ImGui-style).
        let prefer_not = self
            .dock_tearoff_follow
            .filter(|_| drag.kind == fret_app::DragKind::DockPanel)
            .map(|f| f.window);

        // Prefer the window we already hovered, if the cursor is still inside it. This makes
        // cross-window drag hover stable even when OS windows overlap and we don't have z-order.
        let hovered = self
            .internal_drag_hover_window
            .filter(|w| self.screen_pos_in_window(*w, screen_pos))
            .filter(|w| Some(*w) != prefer_not)
            .or_else(|| self.window_under_cursor(screen_pos, prefer_not));
        let hovered = hovered.or_else(|| {
            // For dock tear-off, keep delivering `InternalDrag::Over` to the source window even
            // when the cursor is outside all windows so the UI can react before mouse-up.
            (drag.kind == fret_app::DragKind::DockPanel)
                .then_some(drag.source_window)
                .filter(|w| self.windows.contains_key(*w))
        });
        if hovered != self.internal_drag_hover_window {
            if let Some(prev) = self.internal_drag_hover_window.take() {
                let prev_pos = self.internal_drag_hover_pos.take().unwrap_or_default();
                self.dispatch_internal_drag_event(prev, InternalDragKind::Leave, prev_pos);
            }
            if let Some(next) = hovered
                && let Some(pos) = self.local_pos_for_window(next, screen_pos)
            {
                self.dispatch_internal_drag_event(next, InternalDragKind::Enter, pos);
                self.internal_drag_hover_window = Some(next);
                self.internal_drag_hover_pos = Some(pos);
            }
        }

        let Some(current) = self.internal_drag_hover_window else {
            return false;
        };
        let Some(pos) = self.local_pos_for_window(current, screen_pos) else {
            return false;
        };

        if let Some(d) = self.app.drag_mut() {
            d.current_window = current;
            d.position = pos;
        }

        self.internal_drag_hover_pos = Some(pos);
        self.dispatch_internal_drag_event(current, InternalDragKind::Over, pos);
        true
    }

    fn route_internal_drag_drop_from_cursor(&mut self) -> bool {
        let Some(drag) = self.app.drag() else {
            return false;
        };
        if !drag.cross_window_hover {
            return false;
        }

        let screen_pos = self
            .cursor_screen_pos
            .or_else(|| self.cursor_screen_pos_fallback_for_window(drag.source_window));
        let Some(screen_pos) = screen_pos else {
            return false;
        };

        let prefer_not = self
            .dock_tearoff_follow
            .filter(|_| drag.kind == fret_app::DragKind::DockPanel)
            .map(|f| f.window);

        // Prefer the last hovered window if possible; window overlap makes hit-testing ambiguous.
        let target = self
            .internal_drag_hover_window
            .filter(|w| self.screen_pos_in_window(*w, screen_pos))
            .filter(|w| Some(*w) != prefer_not)
            .or_else(|| self.window_under_cursor(screen_pos, prefer_not))
            .or(self.internal_drag_hover_window);
        // If the cursor is outside all windows (Unity/ImGui-style tear-off), still deliver the
        // drop to the source window using the last known screen cursor position.
        let target = target.unwrap_or(drag.source_window);
        let pos = self.local_pos_for_window(target, screen_pos).or_else(|| {
            if self.internal_drag_hover_window == Some(target) {
                self.internal_drag_hover_pos
            } else {
                None
            }
        });
        let Some(pos) = pos else {
            return false;
        };

        if drag.kind == fret_app::DragKind::DockPanel
            && target != drag.source_window
            && let Some(runtime) = self.windows.get(target)
        {
            let sender = self
                .windows
                .get(drag.source_window)
                .map(|w| w.window.as_ref());
            let _ = bring_window_to_front(runtime.window.as_ref(), sender);
        }

        if let Some(prev) = self.internal_drag_hover_window.take()
            && prev != target
        {
            let prev_pos = self.internal_drag_hover_pos.take().unwrap_or_default();
            self.dispatch_internal_drag_event(prev, InternalDragKind::Leave, prev_pos);
        }
        self.internal_drag_hover_window = Some(target);
        self.internal_drag_hover_pos = Some(pos);

        if let Some(d) = self.app.drag_mut() {
            d.current_window = target;
            d.position = pos;
        }

        self.dispatch_internal_drag_event(target, InternalDragKind::Drop, pos);
        true
    }

    fn cursor_screen_pos_fallback_for_window(
        &self,
        window: fret_core::AppWindowId,
    ) -> Option<PhysicalPosition<f64>> {
        let state = self.windows.get(window)?;
        let inner = state.window.surface_position();
        let scale = state.window.scale_factor();
        let x = inner.x as f64 + state.platform.input.cursor_pos.x.0 as f64 * scale;
        let y = inner.y as f64 + state.platform.input.cursor_pos.y.0 as f64 * scale;
        Some(PhysicalPosition::new(x, y))
    }

    fn screen_pos_in_window(
        &self,
        window: fret_core::AppWindowId,
        screen_pos: PhysicalPosition<f64>,
    ) -> bool {
        let Some(state) = self.windows.get(window) else {
            return false;
        };
        let inner = state.window.surface_position();
        let size = state.window.surface_size();
        let left = inner.x as f64;
        let top = inner.y as f64;
        let right = left + size.width as f64;
        let bottom = top + size.height as f64;
        screen_pos.x >= left && screen_pos.x < right && screen_pos.y >= top && screen_pos.y < bottom
    }

    fn local_pos_for_window(
        &self,
        window: fret_core::AppWindowId,
        screen_pos: PhysicalPosition<f64>,
    ) -> Option<Point> {
        let state = self.windows.get(window)?;
        let inner = state.window.surface_position();
        let local_physical =
            PhysicalPosition::new(screen_pos.x - inner.x as f64, screen_pos.y - inner.y as f64);
        let local_logical: winit::dpi::LogicalPosition<f32> =
            local_physical.to_logical(state.window.scale_factor());
        Some(Point::new(Px(local_logical.x), Px(local_logical.y)))
    }

    fn window_under_cursor(
        &self,
        screen_pos: PhysicalPosition<f64>,
        prefer_not: Option<fret_core::AppWindowId>,
    ) -> Option<fret_core::AppWindowId> {
        let mut fallback: Option<fret_core::AppWindowId> = None;
        for w in self.windows.keys() {
            let Some(state) = self.windows.get(w) else {
                continue;
            };
            let inner = state.window.surface_position();
            let size = state.window.surface_size();
            let left = inner.x as f64;
            let top = inner.y as f64;
            let right = left + size.width as f64;
            let bottom = top + size.height as f64;
            if screen_pos.x >= left
                && screen_pos.x < right
                && screen_pos.y >= top
                && screen_pos.y < bottom
            {
                if prefer_not.is_some_and(|p| p == w) {
                    fallback = Some(w);
                    continue;
                }
                return Some(w);
            }
        }
        fallback
    }

    fn is_left_mouse_down_for_window(&self, window: fret_core::AppWindowId) -> bool {
        #[cfg(target_os = "macos")]
        {
            if macos_is_left_mouse_down() {
                return true;
            }
        }

        self.left_mouse_down
            || self
                .windows
                .get(window)
                .is_some_and(|w| w.platform.input.pressed_buttons.left)
    }

    fn update_dock_tearoff_follow(&mut self) -> bool {
        let (window, grab_offset, manual_follow, last_outer_pos) = match self.dock_tearoff_follow {
            Some(follow) => (
                follow.window,
                follow.grab_offset,
                follow.manual_follow,
                follow.last_outer_pos,
            ),
            None => return false,
        };

        if !manual_follow {
            return false;
        }

        if self.windows.get(window).is_none() {
            self.dock_tearoff_follow = None;
            return false;
        }

        let Some(pos) = self.compute_window_outer_position_from_cursor_grab(window, grab_offset)
        else {
            return false;
        };

        let next_phys = {
            let Some(state) = self.windows.get(window) else {
                self.dock_tearoff_follow = None;
                return false;
            };
            let scale_factor = state.window.scale_factor();
            match pos {
                Position::Physical(p) => p,
                Position::Logical(p) => p.to_physical::<i32>(scale_factor),
            }
        };

        // Avoid spamming redundant position updates (helps reduce stutter on high-frequency
        // input devices).
        if last_outer_pos.is_some_and(|prev| prev == next_phys) {
            return false;
        }

        if let Some(state) = self.windows.get(window) {
            // Keep the moving window visible while docking back into another window (ImGui-style).
            state.window.set_window_level(WindowLevel::AlwaysOnTop);
            state.window.set_outer_position(pos);
        }

        dock_tearoff_log(format_args!(
            "[follow-move] window={:?} cursor={:?} outer_pos={:?}",
            window, self.cursor_screen_pos, next_phys
        ));

        if let Some(follow) = self.dock_tearoff_follow.as_mut() {
            follow.last_outer_pos = Some(next_phys);
        }

        true
    }

    fn stop_dock_tearoff_follow(&mut self, _now: Instant, _raise_on_macos: bool) {
        let Some(follow) = self.dock_tearoff_follow.take() else {
            return;
        };

        if let Some(state) = self.windows.get(follow.window) {
            state.window.set_window_level(WindowLevel::Normal);
            if let Some(pos) =
                self.settle_window_outer_position(state.window.as_ref(), self.cursor_screen_pos)
            {
                state.window.set_outer_position(Position::Physical(pos));
            }
        }

        #[cfg(target_os = "macos")]
        if _raise_on_macos {
            self.enqueue_window_front(follow.window, Some(follow.source_window), None, _now);
        }
    }
}

impl<D: WinitAppDriver> WinitDriver for WinitAppDriverAdapter<D> {
    type WindowState = D::WindowState;

    fn init(&mut self, app: &mut App, main_window: fret_core::AppWindowId) {
        self.inner.init(app, main_window);
    }

    fn gpu_ready(&mut self, app: &mut App, context: &WgpuContext, renderer: &mut Renderer) {
        self.inner.gpu_ready(app, context, renderer);
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
        tick_id: TickId,
        frame_id: FrameId,
    ) -> EngineFrameUpdate {
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
        tick_id: TickId,
        frame_id: FrameId,
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

    fn viewport_input(&mut self, app: &mut App, event: ViewportInputEvent) {
        self.inner.viewport_input(app, event);
    }

    fn dock_op(&mut self, app: &mut App, op: fret_core::DockOp) {
        self.inner.dock_op(app, op);
    }

    fn handle_command(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        command: fret_app::CommandId,
    ) {
        self.inner.handle_command(
            WinitCommandContext {
                app,
                services,
                window,
                state,
            },
            command,
        );
    }

    fn handle_global_command(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        command: fret_app::CommandId,
    ) {
        self.inner
            .handle_global_command(WinitGlobalContext { app, services }, command);
    }

    fn handle_model_changes(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        changed: &[fret_app::ModelId],
    ) {
        self.inner
            .handle_model_changes(WinitWindowContext { app, window, state }, changed);
    }

    fn handle_global_changes(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        changed: &[TypeId],
    ) {
        self.inner
            .handle_global_changes(WinitWindowContext { app, window, state }, changed);
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
        app: &mut App,
        services: &mut dyn UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        event: &Event,
    ) {
        self.inner.handle_event(
            WinitEventContext {
                app,
                services,
                window,
                state,
            },
            event,
        );
    }

    fn render(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        bounds: Rect,
        scale_factor: f32,
        services: &mut dyn fret_core::UiServices,
        scene: &mut Scene,
    ) {
        self.inner.render(WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        });
    }

    fn window_create_spec(
        &mut self,
        app: &mut App,
        request: &CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        self.inner.window_create_spec(app, request)
    }

    fn window_created(
        &mut self,
        app: &mut App,
        request: &CreateWindowRequest,
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
        services: &mut dyn UiServices,
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
        services: &mut dyn UiServices,
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
        services: &mut dyn UiServices,
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
        services: &mut dyn UiServices,
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
        services: &mut dyn UiServices,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        self.inner
            .accessibility_replace_selected_text(app, services, window, state, target, value);
    }
}

impl<D: WinitAppDriver> WinitRunner<WinitAppDriverAdapter<D>> {
    pub fn new_app(config: WinitRunnerConfig, app: App, driver: D) -> Self {
        Self::new(config, app, WinitAppDriverAdapter::new(driver))
    }
}
