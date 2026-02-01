//! Desktop launcher implementation (winit + wgpu).

pub use super::common::*;

use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    fmt,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "hotpatch-subsecond")]
mod hotpatch;

use fret_app::{App, CreateWindowKind, CreateWindowRequest, Effect, WindowRequest};
use fret_core::{
    Event, ExternalDragEvent, ExternalDragKind, InternalDragEvent, InternalDragKind, Point, Px,
    Rect, Scene, Size, UiServices, ViewportInputEvent, WindowMetricsService,
};
use fret_platform_native::clipboard::NativeClipboard;
use fret_platform_native::external_drop::NativeExternalDrop;
use fret_platform_native::file_dialog::NativeFileDialog;
use fret_platform_native::open_url::NativeOpenUrl;
use fret_render::{Renderer, SurfaceState, UploadedRgba8Image, WgpuContext};
use fret_runner_winit::accessibility;
#[cfg(windows)]
use fret_runtime::TaskbarVisibility;
use fret_runtime::{
    ActivationPolicy, ExternalDragPayloadKind, ExternalDragPositionQuality, FrameId,
    PlatformCapabilities, PlatformCompletion, TickId, WindowStyleRequest, WindowZLevel,
};
use slotmap::SlotMap;
use tracing::error;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, Position},
    event::{DeviceEvent, ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{Window, WindowId, WindowLevel},
};

use crate::RunnerError;
use fret_platform::clipboard::Clipboard as _;
use fret_platform::external_drop::ExternalDropProvider as _;
use fret_platform::file_dialog::FileDialogProvider as _;
use fret_platform::open_url::OpenUrl as _;

type WindowAnchor = fret_core::WindowAnchor;

mod app_handler;
mod diag_bundle_screenshots;
#[cfg(feature = "diag-screenshots")]
mod diag_screenshots;
mod dispatcher;
#[cfg(target_os = "macos")]
mod macos_menu;
mod no_services;
mod renderdoc_capture;
#[cfg(windows)]
mod windows_menu;

use super::streaming_upload::StreamingUploadQueue;
use diag_bundle_screenshots::DiagBundleScreenshotCapture;
use dispatcher::DesktopDispatcher;
use no_services::NoUiServices;
use renderdoc_capture::RenderDocCapture;

#[cfg(windows)]
static WINDOWS_IME_MSG_HOOK_ENABLED: AtomicBool = AtomicBool::new(true);

#[cfg(windows)]
pub fn windows_msg_hook(msg: *const std::ffi::c_void) -> bool {
    if WINDOWS_IME_MSG_HOOK_ENABLED.load(Ordering::Relaxed) {
        fret_runner_winit::windows_ime::msg_hook(msg);
    }
    windows_menu::msg_hook(msg)
}

#[derive(Debug, Clone)]
pub enum RunnerUserEvent {
    PlatformCompletion {
        window: fret_core::AppWindowId,
        completion: PlatformCompletion,
    },
    #[cfg(windows)]
    WindowsMenuCommand {
        window: fret_core::AppWindowId,
        command: fret_runtime::CommandId,
    },
    #[cfg(target_os = "macos")]
    MacosMenuCommand {
        window: Option<fret_core::AppWindowId>,
        command: fret_runtime::CommandId,
    },
    #[cfg(target_os = "macos")]
    MacosMenuWillOpen,
}

#[cfg(feature = "hotpatch-subsecond")]
use hotpatch::{HotpatchRequestKind, HotpatchTrigger, hotpatch_trigger_from_env};

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
    crate::configure_stacksafe_from_env();
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
                    use winit::platform::windows::EventLoopBuilderExtWindows as _;
                    WINDOWS_IME_MSG_HOOK_ENABLED
                        .store(windows_ime_msg_hook_enabled, Ordering::Relaxed);
                    builder.with_msg_hook(windows_msg_hook);
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

#[cfg(target_os = "macos")]
fn macos_dockfloating_parenting_enabled() -> bool {
    use std::sync::OnceLock;

    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("FRET_MACOS_DOCKFLOAT_PARENT").is_some_and(|v| !v.is_empty())
    })
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
fn macos_cursor_trace_enabled() -> bool {
    use std::sync::OnceLock;

    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED
        .get_or_init(|| std::env::var_os("FRET_MACOS_CURSOR_TRACE").is_some_and(|v| !v.is_empty()))
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
fn macos_mouse_location() -> Option<cocoa::foundation::NSPoint> {
    use cocoa::foundation::NSPoint;
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};
    unsafe {
        let Some(class) = Class::get("NSEvent") else {
            return None;
        };
        let point: NSPoint = msg_send![class, mouseLocation];
        Some(point)
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
#[derive(Clone, Copy, Debug, Default)]
struct MacCursorTransform {
    scale_factor: f64,
    x_offset: f64,
    y_offset: f64,
    y_flipped: Option<bool>,
    last_winit_y: Option<f64>,
    last_cocoa_y: Option<f64>,
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct MacCursorScreenKey {
    origin_x: i32,
    origin_y: i32,
    width: i32,
    height: i32,
    scale_milli: i32,
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
impl MacCursorScreenKey {
    fn from_frame(frame: cocoa::foundation::NSRect, scale_factor: f64) -> Self {
        Self {
            origin_x: frame.origin.x.round() as i32,
            origin_y: frame.origin.y.round() as i32,
            width: frame.size.width.round() as i32,
            height: frame.size.height.round() as i32,
            scale_milli: (scale_factor * 1000.0).round() as i32,
        }
    }

    fn unknown(scale_factor: f64) -> Self {
        Self {
            origin_x: 0,
            origin_y: 0,
            width: 0,
            height: 0,
            scale_milli: (scale_factor * 1000.0).round() as i32,
        }
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn macos_screen_key_for_point(point: cocoa::foundation::NSPoint) -> Option<MacCursorScreenKey> {
    use cocoa::base::id;
    use cocoa::foundation::NSRect;
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let Some(class) = Class::get("NSScreen") else {
            return None;
        };
        let screens: id = msg_send![class, screens];
        if screens.is_null() {
            return None;
        }
        let count: usize = msg_send![screens, count];
        for idx in 0..count {
            let screen: id = msg_send![screens, objectAtIndex: idx];
            if screen.is_null() {
                continue;
            }
            let frame: NSRect = msg_send![screen, frame];
            let min_x = frame.origin.x;
            let min_y = frame.origin.y;
            let max_x = min_x + frame.size.width;
            let max_y = min_y + frame.size.height;
            if point.x >= min_x && point.x < max_x && point.y >= min_y && point.y < max_y {
                let scale_factor: f64 = msg_send![screen, backingScaleFactor];
                return Some(MacCursorScreenKey::from_frame(frame, scale_factor));
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
impl MacCursorTransform {
    fn update_from_sample(
        &mut self,
        winit_screen_pos: PhysicalPosition<f64>,
        cocoa_mouse_location: cocoa::foundation::NSPoint,
        scale_factor: f64,
    ) {
        let cocoa_x = cocoa_mouse_location.x * scale_factor;
        let cocoa_y = cocoa_mouse_location.y * scale_factor;

        if self.y_flipped.is_none()
            && let (Some(prev_winit_y), Some(prev_cocoa_y)) = (self.last_winit_y, self.last_cocoa_y)
        {
            let dy_winit = winit_screen_pos.y - prev_winit_y;
            let dy_cocoa = cocoa_y - prev_cocoa_y;
            if dy_winit.abs() > 0.5 && dy_cocoa.abs() > 0.5 {
                self.y_flipped = Some(dy_winit * dy_cocoa < 0.0);
            }
        }

        self.last_winit_y = Some(winit_screen_pos.y);
        self.last_cocoa_y = Some(cocoa_y);

        self.scale_factor = scale_factor;
        self.x_offset = winit_screen_pos.x - cocoa_x;

        let y_flipped = self.y_flipped.unwrap_or(true);
        self.y_offset = if y_flipped {
            winit_screen_pos.y + cocoa_y
        } else {
            winit_screen_pos.y - cocoa_y
        };

        if macos_cursor_trace_enabled() {
            dock_tearoff_log(format_args!(
                "[cursor-calibrate] winit=({:.1},{:.1}) cocoa=({:.1},{:.1}) scale={:.3} flipped={:?} x_off={:.1} y_off={:.1}",
                winit_screen_pos.x,
                winit_screen_pos.y,
                cocoa_x,
                cocoa_y,
                self.scale_factor,
                self.y_flipped,
                self.x_offset,
                self.y_offset,
            ));
        }
    }

    fn map(&self, cocoa_mouse_location: cocoa::foundation::NSPoint) -> PhysicalPosition<f64> {
        let cocoa_x = cocoa_mouse_location.x * self.scale_factor;
        let cocoa_y = cocoa_mouse_location.y * self.scale_factor;
        let x = cocoa_x + self.x_offset;
        let y = if self.y_flipped.unwrap_or(true) {
            self.y_offset - cocoa_y
        } else {
            cocoa_y + self.y_offset
        };
        let out = PhysicalPosition::new(x, y);
        if macos_cursor_trace_enabled() {
            dock_tearoff_log(format_args!(
                "[cursor-map] cocoa=({:.1},{:.1}) scale={:.3} flipped={:?} out=({:.1},{:.1})",
                cocoa_x, cocoa_y, self.scale_factor, self.y_flipped, out.x, out.y
            ));
        }
        out
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
#[derive(Default)]
struct MacCursorTransformTable {
    by_screen: HashMap<MacCursorScreenKey, MacCursorTransform>,
    last_used: Option<MacCursorScreenKey>,
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
impl MacCursorTransformTable {
    fn update_from_window_sample(
        &mut self,
        winit_screen_pos: PhysicalPosition<f64>,
        cocoa_pos: cocoa::foundation::NSPoint,
        scale_factor: f64,
    ) {
        let key = macos_screen_key_for_point(cocoa_pos).unwrap_or_else(|| {
            // If we can't resolve the screen (AppKit oddities), still store a transform so we can
            // map `NSEvent::mouseLocation` during cross-window drags without integrating deltas.
            MacCursorScreenKey::unknown(scale_factor)
        });
        let transform = self
            .by_screen
            .entry(key)
            .or_insert_with(MacCursorTransform::default);
        transform.update_from_sample(winit_screen_pos, cocoa_pos, scale_factor);
        self.last_used = Some(key);
    }

    fn map_with_key_hint(
        &mut self,
        cocoa_pos: cocoa::foundation::NSPoint,
        key_hint: Option<MacCursorScreenKey>,
    ) -> Option<PhysicalPosition<f64>> {
        let hint_hit = key_hint.is_some_and(|k| self.by_screen.contains_key(&k));
        let last_hit = self
            .last_used
            .is_some_and(|k| self.by_screen.contains_key(&k));
        let selection = if hint_hit {
            "key"
        } else if last_hit {
            "last"
        } else {
            "any"
        };

        let transform = key_hint
            .and_then(|k| self.by_screen.get(&k).copied())
            .or_else(|| self.last_used.and_then(|k| self.by_screen.get(&k).copied()))
            .or_else(|| self.by_screen.values().next().copied())?;

        let out = transform.map(cocoa_pos);

        if macos_cursor_trace_enabled() {
            dock_tearoff_log(format_args!(
                "[cursor-refresh] cocoa=({:.1},{:.1}) selection={} key={:?} last={:?} transforms={}",
                cocoa_pos.x,
                cocoa_pos.y,
                selection,
                key_hint,
                self.last_used,
                self.by_screen.len(),
            ));
        }

        if let Some(key) = key_hint {
            self.last_used = Some(key);
        }

        Some(out)
    }

    fn map(&mut self, cocoa_pos: cocoa::foundation::NSPoint) -> Option<PhysicalPosition<f64>> {
        self.map_with_key_hint(cocoa_pos, macos_screen_key_for_point(cocoa_pos))
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn bring_window_to_front(window: &dyn Window, sender: Option<&dyn Window>) -> bool {
    use cocoa::{
        appkit::{NSApp, NSApplication, NSWindow},
        base::{id, nil},
    };
    use objc::runtime::YES;
    use objc::{msg_send, sel, sel_impl};
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    unsafe fn ns_window_id(window: &dyn Window) -> Option<id> {
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
    #[cfg(windows)]
    os_menu: Option<windows_menu::WindowsMenuBar>,
}

#[derive(Debug, Clone)]
struct PendingFrontRequest {
    source_window: Option<fret_core::AppWindowId>,
    panel: Option<fret_core::PanelKey>,
    created_at: Instant,
    next_attempt_at: Instant,
    attempts_left: u8,
}

pub struct WinitRunner<D: WinitAppDriver> {
    pub config: WinitRunnerConfig,
    pub app: App,
    pub driver: D,
    dispatcher: DesktopDispatcher,
    event_loop_proxy: Option<EventLoopProxy>,
    proxy_events: Arc<Mutex<Vec<RunnerUserEvent>>>,

    renderdoc: Option<RenderDocCapture>,
    context: Option<WgpuContext>,
    renderer: Option<Renderer>,
    renderer_caps: Option<fret_render::RendererCapabilities>,
    no_services: NoUiServices,
    diag_bundle_screenshots: DiagBundleScreenshotCapture,

    windows: SlotMap<fret_core::AppWindowId, WindowRuntime<D::WindowState>>,
    window_registry: fret_runner_winit::window_registry::WinitWindowRegistry,
    main_window: Option<fret_core::AppWindowId>,
    menu_bar: Option<fret_runtime::MenuBar>,
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
    #[cfg(target_os = "macos")]
    macos_cursor_transform: MacCursorTransformTable,
    internal_drag_hover_window: Option<fret_core::AppWindowId>,
    internal_drag_hover_pos: Option<Point>,
    internal_drag_pointer_id: Option<fret_core::PointerId>,

    external_drop: NativeExternalDrop,

    uploaded_images: HashMap<fret_core::ImageId, UploadedImageEntry>,
    streaming_uploads: StreamingUploadQueue,
    nv12_gpu: Option<super::yuv_gpu::Nv12GpuConverter>,

    #[cfg(feature = "hotpatch-subsecond")]
    hotpatch: Option<HotpatchTrigger>,
    #[cfg(feature = "hotpatch-subsecond")]
    hot_reload_generation: u64,

    #[cfg(feature = "diag-screenshots")]
    diag_screenshots: Option<diag_screenshots::DiagScreenshotCapture>,
}

struct UploadedImageEntry {
    uploaded: UploadedRgba8Image,
    stream_generation: u64,
    alpha_mode: fret_core::AlphaMode,
    nv12_planes: Option<super::yuv_gpu::Nv12Planes>,
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

struct StreamingImageUpdateRgba8<'a> {
    window: Option<fret_core::AppWindowId>,
    token: fret_core::ImageUpdateToken,
    image: fret_core::ImageId,
    stream_generation: u64,
    width: u32,
    height: u32,
    update_rect_px: Option<fret_core::RectPx>,
    bytes_per_row: u32,
    bytes: &'a [u8],
    color_info: fret_core::ImageColorInfo,
    alpha_mode: fret_core::AlphaMode,
}

struct StreamingImageUpdateNv12<'a> {
    window: Option<fret_core::AppWindowId>,
    token: fret_core::ImageUpdateToken,
    image: fret_core::ImageId,
    stream_generation: u64,
    width: u32,
    height: u32,
    update_rect_px: Option<fret_core::RectPx>,
    y_bytes_per_row: u32,
    y_plane: &'a [u8],
    uv_bytes_per_row: u32,
    uv_plane: &'a [u8],
    color_info: fret_core::ImageColorInfo,
}

impl<D: WinitAppDriver> WinitRunner<D> {
    const WINDOW_VISIBILITY_PADDING_PX: f64 = 40.0;

    #[cfg(target_os = "macos")]
    fn macos_bootstrap_cursor_transform_from_active_drag(&mut self) -> bool {
        let Some(pointer_id) = self.dock_drag_pointer_id() else {
            return false;
        };
        let Some(drag) = self.app.drag(pointer_id) else {
            return false;
        };
        let window = drag.current_window;
        let Some(screen_pos) = self.cursor_screen_pos_fallback_for_window(window) else {
            return false;
        };
        let scale_factor = self
            .windows
            .get(window)
            .map(|s| s.window.scale_factor())
            .unwrap_or(1.0);
        self.macos_calibrate_cursor_transform_from_window_sample(screen_pos, scale_factor);
        true
    }

    #[cfg(target_os = "macos")]
    fn macos_refresh_cursor_screen_pos_for_dock_drag(&mut self) {
        if self.dock_drag_pointer_id().is_none() && self.dock_tearoff_follow.is_none() {
            return;
        }
        if self.macos_refresh_cursor_screen_pos_from_nsevent() {
            return;
        }
        if self.macos_bootstrap_cursor_transform_from_active_drag() {
            let _ = self.macos_refresh_cursor_screen_pos_from_nsevent();
        }
    }

    #[cfg(target_os = "macos")]
    fn macos_calibrate_cursor_transform_from_window_sample(
        &mut self,
        winit_screen_pos: PhysicalPosition<f64>,
        scale_factor: f64,
    ) {
        let Some(cocoa_pos) = macos_mouse_location() else {
            return;
        };
        self.macos_cursor_transform.update_from_window_sample(
            winit_screen_pos,
            cocoa_pos,
            scale_factor,
        );
    }

    #[cfg(target_os = "macos")]
    fn macos_refresh_cursor_screen_pos_from_nsevent(&mut self) -> bool {
        let Some(cocoa_pos) = macos_mouse_location() else {
            return false;
        };
        let Some(mapped) = self.macos_cursor_transform.map(cocoa_pos) else {
            return false;
        };
        self.cursor_screen_pos = Some(mapped);
        true
    }

    fn init_renderdoc_if_needed(&mut self) {
        if self.renderdoc.is_some() {
            return;
        }

        let enabled = std::env::var_os("FRET_RENDERDOC")
            .filter(|v| !v.is_empty())
            .is_some()
            || std::env::var_os("FRET_RENDERDOC_DLL")
                .filter(|v| !v.is_empty())
                .is_some();

        if !enabled {
            return;
        }

        self.renderdoc = RenderDocCapture::try_init();
        if self.renderdoc.is_some() {
            tracing::info!("renderdoc capture enabled");
        } else {
            tracing::warn!(
                "renderdoc capture requested but renderdoc API is unavailable (set FRET_RENDERDOC_DLL to renderdoc.dll path)"
            );
        }
    }

    fn apply_streaming_image_update_rgba8(
        &mut self,
        stats: &mut super::streaming_upload::StreamingUploadStats,
        update: StreamingImageUpdateRgba8<'_>,
    ) {
        let StreamingImageUpdateRgba8 {
            window,
            token,
            image,
            stream_generation,
            width,
            height,
            update_rect_px,
            bytes_per_row,
            bytes,
            color_info,
            alpha_mode,
        } = update;

        let Some(context) = self.context.as_ref() else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::RendererNotReady,
                        },
                    );
                }
            }
            return;
        };
        let Some(renderer) = self.renderer.as_mut() else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::RendererNotReady,
                        },
                    );
                }
            }
            return;
        };
        let Some(entry) = self.uploaded_images.get_mut(&image) else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::UnknownImage,
                        },
                    );
                }
            }
            return;
        };

        if width == 0 || height == 0 {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return;
        }

        if stream_generation < entry.stream_generation {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::Coalesced,
                        },
                    );
                }
            }
            return;
        }
        entry.stream_generation = stream_generation;

        let rect = update_rect_px.unwrap_or_else(|| fret_core::RectPx::full(width, height));
        if rect.is_empty() {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return;
        }

        if rect.x > width
            || rect.y > height
            || rect.x.saturating_add(rect.w) > width
            || rect.y.saturating_add(rect.h) > height
        {
            tracing::warn!(
                image = ?image,
                width,
                height,
                rect = ?rect,
                "ignoring ImageUpdateRgba8 with out-of-bounds update rect"
            );
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return;
        }

        let color_space = match color_info.encoding {
            fret_core::ImageEncoding::Srgb => fret_render::ImageColorSpace::Srgb,
            fret_core::ImageEncoding::Linear => fret_render::ImageColorSpace::Linear,
        };

        let row_bytes = rect.w.saturating_mul(4);
        if bytes_per_row < row_bytes {
            tracing::warn!(
                image = ?image,
                bytes_per_row,
                row_bytes,
                "ignoring ImageUpdateRgba8 with undersized bytes_per_row"
            );
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return;
        }

        let expected_len = (bytes_per_row as usize).saturating_mul(rect.h as usize);
        if bytes.len() != expected_len {
            tracing::warn!(
                image = ?image,
                got = bytes.len(),
                expected = expected_len,
                "ignoring ImageUpdateRgba8 with invalid byte length"
            );
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return;
        }

        if entry.alpha_mode != alpha_mode {
            if !renderer.update_image(
                image,
                fret_render::ImageDescriptor {
                    view: entry.uploaded.view.clone(),
                    size: entry.uploaded.size,
                    format: entry.uploaded.format,
                    color_space: entry.uploaded.color_space,
                    alpha_mode,
                },
            ) {
                self.uploaded_images.remove(&image);
                if self.config.streaming_update_ack_enabled {
                    let target = window
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    if let Some(target) = target {
                        self.deliver_window_event_now(
                            target,
                            &Event::ImageUpdateDropped {
                                token,
                                image,
                                reason: fret_core::ImageUpdateDropReason::UnknownImage,
                            },
                        );
                    }
                }
                return;
            }
            entry.alpha_mode = alpha_mode;
        }

        let needs_replace =
            entry.uploaded.size != (width, height) || entry.uploaded.color_space != color_space;
        let applied_upload_bytes = if needs_replace {
            let is_full_update = rect.x == 0 && rect.y == 0 && rect.w == width && rect.h == height;
            if !is_full_update {
                tracing::warn!(
                    image = ?image,
                    old_size = ?entry.uploaded.size,
                    new_size = ?(width, height),
                    "ignoring partial ImageUpdateRgba8 while image storage needs replace"
                );
                if self.config.streaming_update_ack_enabled {
                    let target = window
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    if let Some(target) = target {
                        self.deliver_window_event_now(
                            target,
                            &Event::ImageUpdateDropped {
                                token,
                                image,
                                reason: fret_core::ImageUpdateDropReason::Unsupported,
                            },
                        );
                    }
                }
                return;
            }

            let (applied_upload_bytes, uploaded) = if bytes_per_row == width.saturating_mul(4)
                && bytes.len()
                    == (width as usize)
                        .saturating_mul(height as usize)
                        .saturating_mul(4)
            {
                (
                    super::streaming_upload::estimate_rgba8_upload_bytes_for_rect(
                        fret_core::RectPx::full(width, height),
                        width.saturating_mul(4),
                    ),
                    fret_render::upload_rgba8_image(
                        &context.device,
                        &context.queue,
                        (width, height),
                        bytes,
                        color_space,
                    ),
                )
            } else {
                let uploaded = fret_render::create_rgba8_image_storage(
                    &context.device,
                    (width, height),
                    color_space,
                );
                uploaded.write_region(
                    &context.queue,
                    (0, 0),
                    (width, height),
                    bytes_per_row,
                    bytes,
                );
                (
                    super::streaming_upload::estimate_rgba8_upload_bytes_for_rect(
                        fret_core::RectPx::full(width, height),
                        bytes_per_row,
                    ),
                    uploaded,
                )
            };

            let view = uploaded
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            if !renderer.update_image(
                image,
                fret_render::ImageDescriptor {
                    view,
                    size: uploaded.size,
                    format: uploaded.format,
                    color_space: uploaded.color_space,
                    alpha_mode,
                },
            ) {
                self.uploaded_images.remove(&image);
                if self.config.streaming_update_ack_enabled {
                    let target = window
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    if let Some(target) = target {
                        self.deliver_window_event_now(
                            target,
                            &Event::ImageUpdateDropped {
                                token,
                                image,
                                reason: fret_core::ImageUpdateDropReason::UnknownImage,
                            },
                        );
                    }
                }
                return;
            }
            entry.uploaded = uploaded;
            entry.alpha_mode = alpha_mode;
            entry.nv12_planes = None;
            applied_upload_bytes
        } else {
            entry.uploaded.write_region(
                &context.queue,
                (rect.x, rect.y),
                (rect.w, rect.h),
                bytes_per_row,
                bytes,
            );
            super::streaming_upload::estimate_rgba8_upload_bytes_for_rect(rect, bytes_per_row)
        };
        stats.upload_bytes_applied = stats
            .upload_bytes_applied
            .saturating_add(applied_upload_bytes);

        if self.config.streaming_update_ack_enabled {
            let target = window
                .or(self.main_window)
                .or_else(|| self.windows.keys().next());
            if let Some(target) = target {
                self.deliver_window_event_now(target, &Event::ImageUpdateApplied { token, image });
            }
        }

        if let Some(state) = window.and_then(|w| self.windows.get(w)) {
            state.window.request_redraw();
        } else {
            for (_id, state) in self.windows.iter() {
                state.window.request_redraw();
            }
        }
    }

    fn try_apply_streaming_image_update_nv12_gpu(
        &mut self,
        stats: &mut super::streaming_upload::StreamingUploadStats,
        update: StreamingImageUpdateNv12<'_>,
    ) -> bool {
        let StreamingImageUpdateNv12 {
            window,
            token,
            image,
            stream_generation,
            width,
            height,
            update_rect_px,
            y_bytes_per_row,
            y_plane,
            uv_bytes_per_row,
            uv_plane,
            color_info,
        } = update;

        let requested = self.config.streaming_nv12_gpu_convert_enabled
            || std::env::var_os("FRET_STREAMING_GPU_YUV").is_some_and(|v| !v.is_empty());
        if !requested {
            return false;
        }

        let supported = self
            .renderer_caps
            .as_ref()
            .is_some_and(|c| c.streaming_images.nv12_gpu_convert);
        if !supported {
            return false;
        }

        if width == 0 || height == 0 {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return true;
        }

        let Some(context) = self.context.as_ref() else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::RendererNotReady,
                        },
                    );
                }
            }
            return true;
        };
        let Some(renderer) = self.renderer.as_mut() else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::RendererNotReady,
                        },
                    );
                }
            }
            return true;
        };

        let Some(entry) = self.uploaded_images.get_mut(&image) else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::UnknownImage,
                        },
                    );
                }
            }
            return true;
        };

        if stream_generation < entry.stream_generation {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::Coalesced,
                        },
                    );
                }
            }
            return true;
        }
        entry.stream_generation = stream_generation;

        let Ok(rect) = super::yuv::normalize_update_rect_420(width, height, update_rect_px) else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return true;
        };

        let color_space = match color_info.encoding {
            fret_core::ImageEncoding::Srgb => fret_render::ImageColorSpace::Srgb,
            fret_core::ImageEncoding::Linear => fret_render::ImageColorSpace::Linear,
        };

        if entry.uploaded.format != wgpu::TextureFormat::Rgba8UnormSrgb {
            return false;
        }
        if color_space != fret_render::ImageColorSpace::Srgb {
            return false;
        }

        let is_full_update = rect.x == 0 && rect.y == 0 && rect.w == width && rect.h == height;
        let needs_replace =
            entry.uploaded.size != (width, height) || entry.uploaded.color_space != color_space;
        if needs_replace {
            if !is_full_update {
                if self.config.streaming_update_ack_enabled {
                    let target = window
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    if let Some(target) = target {
                        self.deliver_window_event_now(
                            target,
                            &Event::ImageUpdateDropped {
                                token,
                                image,
                                reason: fret_core::ImageUpdateDropReason::Unsupported,
                            },
                        );
                    }
                }
                return true;
            }

            let uploaded = fret_render::create_rgba8_image_storage(
                &context.device,
                (width, height),
                color_space,
            );
            let view = uploaded
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            if !renderer.update_image(
                image,
                fret_render::ImageDescriptor {
                    view,
                    size: uploaded.size,
                    format: uploaded.format,
                    color_space: uploaded.color_space,
                    alpha_mode: fret_core::AlphaMode::Opaque,
                },
            ) {
                self.uploaded_images.remove(&image);
                if self.config.streaming_update_ack_enabled {
                    let target = window
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    if let Some(target) = target {
                        self.deliver_window_event_now(
                            target,
                            &Event::ImageUpdateDropped {
                                token,
                                image,
                                reason: fret_core::ImageUpdateDropReason::UnknownImage,
                            },
                        );
                    }
                }
                return true;
            }
            entry.uploaded = uploaded;
            entry.alpha_mode = fret_core::AlphaMode::Opaque;
            entry.nv12_planes = None;
        }

        if entry.alpha_mode != fret_core::AlphaMode::Opaque {
            if !renderer.update_image(
                image,
                fret_render::ImageDescriptor {
                    view: entry.uploaded.view.clone(),
                    size: entry.uploaded.size,
                    format: entry.uploaded.format,
                    color_space: entry.uploaded.color_space,
                    alpha_mode: fret_core::AlphaMode::Opaque,
                },
            ) {
                self.uploaded_images.remove(&image);
                if self.config.streaming_update_ack_enabled {
                    let target = window
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    if let Some(target) = target {
                        self.deliver_window_event_now(
                            target,
                            &Event::ImageUpdateDropped {
                                token,
                                image,
                                reason: fret_core::ImageUpdateDropReason::UnknownImage,
                            },
                        );
                    }
                }
                return true;
            }
            entry.alpha_mode = fret_core::AlphaMode::Opaque;
        }

        if entry
            .nv12_planes
            .as_ref()
            .is_none_or(|p| p.size != (width, height))
        {
            entry.nv12_planes = Some(super::yuv_gpu::Nv12Planes::new(
                &context.device,
                (width, height),
            ));
        }
        if self.nv12_gpu.is_none() {
            self.nv12_gpu = Some(super::yuv_gpu::Nv12GpuConverter::new(&context.device));
        }

        let Some(planes) = entry.nv12_planes.as_ref() else {
            return false;
        };
        let Some(converter) = self.nv12_gpu.as_ref() else {
            return false;
        };

        let t0 = std::time::Instant::now();
        let Ok(uploaded_bytes) = super::yuv_gpu::write_nv12_rect(
            &context.queue,
            planes,
            rect,
            y_bytes_per_row,
            y_plane,
            uv_bytes_per_row,
            uv_plane,
        ) else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return true;
        };

        stats.upload_bytes_applied = stats.upload_bytes_applied.saturating_add(uploaded_bytes);

        converter.convert_rect_into(super::yuv_gpu::Nv12ConvertRectIntoArgs {
            device: &context.device,
            queue: &context.queue,
            dst_view: &entry.uploaded.view,
            rect,
            y_view: &planes.y_view,
            uv_view: &planes.uv_view,
            range: color_info.range,
            matrix: color_info.matrix,
        });

        stats.yuv_conversions_applied = stats.yuv_conversions_applied.saturating_add(1);
        stats.yuv_convert_us = stats
            .yuv_convert_us
            .saturating_add(t0.elapsed().as_micros() as u64);
        stats.yuv_convert_output_bytes = stats
            .yuv_convert_output_bytes
            .saturating_add(rect.w.saturating_mul(rect.h).saturating_mul(4) as u64);

        if self.config.streaming_update_ack_enabled {
            let target = window
                .or(self.main_window)
                .or_else(|| self.windows.keys().next());
            if let Some(target) = target {
                self.deliver_window_event_now(target, &Event::ImageUpdateApplied { token, image });
            }
        }

        if let Some(state) = window.and_then(|w| self.windows.get(w)) {
            state.window.request_redraw();
        } else {
            for (_id, state) in self.windows.iter() {
                state.window.request_redraw();
            }
        }

        true
    }

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

        let dispatcher = DesktopDispatcher::new(caps.exec);
        app.set_global::<fret_runtime::DispatcherHandle>(dispatcher.handle());

        #[cfg(feature = "hotpatch-subsecond")]
        let now = Instant::now();
        Self {
            config,
            app,
            driver,
            dispatcher,
            event_loop_proxy: None,
            proxy_events: Arc::new(Mutex::new(Vec::new())),
            renderdoc: None,
            context: None,
            renderer: None,
            renderer_caps: None,
            no_services: NoUiServices,
            diag_bundle_screenshots: DiagBundleScreenshotCapture::from_env(),
            windows: SlotMap::with_key(),
            window_registry: fret_runner_winit::window_registry::WinitWindowRegistry::default(),
            main_window: None,
            menu_bar: None,
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
            #[cfg(target_os = "macos")]
            macos_cursor_transform: MacCursorTransformTable::default(),
            internal_drag_hover_window: None,
            internal_drag_hover_pos: None,
            internal_drag_pointer_id: None,
            external_drop: NativeExternalDrop::default(),
            uploaded_images: HashMap::new(),
            streaming_uploads: StreamingUploadQueue::default(),
            nv12_gpu: None,
            #[cfg(feature = "hotpatch-subsecond")]
            hotpatch: hotpatch_trigger_from_env(now),
            #[cfg(feature = "hotpatch-subsecond")]
            hot_reload_generation: 0,

            #[cfg(feature = "diag-screenshots")]
            diag_screenshots: diag_screenshots::DiagScreenshotCapture::from_env(),
        }
    }

    fn backend_platform_capabilities(_config: &WinitRunnerConfig) -> PlatformCapabilities {
        let mut caps = PlatformCapabilities::default();

        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            caps.exec.background_work = fret_runtime::ExecBackgroundWork::Threads;
            caps.exec.wake = fret_runtime::ExecWake::Reliable;
            caps.exec.timers = fret_runtime::ExecTimers::Reliable;

            caps.ui.multi_window = true;
            caps.ui.window_tear_off = true;
            caps.ui.cursor_icons = true;

            #[cfg(any(target_os = "windows", target_os = "macos"))]
            {
                caps.ui.window_hover_detection =
                    fret_runtime::WindowHoverDetectionQuality::Reliable;
                caps.ui.window_set_outer_position =
                    fret_runtime::WindowSetOuterPositionQuality::Reliable;
                caps.ui.window_z_level = fret_runtime::WindowZLevelQuality::Reliable;
            }

            #[cfg(target_os = "linux")]
            {
                // Linux windowing behavior varies significantly across X11/Wayland and
                // compositors. Default to best-effort until we add backend-specific detection.
                caps.ui.window_hover_detection =
                    fret_runtime::WindowHoverDetectionQuality::BestEffort;
                caps.ui.window_set_outer_position =
                    fret_runtime::WindowSetOuterPositionQuality::BestEffort;
                caps.ui.window_z_level = fret_runtime::WindowZLevelQuality::BestEffort;

                // Wayland compositors do not provide a reliable "window under cursor" contract and
                // may ignore programmatic window positioning/z-level hints. Prefer a predictable
                // in-window floating fallback over OS tear-off UX (ADR 0054 / ADR 0084).
                if linux_is_wayland_session() {
                    caps.ui.window_tear_off = false;
                    caps.ui.window_hover_detection =
                        fret_runtime::WindowHoverDetectionQuality::None;
                    caps.ui.window_z_level = fret_runtime::WindowZLevelQuality::None;
                }
            }

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
            caps.exec.background_work = fret_runtime::ExecBackgroundWork::Cooperative;
            caps.exec.wake = fret_runtime::ExecWake::BestEffort;
            caps.exec.timers = fret_runtime::ExecTimers::BestEffort;

            caps.ui.multi_window = false;
            caps.ui.window_tear_off = false;
            caps.ui.cursor_icons = false;
            caps.ui.window_hover_detection = fret_runtime::WindowHoverDetectionQuality::None;
            caps.ui.window_set_outer_position = fret_runtime::WindowSetOuterPositionQuality::None;
            caps.ui.window_z_level = fret_runtime::WindowZLevelQuality::None;

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
            caps.exec.background_work = fret_runtime::ExecBackgroundWork::Threads;
            caps.exec.wake = fret_runtime::ExecWake::Reliable;
            caps.exec.timers = fret_runtime::ExecTimers::Reliable;

            caps.ui.multi_window = false;
            caps.ui.window_tear_off = false;
            caps.ui.cursor_icons = false;
            caps.ui.window_hover_detection = fret_runtime::WindowHoverDetectionQuality::None;
            caps.ui.window_set_outer_position = fret_runtime::WindowSetOuterPositionQuality::None;
            caps.ui.window_z_level = fret_runtime::WindowZLevelQuality::None;

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

        caps.exec.background_work = caps
            .exec
            .background_work
            .clamp_to_available(available.exec.background_work);
        caps.exec.wake = caps.exec.wake.clamp_to_available(available.exec.wake);
        caps.exec.timers = caps.exec.timers.clamp_to_available(available.exec.timers);

        caps.ui.multi_window &= available.ui.multi_window;
        caps.ui.window_tear_off &= available.ui.window_tear_off;
        caps.ui.cursor_icons &= available.ui.cursor_icons;
        caps.ui.window_hover_detection = caps
            .ui
            .window_hover_detection
            .clamp_to_available(available.ui.window_hover_detection);
        caps.ui.window_set_outer_position = caps
            .ui
            .window_set_outer_position
            .clamp_to_available(available.ui.window_set_outer_position);
        caps.ui.window_z_level = caps
            .ui
            .window_z_level
            .clamp_to_available(available.ui.window_z_level);

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
        #[cfg(feature = "hotpatch-subsecond")]
        if let Some(hotpatch) = self.hotpatch.as_ref() {
            hotpatch.set_event_loop_proxy(proxy.clone());
        }
        #[cfg(windows)]
        windows_menu::set_event_loop_proxy(proxy.clone(), self.proxy_events.clone());
        #[cfg(target_os = "macos")]
        macos_menu::set_event_loop_proxy(proxy.clone(), self.proxy_events.clone());
        self.dispatcher.set_event_loop_proxy(proxy.clone());
        self.event_loop_proxy = Some(proxy);
    }

    fn spawn_platform_completion_task<F>(&self, window: fret_core::AppWindowId, task: F) -> bool
    where
        F: FnOnce() -> PlatformCompletion + Send + 'static,
    {
        let Some(_proxy) = self.event_loop_proxy.clone() else {
            return false;
        };
        let events = self.proxy_events.clone();

        let dispatcher = self.dispatcher.handle();
        let wake_dispatcher = dispatcher.clone();
        dispatcher.dispatch_background(
            Box::new(move || {
                let completion = task();
                if let Ok(mut queue) = events.lock() {
                    queue.push(RunnerUserEvent::PlatformCompletion { window, completion });
                }
                wake_dispatcher.wake(Some(window));
            }),
            fret_runtime::DispatchPriority::High,
        );

        true
    }

    fn deliver_window_event_now(&mut self, window: fret_core::AppWindowId, event: &Event) {
        if self.maybe_handle_hotpatch_event(window, event) {
            return;
        }
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        fret_runtime::apply_window_metrics_event(&mut self.app, window, event);
        let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
        self.driver.handle_event(
            WinitEventContext {
                app: &mut self.app,
                services,
                window,
                state: &mut state.user,
            },
            event,
        );
    }

    fn maybe_handle_hotpatch_event(
        &mut self,
        _window: fret_core::AppWindowId,
        _event: &Event,
    ) -> bool {
        #[cfg(feature = "hotpatch-subsecond")]
        {
            if self.hotpatch.is_none() {
                return false;
            }

            let Event::KeyDown {
                key,
                modifiers,
                repeat,
            } = _event
            else {
                return false;
            };
            if *repeat {
                return false;
            }

            let is_reload_chord = *key == fret_core::KeyCode::KeyR
                && modifiers.ctrl
                && modifiers.shift
                && !modifiers.alt
                && !modifiers.alt_gr
                && !modifiers.meta;
            if !is_reload_chord {
                return false;
            }

            self.hot_reload_all_windows("key chord (Ctrl+Shift+R)");
            return true;
        }

        #[cfg(not(feature = "hotpatch-subsecond"))]
        {
            false
        }
    }

    #[cfg(feature = "hotpatch-subsecond")]
    fn hot_reload_all_windows(&mut self, reason: &'static str) {
        self.hot_reload_generation = self.hot_reload_generation.saturating_add(1);
        let generation = self.hot_reload_generation;
        tracing::info!(%reason, generation, "hotpatch: hot reload requested");
        hotpatch::hotpatch_diag_log(&format!(
            "runner: hot_reload_all_windows begin reason={reason} generation={generation}"
        ));

        // Ensure pending queued work does not cross the reload boundary.
        self.dispatcher.hot_reload_boundary();

        // Cancel any in-flight drag to avoid leaving the runner in an inconsistent state.
        {
            use fret_runtime::DragHost as _;
            let _ = self.app.cancel_drag_sessions(|_| true);
        }

        {
            let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
            self.driver.hot_reload_global(&mut self.app, services);
        }

        // Collect first: we need to re-enter `self` mutably when mutating window states.
        let windows: Vec<fret_core::AppWindowId> = self.windows.keys().collect();
        tracing::debug!(
            generation,
            windows = windows.len(),
            "hotpatch: scheduling window resets"
        );
        hotpatch::hotpatch_diag_log(&format!(
            "runner: scheduling window resets generation={generation} windows={}",
            windows.len()
        ));

        for window in windows {
            let Some(state) = self.windows.get_mut(window) else {
                continue;
            };

            let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
            self.driver
                .hot_reload_window(&mut self.app, services, window, &mut state.user);
            tracing::debug!(generation, ?window, "hotpatch: window reset complete");
            hotpatch::hotpatch_diag_log(&format!(
                "runner: window reset complete generation={generation} window={window:?}"
            ));

            state.last_accessibility_snapshot = None;
            state.window.request_redraw();
        }
        hotpatch::hotpatch_diag_log(&format!(
            "runner: hot_reload_all_windows end generation={generation}"
        ));
    }

    fn poll_hotpatch_trigger(&mut self, now: Instant) -> bool {
        #[cfg(feature = "hotpatch-subsecond")]
        {
            let Some(trigger) = self.hotpatch.as_mut() else {
                return false;
            };

            let Some(request) = trigger.poll(now) else {
                return false;
            };

            match request.kind {
                HotpatchRequestKind::SubsecondPatchApplied => {
                    hotpatch::hotpatch_diag_log("runner: observed SubsecondPatchApplied");
                    self.hot_reload_all_windows("subsecond patch applied");
                }
                HotpatchRequestKind::TriggerFileChanged => {
                    if let Some(path) = request.trigger_path.as_ref() {
                        tracing::info!(path = %path.display(), "hotpatch: trigger file changed");
                    }
                    hotpatch::hotpatch_diag_log("runner: observed TriggerFileChanged");
                    self.hot_reload_all_windows("trigger file changed");
                }
            }

            true
        }

        #[cfg(not(feature = "hotpatch-subsecond"))]
        {
            let _ = now;
            false
        }
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
        style: WindowStyleRequest,
        _parent_window: Option<winit::raw_window_handle::RawWindowHandle>,
    ) -> Result<(Arc<dyn Window>, Option<accessibility::WinitAccessibility>), RunnerError> {
        let mut attrs = winit::window::WindowAttributes::default()
            .with_title(spec.title)
            .with_surface_size(spec.size)
            .with_visible(if self.config.accessibility_enabled {
                false
            } else {
                spec.visible
            });
        if let Some(policy) = style.activation {
            let active = matches!(policy, ActivationPolicy::Activates);
            attrs = attrs.with_active(active);
        }
        if let Some(position) = spec.position {
            attrs = attrs.with_position(position);
        }
        #[cfg(windows)]
        {
            if let Some(taskbar) = style.taskbar {
                use winit::platform::windows::WindowAttributesWindows;

                let win = WindowAttributesWindows::default()
                    .with_skip_taskbar(matches!(taskbar, TaskbarVisibility::Hide));
                attrs = attrs.with_platform_attributes(Box::new(win));
            }
        }
        #[cfg(target_os = "macos")]
        if _parent_window.is_some() {
            // macOS tool/aux windows: best-effort parent/child relationship so DockFloating windows
            // follow the parent window's Space/fullscreen lifecycle.
            //
            // winit maps this to `NSWindow.addChildWindow_ordered(...)`.
            attrs = unsafe { attrs.with_parent_window(_parent_window) };
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

        if let Some(level) = style.z_level {
            window.set_window_level(match level {
                WindowZLevel::Normal => WindowLevel::Normal,
                WindowZLevel::AlwaysOnTop => WindowLevel::AlwaysOnTop,
            });
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
        let surface = SurfaceState::new_with_usage(
            &context.adapter,
            &context.device,
            surface,
            size.width,
            size.height,
            self.diag_bundle_screenshots.surface_usage(),
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
                #[cfg(windows)]
                os_menu: None,
            }
        });

        if let Some(state) = self.windows.get(id) {
            let size_phys = state.window.surface_size();
            let size_logical: winit::dpi::LogicalSize<f32> =
                size_phys.to_logical(state.window.scale_factor());
            fret_runtime::apply_window_metrics_event(
                &mut self.app,
                id,
                &Event::WindowResized {
                    width: Px(size_logical.width),
                    height: Px(size_logical.height),
                },
            );
            fret_runtime::apply_window_metrics_event(
                &mut self.app,
                id,
                &Event::WindowScaleFactorChanged(state.window.scale_factor() as f32),
            );
        }

        let winit_id = self.windows[id].window.id();
        self.window_registry.insert(winit_id, id);

        #[cfg(windows)]
        windows_menu::register_window(self.windows[id].window.as_ref(), id);
        #[cfg(target_os = "macos")]
        macos_menu::register_window(self.windows[id].window.as_ref(), id);

        #[cfg(windows)]
        if let Some(menu_bar) = self.menu_bar.as_ref()
            && let Some(state) = self.windows.get_mut(id)
        {
            if let Some(menu) =
                windows_menu::set_window_menu_bar(&self.app, state.window.as_ref(), id, menu_bar)
            {
                state.os_menu = Some(menu);
            }
        }

        // Ensure the window draws at least one frame after creation.
        //
        // Important: `WindowEvent::RedrawRequested` is keyed by the winit `WindowId`, so we must
        // install the `WindowId` -> `AppWindowId` mapping *before* requesting the redraw. Otherwise, the first
        // redraw can be dropped and the window may appear blank until another event arrives.
        if let Some(state) = self.windows.get(id) {
            state.window.request_redraw();
            // `request_redraw()` alone may not wake the event loop on some platforms; schedule a
            // one-shot RAF so the initial frame presents without requiring any user input.
            self.raf_windows.insert(id);
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

    fn close_window(&mut self, window: fret_core::AppWindowId) -> bool {
        self.close_window_impl(window, true)
    }

    fn force_close_window(&mut self, window: fret_core::AppWindowId) -> bool {
        self.close_window_impl(window, false)
    }

    fn close_window_impl(
        &mut self,
        window: fret_core::AppWindowId,
        check_before_close: bool,
    ) -> bool {
        if !self.windows.contains_key(window) {
            return false;
        }

        if check_before_close {
            let should_close = self.driver.before_close_window(&mut self.app, window);
            if !should_close {
                return false;
            }
        }

        if self
            .dock_tearoff_follow
            .is_some_and(|f| f.window == window || f.source_window == window)
        {
            self.stop_dock_tearoff_follow(Instant::now(), false);
        }

        if self.internal_drag_hover_window == Some(window) {
            self.internal_drag_hover_window = None;
            self.internal_drag_hover_pos = None;
            self.internal_drag_pointer_id = None;
        }

        {
            use fret_runtime::DragHost as _;
            use std::collections::HashSet;

            let mut visited: HashSet<fret_core::PointerId> = HashSet::new();
            while let Some(pointer_id) = self.app.find_drag_pointer_id(|d| {
                !visited.contains(&d.pointer_id) && d.source_window == window
            }) {
                visited.insert(pointer_id);
                self.app.cancel_drag(pointer_id);
            }

            let mut visited: HashSet<fret_core::PointerId> = HashSet::new();
            while let Some(pointer_id) = self.app.find_drag_pointer_id(|d| {
                !visited.contains(&d.pointer_id) && d.current_window == window
            }) {
                visited.insert(pointer_id);
                if let Some(drag) = self.app.drag_mut(pointer_id) {
                    drag.current_window = drag.source_window;
                }
            }
        }

        let Some(state) = self.windows.remove(window) else {
            return false;
        };
        #[cfg(windows)]
        windows_menu::unregister_window(state.window.as_ref());
        #[cfg(target_os = "macos")]
        macos_menu::unregister_window(state.window.as_ref());
        self.window_registry.remove(state.window.id());

        self.app.with_global_mut(
            fret_runtime::WindowInputContextService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowCommandAvailabilityService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowCommandEnabledService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowCommandGatingService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowTextInputSnapshotService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app
            .with_global_mut(WindowMetricsService::default, |svc, _app| {
                svc.remove(window);
            });
        if Some(window) == self.main_window {
            self.main_window = None;
        }

        true
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

    fn compute_window_position_from_cursor_grab_estimate(
        &self,
        reference_window: fret_core::AppWindowId,
        new_window_inner_size: winit::dpi::LogicalSize<f64>,
        grab_offset_logical: Point,
    ) -> Option<Position> {
        let screen_pos = self.cursor_screen_pos?;
        let state = self.windows.get(reference_window)?;
        let scale = state.window.scale_factor();

        let max_client = winit::dpi::LogicalSize::new(
            new_window_inner_size.width as f32,
            new_window_inner_size.height as f32,
        );

        let mut x = screen_pos.x;
        let mut y = screen_pos.y;

        if let Some((ox, oy)) = outer_pos_for_cursor_grab(
            screen_pos,
            grab_offset_logical,
            scale,
            state.window.surface_position(),
            Some(max_client),
        ) {
            x = ox;
            y = oy;
        }

        // Best-effort clamping: avoid creating "off-screen" floating windows due to
        // platform-specific coordinate spaces and DPI conversions.
        let outer_size = new_window_inner_size.to_physical::<u32>(scale);

        #[cfg(target_os = "windows")]
        if let Some(work) = win32::monitor_work_area_for_point(screen_pos) {
            (x, y) = Self::clamp_window_outer_pos_to_monitor(
                x,
                y,
                outer_size,
                work,
                Self::WINDOW_VISIBILITY_PADDING_PX,
            );
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
            }
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

        // Clamp the grab point to the target window's current client size. During tear-off, the
        // grab offset comes from the source window's client coordinates; if the new floating
        // window is smaller, keeping the original offset would place the cursor outside the new
        // window (visible as a fixed offset between cursor and window).
        let target_inner = state.window.surface_size();
        let target_inner_logical: winit::dpi::LogicalSize<f32> = target_inner.to_logical(scale);
        let (mut x, mut y) = outer_pos_for_cursor_grab(
            screen_pos,
            grab_offset_logical,
            scale,
            state.window.surface_position(),
            Some(target_inner_logical),
        )?;

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
                if let Some(anchor) = request.anchor {
                    // Initial positioning is best-effort until the OS window exists, but it's
                    // worth approximating with the source window's decoration offset so Windows
                    // doesn't "jump" after creation under mixed DPI / non-client offsets.
                    spec.position = self.compute_window_position_from_cursor_grab_estimate(
                        anchor.window,
                        spec.size,
                        anchor.position,
                    );
                }
                if spec.position.is_none() {
                    spec.position = self.compute_window_position_from_cursor(source_window);
                }
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

        #[cfg(target_os = "macos")]
        let parent_window = {
            use winit::raw_window_handle::HasWindowHandle as _;
            if !macos_dockfloating_parenting_enabled() {
                None
            } else {
                match request.kind {
                    CreateWindowKind::DockFloating { source_window, .. } => self
                        .windows
                        .get(source_window)
                        .and_then(|w| w.window.window_handle().ok())
                        .map(|h| h.as_raw()),
                    _ => None,
                }
            }
        };
        #[cfg(not(target_os = "macos"))]
        let parent_window = None;

        let (window, accessibility) =
            self.create_os_window(event_loop, spec, request.style, parent_window)?;
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
                        WinitEventContext {
                            app: &mut self.app,
                            services,
                            window,
                            state: &mut state.user,
                        },
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
            let mut did_work = self.dispatcher.drain_turn(now);
            did_work |= self.drain_inboxes(None);
            let effects = self.app.flush_effects();
            let (effects, mut stats, acks) = self.streaming_uploads.process_effects(
                self.frame_id,
                effects,
                self.config.streaming_upload_budget_bytes_per_frame,
                self.config.streaming_staging_budget_bytes,
                self.config.streaming_update_ack_enabled,
            );
            tracing::trace!(
                did_work,
                effects = effects.len(),
                acks = acks.len(),
                "driver: drain_effects turn"
            );
            if self.config.streaming_update_ack_enabled {
                for ack in acks {
                    let window = ack
                        .window_hint
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    let Some(window) = window else {
                        continue;
                    };
                    match ack.kind {
                        super::streaming_upload::StreamingUploadAckKind::Dropped(reason) => {
                            self.deliver_window_event_now(
                                window,
                                &Event::ImageUpdateDropped {
                                    token: ack.token,
                                    image: ack.image,
                                    reason,
                                },
                            );
                        }
                    }
                }
            }

            did_work |= self.poll_hotpatch_trigger(now);
            did_work |= !effects.is_empty();
            let mut window_state_dirty: HashSet<fret_core::AppWindowId> = HashSet::new();

            for effect in effects {
                match effect {
                    Effect::Redraw(window) => {
                        if let Some(state) = self.windows.get(window) {
                            state.window.request_redraw();
                            // Some platforms may not wake the event loop for `request_redraw()`
                            // alone; scheduling a one-shot RAF ensures the first frame presents
                            // without requiring any input events.
                            self.raf_windows.insert(window);
                        }
                    }
                    Effect::ImeAllow { window, enabled } => {
                        if let Some(state) = self.windows.get_mut(window)
                            && state.platform.set_ime_allowed(enabled)
                        {
                            window_state_dirty.insert(window);
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
                    Effect::QuitApp => {
                        let prompt_window = self.main_window.or_else(|| self.windows.keys().next());
                        if let Some(window) = prompt_window {
                            if !self.driver.before_close_window(&mut self.app, window) {
                                continue;
                            }
                        }

                        let windows: Vec<fret_core::AppWindowId> = self.windows.keys().collect();
                        for window in windows {
                            let _ = self.force_close_window(window);
                        }

                        self.dispatcher.shutdown();
                        event_loop.exit();
                        return;
                    }
                    Effect::ShowAboutPanel => {
                        #[cfg(target_os = "macos")]
                        {
                            macos_menu::show_about_panel();
                        }
                    }
                    Effect::HideApp => {
                        #[cfg(target_os = "macos")]
                        {
                            macos_menu::hide_app();
                        }
                    }
                    Effect::HideOtherApps => {
                        #[cfg(target_os = "macos")]
                        {
                            macos_menu::hide_other_apps();
                        }
                    }
                    Effect::UnhideAllApps => {
                        #[cfg(target_os = "macos")]
                        {
                            macos_menu::unhide_all_apps();
                        }
                    }
                    Effect::Command { window, command } => match window {
                        Some(window) => {
                            if let Some(state) = self.windows.get_mut(window) {
                                let services = Self::ui_services_mut(
                                    &mut self.renderer,
                                    &mut self.no_services,
                                );
                                self.driver.handle_command(
                                    WinitCommandContext {
                                        app: &mut self.app,
                                        services,
                                        window,
                                        state: &mut state.user,
                                    },
                                    command,
                                );
                            }
                        }
                        None => {
                            let services =
                                Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                            self.driver.handle_global_command(
                                WinitGlobalContext {
                                    app: &mut self.app,
                                    services,
                                },
                                command,
                            );
                        }
                    },
                    Effect::SetMenuBar { window, menu_bar } => {
                        if window.is_none() {
                            self.menu_bar = Some(menu_bar.clone());
                        }
                        #[cfg(windows)]
                        {
                            let targets: Vec<fret_core::AppWindowId> = match window {
                                Some(window) => vec![window],
                                None => self.windows.keys().collect(),
                            };
                            for window in targets {
                                let Some(state) = self.windows.get_mut(window) else {
                                    continue;
                                };
                                let Some(menu) = windows_menu::set_window_menu_bar(
                                    &self.app,
                                    state.window.as_ref(),
                                    window,
                                    &menu_bar,
                                ) else {
                                    continue;
                                };
                                state.os_menu = Some(menu);
                            }
                        }
                        #[cfg(target_os = "macos")]
                        {
                            let _ = window;
                            macos_menu::set_app_menu_bar(&self.app, &menu_bar);
                        }
                        #[cfg(all(not(windows), not(target_os = "macos")))]
                        {
                            let _ = (window, menu_bar);
                        }
                    }
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
                    Effect::PrimarySelectionSetText { text } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.clipboard.primary_text {
                            continue;
                        }
                        if let Err(err) = self.clipboard.set_primary_text(&text) {
                            tracing::debug!(?err, "failed to set primary selection text");
                        }
                    }
                    Effect::PrimarySelectionGetText { window, token } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.clipboard.primary_text {
                            self.deliver_window_event_now(
                                window,
                                &Event::PrimarySelectionTextUnavailable { token },
                            );
                            continue;
                        }

                        match self.clipboard.get_primary_text() {
                            Ok(Some(text)) => self.deliver_window_event_now(
                                window,
                                &Event::PrimarySelectionText { token, text },
                            ),
                            Ok(None) | Err(_) => self.deliver_window_event_now(
                                window,
                                &Event::PrimarySelectionTextUnavailable { token },
                            ),
                        }
                    }
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

                        let _ = fret_runtime::apply_font_catalog_update(
                            &mut self.app,
                            renderer.all_font_names(),
                            fret_runtime::FontFamilyDefaultsPolicy::None,
                        );
                        if let Some(config) = self.app.global::<fret_core::TextFontFamilyConfig>() {
                            let _ = renderer.set_text_font_families(config);
                        }
                        self.app.set_global::<fret_runtime::TextFontStackKey>(
                            fret_runtime::TextFontStackKey(renderer.text_font_stack_key()),
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
                        color_info,
                        alpha_mode,
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

                        let color_space = match color_info.encoding {
                            fret_core::ImageEncoding::Srgb => fret_render::ImageColorSpace::Srgb,
                            fret_core::ImageEncoding::Linear => {
                                fret_render::ImageColorSpace::Linear
                            }
                        };

                        let uploaded = fret_render::upload_rgba8_image(
                            &context.device,
                            &context.queue,
                            (width, height),
                            &bytes,
                            color_space,
                        );

                        let view = uploaded
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        let image = renderer.register_image(fret_render::ImageDescriptor {
                            view,
                            size: uploaded.size,
                            format: uploaded.format,
                            color_space: uploaded.color_space,
                            alpha_mode,
                        });
                        self.uploaded_images.insert(
                            image,
                            UploadedImageEntry {
                                uploaded,
                                stream_generation: 0,
                                alpha_mode,
                                nv12_planes: None,
                            },
                        );

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
                    Effect::ImageUpdateRgba8 {
                        window,
                        token,
                        image,
                        stream_generation,
                        width,
                        height,
                        update_rect_px,
                        bytes_per_row,
                        bytes,
                        color_info,
                        alpha_mode,
                    } => {
                        self.apply_streaming_image_update_rgba8(
                            &mut stats,
                            StreamingImageUpdateRgba8 {
                                window,
                                token,
                                image,
                                stream_generation,
                                width,
                                height,
                                update_rect_px,
                                bytes_per_row,
                                bytes: &bytes,
                                color_info,
                                alpha_mode,
                            },
                        );
                    }
                    Effect::ImageUpdateNv12 {
                        window,
                        token,
                        image,
                        stream_generation,
                        width,
                        height,
                        update_rect_px,
                        y_bytes_per_row,
                        y_plane,
                        uv_bytes_per_row,
                        uv_plane,
                        color_info,
                        alpha_mode: _,
                    } => {
                        stats.yuv_conversions_attempted =
                            stats.yuv_conversions_attempted.saturating_add(1);
                        if self.try_apply_streaming_image_update_nv12_gpu(
                            &mut stats,
                            StreamingImageUpdateNv12 {
                                window,
                                token,
                                image,
                                stream_generation,
                                width,
                                height,
                                update_rect_px,
                                y_bytes_per_row,
                                y_plane: &y_plane,
                                uv_bytes_per_row,
                                uv_plane: &uv_plane,
                                color_info,
                            },
                        ) {
                            continue;
                        }

                        let t0 = std::time::Instant::now();
                        match super::yuv::nv12_to_rgba8_rect(super::yuv::Nv12ToRgba8RectInput {
                            width,
                            height,
                            update_rect_px,
                            y_bytes_per_row,
                            y_plane: &y_plane,
                            uv_bytes_per_row,
                            uv_plane: &uv_plane,
                            range: color_info.range,
                            matrix: color_info.matrix,
                        }) {
                            Ok((rect, rgba)) => {
                                stats.yuv_conversions_applied =
                                    stats.yuv_conversions_applied.saturating_add(1);
                                stats.yuv_convert_us = stats
                                    .yuv_convert_us
                                    .saturating_add(t0.elapsed().as_micros() as u64);
                                stats.yuv_convert_output_bytes = stats
                                    .yuv_convert_output_bytes
                                    .saturating_add(rgba.len() as u64);

                                self.apply_streaming_image_update_rgba8(
                                    &mut stats,
                                    StreamingImageUpdateRgba8 {
                                        window,
                                        token,
                                        image,
                                        stream_generation,
                                        width,
                                        height,
                                        update_rect_px: Some(rect),
                                        bytes_per_row: rect.w.saturating_mul(4),
                                        bytes: &rgba,
                                        color_info: fret_core::ImageColorInfo::srgb_rgba(),
                                        alpha_mode: fret_core::AlphaMode::Opaque,
                                    },
                                );
                            }
                            Err(_message) => {
                                if self.config.streaming_update_ack_enabled {
                                    let target = window
                                        .or(self.main_window)
                                        .or_else(|| self.windows.keys().next());
                                    if let Some(target) = target {
                                        self.deliver_window_event_now(
                                            target,
                                            &Event::ImageUpdateDropped {
                                                token,
                                                image,
                                                reason:
                                                    fret_core::ImageUpdateDropReason::InvalidPayload,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Effect::ImageUpdateI420 {
                        window,
                        token,
                        image,
                        stream_generation,
                        width,
                        height,
                        update_rect_px,
                        y_bytes_per_row,
                        y_plane,
                        u_bytes_per_row,
                        u_plane,
                        v_bytes_per_row,
                        v_plane,
                        color_info,
                        alpha_mode: _,
                    } => {
                        stats.yuv_conversions_attempted =
                            stats.yuv_conversions_attempted.saturating_add(1);
                        let t0 = std::time::Instant::now();
                        match super::yuv::i420_to_rgba8_rect(super::yuv::I420ToRgba8RectInput {
                            width,
                            height,
                            update_rect_px,
                            y_bytes_per_row,
                            y_plane: &y_plane,
                            u_bytes_per_row,
                            u_plane: &u_plane,
                            v_bytes_per_row,
                            v_plane: &v_plane,
                            range: color_info.range,
                            matrix: color_info.matrix,
                        }) {
                            Ok((rect, rgba)) => {
                                stats.yuv_conversions_applied =
                                    stats.yuv_conversions_applied.saturating_add(1);
                                stats.yuv_convert_us = stats
                                    .yuv_convert_us
                                    .saturating_add(t0.elapsed().as_micros() as u64);
                                stats.yuv_convert_output_bytes = stats
                                    .yuv_convert_output_bytes
                                    .saturating_add(rgba.len() as u64);

                                self.apply_streaming_image_update_rgba8(
                                    &mut stats,
                                    StreamingImageUpdateRgba8 {
                                        window,
                                        token,
                                        image,
                                        stream_generation,
                                        width,
                                        height,
                                        update_rect_px: Some(rect),
                                        bytes_per_row: rect.w.saturating_mul(4),
                                        bytes: &rgba,
                                        color_info: fret_core::ImageColorInfo::srgb_rgba(),
                                        alpha_mode: fret_core::AlphaMode::Opaque,
                                    },
                                );
                            }
                            Err(_message) => {
                                if self.config.streaming_update_ack_enabled {
                                    let target = window
                                        .or(self.main_window)
                                        .or_else(|| self.windows.keys().next());
                                    if let Some(target) = target {
                                        self.deliver_window_event_now(
                                            target,
                                            &Event::ImageUpdateDropped {
                                                token,
                                                image,
                                                reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Effect::ImageUnregister { image } => {
                        let Some(renderer) = self.renderer.as_mut() else {
                            continue;
                        };

                        self.uploaded_images.remove(&image);

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
                            let closed = self.close_window(window);
                            if !closed {
                                continue;
                            }

                            if is_main && self.config.exit_on_main_window_close {
                                let windows: Vec<fret_core::AppWindowId> =
                                    self.windows.keys().collect();
                                for window in windows {
                                    let _ = self.force_close_window(window);
                                }
                                self.dispatcher.shutdown();
                                event_loop.exit();
                                return;
                            }

                            if self.windows.is_empty() {
                                self.dispatcher.shutdown();
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
                                    let caps = self
                                        .app
                                        .global::<PlatformCapabilities>()
                                        .cloned()
                                        .unwrap_or_default();
                                    let allow_follow = caps.ui.window_set_outer_position
                                        == fret_runtime::WindowSetOuterPositionQuality::Reliable;
                                    if allow_follow {
                                        if caps.ui.window_z_level
                                            != fret_runtime::WindowZLevelQuality::None
                                            && let Some(state) = self.windows.get(new_window)
                                        {
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

                            self.app.request_redraw(new_window);
                        }
                        WindowRequest::SetInnerSize { window, size } => {
                            if let Some(state) = self.windows.get(window) {
                                let _ = state.window.request_surface_size(
                                    winit::dpi::LogicalSize::new(
                                        size.width.0 as f64,
                                        size.height.0 as f64,
                                    )
                                    .into(),
                                );
                                state.window.request_redraw();
                            }
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

            let streaming_snapshot_enabled = self.config.streaming_perf_snapshot_enabled
                || std::env::var_os("FRET_STREAMING_DEBUG").is_some_and(|v| !v.is_empty());
            let streaming_stats_have_activity = stats.update_effects_seen > 0
                || stats.update_effects_enqueued > 0
                || stats.update_effects_replaced > 0
                || stats.update_effects_applied > 0
                || stats.update_effects_delayed_budget > 0
                || stats.update_effects_dropped_staging > 0
                || stats.upload_bytes_budgeted > 0
                || stats.upload_bytes_applied > 0
                || stats.pending_updates > 0
                || stats.pending_staging_bytes > 0
                || stats.yuv_conversions_attempted > 0
                || stats.yuv_convert_us > 0;
            if streaming_snapshot_enabled && streaming_stats_have_activity {
                self.app.set_global(fret_core::StreamingUploadPerfSnapshot {
                    frame_id: self.frame_id,
                    upload_budget_bytes_per_frame: stats.upload_budget_bytes_per_frame,
                    staging_budget_bytes: stats.staging_budget_bytes,
                    update_effects_seen: u64::from(stats.update_effects_seen),
                    update_effects_enqueued: u64::from(stats.update_effects_enqueued),
                    update_effects_replaced: u64::from(stats.update_effects_replaced),
                    update_effects_applied: u64::from(stats.update_effects_applied),
                    update_effects_delayed_budget: u64::from(stats.update_effects_delayed_budget),
                    update_effects_dropped_staging: u64::from(stats.update_effects_dropped_staging),
                    upload_bytes_budgeted: stats.upload_bytes_budgeted,
                    upload_bytes_applied: stats.upload_bytes_applied,
                    pending_updates: u64::from(stats.pending_updates),
                    pending_staging_bytes: stats.pending_staging_bytes,
                    yuv_convert_us: stats.yuv_convert_us,
                    yuv_convert_output_bytes: stats.yuv_convert_output_bytes,
                    yuv_conversions_attempted: u64::from(stats.yuv_conversions_attempted),
                    yuv_conversions_applied: u64::from(stats.yuv_conversions_applied),
                });
            }

            if std::env::var_os("FRET_STREAMING_DEBUG").is_some_and(|v| !v.is_empty())
                && (stats.update_effects_delayed_budget > 0
                    || stats.update_effects_dropped_staging > 0
                    || stats.update_effects_replaced > 0
                    || stats.yuv_conversions_attempted > 0)
            {
                tracing::debug!(
                    seen = stats.update_effects_seen,
                    enqueued = stats.update_effects_enqueued,
                    replaced = stats.update_effects_replaced,
                    applied = stats.update_effects_applied,
                    delayed_budget = stats.update_effects_delayed_budget,
                    dropped_staging = stats.update_effects_dropped_staging,
                    upload_bytes_budgeted = stats.upload_bytes_budgeted,
                    upload_bytes_applied = stats.upload_bytes_applied,
                    upload_budget_bytes_per_frame = stats.upload_budget_bytes_per_frame,
                    staging_budget_bytes = stats.staging_budget_bytes,
                    pending_updates = stats.pending_updates,
                    pending_staging_bytes = stats.pending_staging_bytes,
                    yuv_attempted = stats.yuv_conversions_attempted,
                    yuv_applied = stats.yuv_conversions_applied,
                    yuv_convert_us = stats.yuv_convert_us,
                    yuv_output_bytes = stats.yuv_convert_output_bytes,
                    "streaming image updates queued/budgeted"
                );
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

            if self.streaming_uploads.has_pending() {
                match self.streaming_uploads.pending_redraw_hint() {
                    Some(windows) if windows.is_empty() => {
                        for (_id, state) in self.windows.iter() {
                            state.window.request_redraw();
                        }
                    }
                    Some(windows) => {
                        for window in windows {
                            if let Some(state) = self.windows.get(window) {
                                state.window.request_redraw();
                            }
                        }
                    }
                    None => {}
                }
            }

            if !did_work {
                break;
            }
        }
    }

    fn drain_inboxes(&mut self, window: Option<fret_core::AppWindowId>) -> bool {
        let did_work = self.app.with_global_mut_untracked(
            fret_runtime::InboxDrainRegistry::default,
            |registry, app| registry.drain_all(app, window),
        );
        tracing::trace!(?window, did_work, "driver: drain_inboxes");
        did_work
    }

    fn propagate_model_changes(&mut self) -> bool {
        let changed = self.app.take_changed_models();
        if changed.is_empty() {
            return false;
        }

        for (window, runtime) in self.windows.iter_mut() {
            self.driver.handle_model_changes(
                WinitWindowContext {
                    app: &mut self.app,
                    window,
                    state: &mut runtime.user,
                },
                &changed,
            );
        }
        true
    }

    fn propagate_global_changes(&mut self) -> bool {
        let changed = self.app.take_changed_globals();
        if changed.is_empty() {
            return false;
        }

        #[cfg(windows)]
        {
            if changed.contains(&TypeId::of::<fret_runtime::KeymapService>()) {
                windows_menu::sync_keymap_from_app(&self.app);
            }
            if changed.contains(&TypeId::of::<fret_runtime::WindowInputContextService>())
                || changed.contains(&TypeId::of::<fret_runtime::WindowCommandEnabledService>())
                || changed.contains(&TypeId::of::<
                    fret_runtime::WindowCommandActionAvailabilityService,
                >())
                || changed.contains(&TypeId::of::<fret_runtime::WindowCommandGatingService>())
            {
                windows_menu::sync_command_gating_from_app(&self.app);
            }
        }

        #[cfg(target_os = "macos")]
        {
            let keymap_changed = changed.contains(&TypeId::of::<fret_runtime::KeymapService>());
            if keymap_changed {
                macos_menu::sync_keymap_from_app(&self.app);
            }
            if changed.contains(&TypeId::of::<fret_runtime::WindowInputContextService>())
                || changed.contains(&TypeId::of::<fret_runtime::WindowCommandEnabledService>())
                || changed.contains(&TypeId::of::<
                    fret_runtime::WindowCommandActionAvailabilityService,
                >())
                || changed.contains(&TypeId::of::<fret_runtime::WindowCommandGatingService>())
            {
                macos_menu::sync_command_gating_from_app(&self.app);
            }
            if keymap_changed && let Some(menu_bar) = self.menu_bar.clone() {
                macos_menu::set_app_menu_bar(&self.app, &menu_bar);
            }
        }

        if changed.contains(&TypeId::of::<fret_core::TextFontFamilyConfig>())
            && let (Some(renderer), Some(config)) = (
                self.renderer.as_mut(),
                self.app.global::<fret_core::TextFontFamilyConfig>(),
            )
            && renderer.set_text_font_families(config)
        {
            let new_key = renderer.text_font_stack_key();
            let old_key = self
                .app
                .global::<fret_runtime::TextFontStackKey>()
                .map(|k| k.0);
            if old_key != Some(new_key) {
                self.app.set_global::<fret_runtime::TextFontStackKey>(
                    fret_runtime::TextFontStackKey(new_key),
                );
            }

            for (_id, state) in self.windows.iter() {
                state.window.request_redraw();
            }
        }

        for (window, runtime) in self.windows.iter_mut() {
            self.driver.handle_global_changes(
                WinitWindowContext {
                    app: &mut self.app,
                    window,
                    state: &mut runtime.user,
                },
                &changed,
            );
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

    fn dock_drag_pointer_id(&self) -> Option<fret_core::PointerId> {
        use fret_runtime::DragHost as _;
        self.app.find_drag_pointer_id(|d| {
            d.cross_window_hover && d.kind == fret_app::DRAG_KIND_DOCK_PANEL
        })
    }

    #[cfg(target_os = "macos")]
    fn maybe_finish_dock_drag_released_outside(&mut self) -> bool {
        let Some(pointer_id) = self.dock_drag_pointer_id() else {
            return false;
        };

        let (source_window, current_window, dragging) = {
            let Some(drag) = self.app.drag(pointer_id) else {
                return false;
            };
            if !drag.cross_window_hover
                || drag.kind != fret_app::DRAG_KIND_DOCK_PANEL
                || macos_is_left_mouse_down()
                || self.saw_left_mouse_release_this_turn
            {
                return false;
            }
            (drag.source_window, drag.current_window, drag.dragging)
        };

        dock_tearoff_log(format_args!(
            "[poll-up] pointer={:?} source={:?} current={:?} screen_pos={:?} dragging={}",
            pointer_id, source_window, current_window, self.cursor_screen_pos, dragging
        ));

        // If the mouse was released outside any window, winit may not deliver a `MouseInput`
        // event to any window. Use the regular cursor-based drop routing so docking back into an
        // existing window still works (ImGui-style).
        if let Some(d) = self.app.drag_mut(pointer_id)
            && d.kind == fret_app::DRAG_KIND_DOCK_PANEL
        {
            d.dragging = true;
        }

        self.route_internal_drag_drop_from_cursor();
        dock_tearoff_log(format_args!(
            "[poll-drop] dispatched target={:?}",
            source_window
        ));

        if self
            .app
            .drag(pointer_id)
            .is_some_and(|d| d.cross_window_hover)
        {
            self.app.cancel_drag(pointer_id);
            let _ = self.clear_internal_drag_hover_if_needed();
        }

        true
    }

    fn dispatch_internal_drag_event(
        &mut self,
        window: fret_core::AppWindowId,
        pointer_id: fret_core::PointerId,
        kind: InternalDragKind,
        position: Point,
    ) {
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        let modifiers = state.platform.input.modifiers;
        let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
        self.driver.handle_event(
            WinitEventContext {
                app: &mut self.app,
                services,
                window,
                state: &mut state.user,
            },
            &Event::InternalDrag(InternalDragEvent {
                pointer_id,
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
        if self.dock_drag_pointer_id().is_some() {
            return false;
        }
        let pointer_id = self
            .internal_drag_pointer_id
            .take()
            .unwrap_or(fret_core::PointerId(0));
        self.internal_drag_hover_window = None;
        let pos = self.internal_drag_hover_pos.take().unwrap_or_default();
        self.dispatch_internal_drag_event(window, pointer_id, InternalDragKind::Cancel, pos);
        true
    }

    fn route_internal_drag_hover_from_cursor(&mut self) -> bool {
        #[cfg(target_os = "macos")]
        self.macos_refresh_cursor_screen_pos_for_dock_drag();

        let Some(pointer_id) = self.dock_drag_pointer_id() else {
            return self.clear_internal_drag_hover_if_needed();
        };
        let Some((drag_kind, drag_source_window, cross_window_hover)) = self
            .app
            .drag(pointer_id)
            .map(|d| (d.kind, d.source_window, d.cross_window_hover))
        else {
            return self.clear_internal_drag_hover_if_needed();
        };
        if !cross_window_hover {
            return self.clear_internal_drag_hover_if_needed();
        }

        let Some(screen_pos) = self.cursor_screen_pos else {
            return false;
        };

        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let allow_window_under_cursor =
            caps.ui.window_hover_detection != fret_runtime::WindowHoverDetectionQuality::None;

        // When a dock tear-off window is following the cursor, the cursor is always "inside" that
        // moving window. Prefer other windows under the cursor so we can dock back into the main
        // window (ImGui-style).
        let prefer_not = self
            .dock_tearoff_follow
            .filter(|_| drag_kind == fret_app::DRAG_KIND_DOCK_PANEL)
            .map(|f| f.window);

        // Prefer the window we already hovered, if the cursor is still inside it. This makes
        // cross-window drag hover stable even when OS windows overlap and we don't have z-order.
        let hovered = self
            .internal_drag_hover_window
            .filter(|w| self.screen_pos_in_window(*w, screen_pos))
            .filter(|w| Some(*w) != prefer_not)
            .or_else(|| {
                allow_window_under_cursor
                    .then(|| self.window_under_cursor(screen_pos, prefer_not))
                    .flatten()
            });
        let hovered = hovered.or_else(|| {
            // For dock tear-off, keep delivering `InternalDrag::Over` to the source window even
            // when the cursor is outside all windows so the UI can react before mouse-up.
            (drag_kind == fret_app::DRAG_KIND_DOCK_PANEL)
                .then_some(drag_source_window)
                .filter(|w| self.windows.contains_key(*w))
        });
        if hovered != self.internal_drag_hover_window {
            if let Some(prev) = self.internal_drag_hover_window.take() {
                let prev_pos = self.internal_drag_hover_pos.take().unwrap_or_default();
                self.dispatch_internal_drag_event(
                    prev,
                    pointer_id,
                    InternalDragKind::Leave,
                    prev_pos,
                );
            }
            if let Some(next) = hovered
                && let Some(pos) = self.local_pos_for_window(next, screen_pos)
            {
                self.dispatch_internal_drag_event(next, pointer_id, InternalDragKind::Enter, pos);
                self.internal_drag_hover_window = Some(next);
                self.internal_drag_hover_pos = Some(pos);
                self.internal_drag_pointer_id = Some(pointer_id);
            }
        }

        let Some(current) = self.internal_drag_hover_window else {
            return false;
        };
        let Some(pos) = self.local_pos_for_window(current, screen_pos) else {
            return false;
        };

        if drag_kind == fret_app::DRAG_KIND_DOCK_PANEL
            && std::env::var_os("FRET_DOCK_TEAROFF_LOG").is_some()
        {
            if let Some(state) = self.windows.get(current) {
                let size_phys = state.window.surface_size();
                let scale = state.window.scale_factor();
                let size_logical: winit::dpi::LogicalSize<f32> = size_phys.to_logical(scale);
                let margin = 32.0f32;
                let oob = pos.x.0 < -margin
                    || pos.y.0 < -margin
                    || pos.x.0 > size_logical.width + margin
                    || pos.y.0 > size_logical.height + margin;
                if oob {
                    let outer = state.window.outer_position().ok();
                    let deco = state.window.surface_position();
                    dock_tearoff_log(format_args!(
                        "[cursor-oob] window={:?} screen=({:.1},{:.1}) local=({:.1},{:.1}) size=({:.1},{:.1}) scale={:.3} outer={:?} deco=({},{})",
                        current,
                        screen_pos.x,
                        screen_pos.y,
                        pos.x.0,
                        pos.y.0,
                        size_logical.width,
                        size_logical.height,
                        scale,
                        outer,
                        deco.x,
                        deco.y,
                    ));
                }
            }
        }

        if let Some(d) = self.app.drag_mut(pointer_id) {
            d.current_window = current;
            d.position = pos;
        }

        self.internal_drag_hover_pos = Some(pos);
        self.dispatch_internal_drag_event(current, pointer_id, InternalDragKind::Over, pos);
        true
    }

    fn route_internal_drag_drop_from_cursor(&mut self) -> bool {
        #[cfg(target_os = "macos")]
        self.macos_refresh_cursor_screen_pos_for_dock_drag();

        let Some(pointer_id) = self.dock_drag_pointer_id() else {
            return false;
        };
        let Some((drag_kind, drag_source_window, cross_window_hover)) = self
            .app
            .drag(pointer_id)
            .map(|d| (d.kind, d.source_window, d.cross_window_hover))
        else {
            return false;
        };
        if !cross_window_hover {
            return false;
        }

        let screen_pos = self
            .cursor_screen_pos
            .or_else(|| self.cursor_screen_pos_fallback_for_window(drag_source_window));
        let Some(screen_pos) = screen_pos else {
            return false;
        };

        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let allow_window_under_cursor =
            caps.ui.window_hover_detection != fret_runtime::WindowHoverDetectionQuality::None;

        let prefer_not = self
            .dock_tearoff_follow
            .filter(|_| drag_kind == fret_app::DRAG_KIND_DOCK_PANEL)
            .map(|f| f.window);

        // Prefer the last hovered window if possible; window overlap makes hit-testing ambiguous.
        let target = self
            .internal_drag_hover_window
            .filter(|w| self.screen_pos_in_window(*w, screen_pos))
            .filter(|w| Some(*w) != prefer_not)
            .or_else(|| {
                allow_window_under_cursor
                    .then(|| self.window_under_cursor(screen_pos, prefer_not))
                    .flatten()
            })
            .or(self.internal_drag_hover_window);
        // If the cursor is outside all windows (Unity/ImGui-style tear-off), still deliver the
        // drop to the source window using the last known screen cursor position.
        let target = target.unwrap_or(drag_source_window);
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

        if drag_kind == fret_app::DRAG_KIND_DOCK_PANEL
            && target != drag_source_window
            && let Some(runtime) = self.windows.get(target)
        {
            let sender = self
                .windows
                .get(drag_source_window)
                .map(|w| w.window.as_ref());
            let _ = bring_window_to_front(runtime.window.as_ref(), sender);
        }

        if let Some(prev) = self.internal_drag_hover_window.take()
            && prev != target
        {
            let prev_pos = self.internal_drag_hover_pos.take().unwrap_or_default();
            self.dispatch_internal_drag_event(prev, pointer_id, InternalDragKind::Leave, prev_pos);
        }
        self.internal_drag_hover_window = Some(target);
        self.internal_drag_hover_pos = Some(pos);
        self.internal_drag_pointer_id = Some(pointer_id);

        if let Some(d) = self.app.drag_mut(pointer_id) {
            d.current_window = target;
            d.position = pos;
        }

        self.dispatch_internal_drag_event(target, pointer_id, InternalDragKind::Drop, pos);
        true
    }

    fn cursor_screen_pos_fallback_for_window(
        &self,
        window: fret_core::AppWindowId,
    ) -> Option<PhysicalPosition<f64>> {
        let state = self.windows.get(window)?;
        // `Window::surface_position()` is defined as the decoration offset from the outer
        // window position to the client/surface origin (ImGui-style multi-viewport contract).
        // Convert it to a screen-space client origin before adding a local cursor position.
        let outer = state.window.outer_position().ok()?;
        let deco = state.window.surface_position();
        let scale = state.window.scale_factor();
        let origin = client_origin_screen(outer, deco);
        let x = origin.x + state.platform.input.cursor_pos.x.0 as f64 * scale;
        let y = origin.y + state.platform.input.cursor_pos.y.0 as f64 * scale;
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
        let Ok(outer) = state.window.outer_position() else {
            return false;
        };
        let deco = state.window.surface_position();
        let size = state.window.surface_size();
        screen_pos_in_client(client_origin_screen(outer, deco), size, screen_pos)
    }

    fn local_pos_for_window(
        &self,
        window: fret_core::AppWindowId,
        screen_pos: PhysicalPosition<f64>,
    ) -> Option<Point> {
        let state = self.windows.get(window)?;
        let outer = state.window.outer_position().ok()?;
        let deco = state.window.surface_position();
        Some(local_pos_for_screen_pos(
            client_origin_screen(outer, deco),
            state.window.scale_factor(),
            screen_pos,
        ))
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
            let Ok(outer) = state.window.outer_position() else {
                continue;
            };
            let deco = state.window.surface_position();
            let size = state.window.surface_size();
            let left = outer.x as f64 + deco.x as f64;
            let top = outer.y as f64 + deco.y as f64;
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
        if self.dock_tearoff_follow.is_some() && self.dock_drag_pointer_id().is_none() {
            // If the dock drag session was canceled (e.g. Escape), ensure we do not keep moving a
            // dock tear-off window indefinitely.
            self.stop_dock_tearoff_follow(Instant::now(), false);
            return true;
        }

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

        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if caps.ui.window_set_outer_position
            != fret_runtime::WindowSetOuterPositionQuality::Reliable
        {
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
            if caps.ui.window_z_level != fret_runtime::WindowZLevelQuality::None {
                state.window.set_window_level(WindowLevel::AlwaysOnTop);
            }
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

        dock_tearoff_log(format_args!(
            "[follow-stop] window={:?} source={:?} cursor={:?} raise_on_macos={}",
            follow.window, follow.source_window, self.cursor_screen_pos, _raise_on_macos
        ));

        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();

        if let Some(state) = self.windows.get(follow.window) {
            if caps.ui.window_z_level != fret_runtime::WindowZLevelQuality::None {
                state.window.set_window_level(WindowLevel::Normal);
            }
            if caps.ui.window_set_outer_position
                == fret_runtime::WindowSetOuterPositionQuality::Reliable
                && let Some(pos) =
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

impl<D: WinitAppDriver> WinitRunner<D> {
    pub fn new_app(config: WinitRunnerConfig, app: App, driver: D) -> Self {
        Self::new(config, app, driver)
    }
}

fn client_origin_screen(
    outer: winit::dpi::PhysicalPosition<i32>,
    decoration_offset: winit::dpi::PhysicalPosition<i32>,
) -> winit::dpi::PhysicalPosition<f64> {
    winit::dpi::PhysicalPosition::new(
        outer.x as f64 + decoration_offset.x as f64,
        outer.y as f64 + decoration_offset.y as f64,
    )
}

fn screen_pos_in_client(
    client_origin: winit::dpi::PhysicalPosition<f64>,
    client_size: winit::dpi::PhysicalSize<u32>,
    screen_pos: winit::dpi::PhysicalPosition<f64>,
) -> bool {
    let left = client_origin.x;
    let top = client_origin.y;
    let right = left + client_size.width as f64;
    let bottom = top + client_size.height as f64;
    screen_pos.x >= left && screen_pos.x < right && screen_pos.y >= top && screen_pos.y < bottom
}

fn is_wayland_session(xdg_session_type: Option<&str>, wayland_display: Option<&str>) -> bool {
    if xdg_session_type.is_some_and(|v| v.eq_ignore_ascii_case("wayland")) {
        return true;
    }
    wayland_display.is_some_and(|v| !v.is_empty())
}

#[cfg(target_os = "linux")]
fn linux_is_wayland_session() -> bool {
    let xdg_session_type = std::env::var("XDG_SESSION_TYPE").ok();
    let wayland_display = std::env::var("WAYLAND_DISPLAY").ok();
    is_wayland_session(xdg_session_type.as_deref(), wayland_display.as_deref())
}

fn local_pos_for_screen_pos(
    client_origin: winit::dpi::PhysicalPosition<f64>,
    scale_factor: f64,
    screen_pos: winit::dpi::PhysicalPosition<f64>,
) -> Point {
    let local_physical = winit::dpi::PhysicalPosition::new(
        screen_pos.x - client_origin.x,
        screen_pos.y - client_origin.y,
    );
    let local_logical: winit::dpi::LogicalPosition<f32> = local_physical.to_logical(scale_factor);
    Point::new(Px(local_logical.x), Px(local_logical.y))
}

fn outer_pos_for_cursor_grab(
    screen_pos: PhysicalPosition<f64>,
    grab_offset_logical: Point,
    scale_factor: f64,
    decoration_offset: winit::dpi::PhysicalPosition<i32>,
    max_client_logical: Option<winit::dpi::LogicalSize<f32>>,
) -> Option<(f64, f64)> {
    if !grab_offset_logical.x.0.is_finite()
        || !grab_offset_logical.y.0.is_finite()
        || grab_offset_logical.x.0 < 0.0
        || grab_offset_logical.y.0 < 0.0
    {
        return None;
    }

    let mut grab_x = grab_offset_logical.x.0;
    let mut grab_y = grab_offset_logical.y.0;
    if let Some(max) = max_client_logical {
        if max.width.is_finite() && max.width > 0.0 {
            grab_x = grab_x.min(max.width).max(0.0);
        } else {
            grab_x = 0.0;
        }
        if max.height.is_finite() && max.height > 0.0 {
            grab_y = grab_y.min(max.height).max(0.0);
        } else {
            grab_y = 0.0;
        }
    }

    // Match ImGui's platform contract:
    // - viewport pos is client/inner screen position (logical)
    // - winit expects outer position
    // - therefore: outer = desired_client - decoration_offset(window)
    // See `repo-ref/dear-imgui-rs/backends/dear-imgui-winit/src/multi_viewport.rs:winit_set_window_pos`.
    let grab_client_x = grab_x as f64 * scale_factor;
    let grab_client_y = grab_y as f64 * scale_factor;
    let grab_outer_x = decoration_offset.x as f64 + grab_client_x;
    let grab_outer_y = decoration_offset.y as f64 + grab_client_y;

    let x = screen_pos.x - grab_outer_x;
    let y = screen_pos.y - grab_outer_y;
    Some((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::{PhysicalPosition, PhysicalSize};

    #[test]
    fn is_wayland_session_true_for_xdg_session_type_wayland() {
        assert!(is_wayland_session(Some("wayland"), None));
        assert!(is_wayland_session(Some("Wayland"), None));
    }

    #[test]
    fn is_wayland_session_true_for_wayland_display() {
        assert!(is_wayland_session(None, Some("wayland-0")));
    }

    #[test]
    fn is_wayland_session_false_for_x11_and_no_wayland_display() {
        assert!(!is_wayland_session(Some("x11"), None));
        assert!(!is_wayland_session(None, Some("")));
    }

    #[test]
    fn outer_pos_for_cursor_grab_accounts_for_decorations_and_scale() {
        let cursor = PhysicalPosition::new(1000.0, 500.0);
        let grab = Point::new(Px(20.0), Px(40.0));
        let scale = 1.5;
        let deco = winit::dpi::PhysicalPosition::new(10, 30);
        let max_client = winit::dpi::LogicalSize::new(200.0f32, 200.0f32);

        let (x, y) = outer_pos_for_cursor_grab(cursor, grab, scale, deco, Some(max_client))
            .expect("expected outer pos");
        assert_eq!(x, 960.0);
        assert_eq!(y, 410.0);
    }

    #[test]
    fn outer_pos_for_cursor_grab_clamps_to_client_size() {
        let cursor = PhysicalPosition::new(1000.0, 500.0);
        let grab = Point::new(Px(9999.0), Px(9999.0));
        let scale = 2.0;
        let deco = winit::dpi::PhysicalPosition::new(0, 0);
        let max_client = winit::dpi::LogicalSize::new(100.0f32, 100.0f32);

        let (x, y) = outer_pos_for_cursor_grab(cursor, grab, scale, deco, Some(max_client))
            .expect("expected outer pos");
        assert_eq!(x, 800.0);
        assert_eq!(y, 300.0);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_cursor_transform_table_prefers_key_hint_then_last_used() {
        let key_a = MacCursorScreenKey {
            origin_x: 0,
            origin_y: 0,
            width: 100,
            height: 100,
            scale_milli: 2000,
        };
        let key_b = MacCursorScreenKey {
            origin_x: 100,
            origin_y: 0,
            width: 100,
            height: 100,
            scale_milli: 2000,
        };

        let mut table = MacCursorTransformTable::default();
        table.by_screen.insert(
            key_a,
            MacCursorTransform {
                scale_factor: 1.0,
                x_offset: 10.0,
                y_offset: 100.0,
                y_flipped: Some(true),
                last_winit_y: None,
                last_cocoa_y: None,
            },
        );
        table.by_screen.insert(
            key_b,
            MacCursorTransform {
                scale_factor: 1.0,
                x_offset: 20.0,
                y_offset: 200.0,
                y_flipped: Some(true),
                last_winit_y: None,
                last_cocoa_y: None,
            },
        );

        let cocoa_pos = cocoa::foundation::NSPoint { x: 1.0, y: 2.0 };
        let mapped = table
            .map_with_key_hint(cocoa_pos, Some(key_a))
            .expect("expected mapping");
        assert_eq!(mapped, PhysicalPosition::new(11.0, 98.0));
        assert_eq!(table.last_used, Some(key_a));

        let mapped = table
            .map_with_key_hint(cocoa_pos, None)
            .expect("expected mapping via last_used");
        assert_eq!(mapped, PhysicalPosition::new(11.0, 98.0));

        let mapped = table
            .map_with_key_hint(cocoa_pos, Some(key_b))
            .expect("expected mapping");
        assert_eq!(mapped, PhysicalPosition::new(21.0, 198.0));
        assert_eq!(table.last_used, Some(key_b));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_cursor_transform_table_falls_back_to_any_transform() {
        let key_a = MacCursorScreenKey {
            origin_x: 0,
            origin_y: 0,
            width: 100,
            height: 100,
            scale_milli: 1000,
        };
        let key_b = MacCursorScreenKey {
            origin_x: 200,
            origin_y: 0,
            width: 100,
            height: 100,
            scale_milli: 1000,
        };

        let mut table = MacCursorTransformTable::default();
        table.by_screen.insert(
            key_a,
            MacCursorTransform {
                scale_factor: 1.0,
                x_offset: 5.0,
                y_offset: 50.0,
                y_flipped: Some(true),
                last_winit_y: None,
                last_cocoa_y: None,
            },
        );

        let cocoa_pos = cocoa::foundation::NSPoint { x: 1.0, y: 2.0 };
        let mapped = table
            .map_with_key_hint(cocoa_pos, Some(key_b))
            .expect("expected mapping via any transform");
        assert_eq!(mapped, PhysicalPosition::new(6.0, 48.0));
    }

    #[test]
    fn client_origin_screen_adds_decoration_offset() {
        let outer = winit::dpi::PhysicalPosition::new(100, 200);
        let deco = winit::dpi::PhysicalPosition::new(12, 34);
        let origin = client_origin_screen(outer, deco);
        assert_eq!(origin, PhysicalPosition::new(112.0, 234.0));
    }

    #[test]
    fn screen_pos_in_client_uses_half_open_bounds() {
        let origin = PhysicalPosition::new(10.0, 20.0);
        let size = PhysicalSize::new(100u32, 50u32);

        assert!(screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(10.0, 20.0)
        ));
        assert!(screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(109.9, 69.9)
        ));

        assert!(!screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(110.0, 20.0)
        ));
        assert!(!screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(10.0, 70.0)
        ));
    }

    #[test]
    fn local_pos_for_screen_pos_respects_scale_factor() {
        let origin = PhysicalPosition::new(100.0, 200.0);
        let scale = 2.0;
        let screen_pos = PhysicalPosition::new(120.0, 240.0);
        let local = local_pos_for_screen_pos(origin, scale, screen_pos);
        assert_eq!(local, Point::new(Px(10.0), Px(20.0)));
    }
}
