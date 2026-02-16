//! Desktop launcher implementation (winit + wgpu).

pub use super::super::common::*;

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};

use fret_core::time::Instant;
#[cfg(feature = "hotpatch-subsecond")]
mod hotpatch;

use fret_app::{App, CreateWindowKind, CreateWindowRequest, Effect};
use fret_core::{
    Event, ExternalDragEvent, ExternalDragKind, InternalDragEvent, InternalDragKind, Point, Px,
    Rect, Scene, Size, UiServices, WindowMetricsService,
};
use fret_platform_native::clipboard::NativeClipboard;
use fret_platform_native::external_drop::NativeExternalDrop;
use fret_platform_native::file_dialog::NativeFileDialog;
use fret_platform_native::open_url::NativeOpenUrl;
use fret_render::{Renderer, SurfaceState, WgpuContext};
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
    event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy},
    window::{Window, WindowId, WindowLevel},
};

#[cfg(target_os = "android")]
use winit::platform::android::EventLoopExtAndroid as _;

use crate::RunnerError;
// Platform provider traits are imported where their methods are called.

fn read_startup_incoming_open_paths_from_args() -> Vec<std::path::PathBuf> {
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    for arg in std::env::args_os().skip(1) {
        if arg.is_empty() {
            continue;
        }
        let path = std::path::PathBuf::from(arg);
        if path.is_file() {
            paths.push(path);
        }
    }
    paths
}

#[derive(Debug, Default)]
struct DiagIncomingOpenPayload {
    files: Vec<fret_core::ExternalDropFileData>,
    texts: Vec<String>,
}

#[derive(Debug, Default)]
struct IncomingOpenPathPayload {
    paths: Vec<std::path::PathBuf>,
}

#[derive(Debug, Default, Clone)]
struct DiagWindowInsetsOverride {
    /// `None` means "no override".
    ///
    /// `Some(None)` means "known-but-none" (cleared).
    ///
    /// `Some(Some(v))` means "override to v".
    safe_area_insets: Option<Option<fret_core::Edges>>,
    /// See `safe_area_insets`.
    occlusion_insets: Option<Option<fret_core::Edges>>,
}

mod app_handler;
#[cfg(feature = "dev-state")]
mod dev_state;
mod diag_bundle_screenshots;
mod diag_cursor_override;
#[cfg(feature = "diag-screenshots")]
mod diag_screenshots;
mod dispatcher;
mod docking;
mod effects;
mod event_routing;
#[cfg(target_os = "ios")]
mod ios_keyboard;
#[cfg(target_os = "macos")]
mod macos_menu;
mod no_services;
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
mod renderdoc_capture;
mod restart_trigger;
#[cfg(windows)]
mod windows_menu;

mod event_loop;
mod macos_cursor;
mod platform_prefs;
mod render;
mod run;
mod streaming_images;
mod timers;
#[cfg(target_os = "windows")]
mod win32;
mod window;
mod window_lifecycle;

pub use event_loop::RunnerUserEvent;
#[cfg(windows)]
pub use event_loop::windows_msg_hook;
pub use run::{WinitAppBuilder, run_app, run_app_with_event_loop};

use super::super::streaming_upload::StreamingUploadQueue;
use diag_bundle_screenshots::DiagBundleScreenshotCapture;
use dispatcher::DesktopDispatcher;
use no_services::NoUiServices;
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use renderdoc_capture::RenderDocCapture;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
struct RenderDocCapture;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
impl RenderDocCapture {
    fn try_init() -> Option<Self> {
        None
    }

    fn request_capture(&mut self) {}

    fn begin_capture_if_requested(&mut self) -> bool {
        false
    }

    fn end_capture(&mut self) {}
}

#[cfg(feature = "hotpatch-subsecond")]
use hotpatch::{HotpatchRequestKind, HotpatchTrigger, hotpatch_trigger_from_env};

#[cfg(target_os = "macos")]
use macos_cursor::{
    MacCursorTransformTable, macos_cursor_trace_enabled, macos_dockfloating_parenting_enabled,
    macos_is_left_mouse_down,
};
use macos_cursor::{dock_tearoff_log, macos_window_log};
use restart_trigger::RestartTrigger;
use streaming_images::UploadedImageEntry;
use window::{
    DockTearoffFollow, PendingFrontRequest, TimerEntry, WindowRuntime, bring_window_to_front,
};

pub struct WinitRunner<D: WinitAppDriver> {
    pub config: WinitRunnerConfig,
    pub app: App,
    pub driver: D,
    dispatcher: DesktopDispatcher,
    event_loop_proxy: Option<EventLoopProxy>,
    proxy_events: Arc<Mutex<Vec<RunnerUserEvent>>>,
    is_suspended: bool,
    driver_initialized: bool,
    wgpu_init_blocked: bool,
    #[cfg(target_os = "android")]
    android_app: Option<winit::platform::android::activity::AndroidApp>,

    renderdoc: Option<RenderDocCapture>,
    context: Option<WgpuContext>,
    renderer: Option<Renderer>,
    renderer_caps: Option<fret_render::RendererCapabilities>,
    system_font_rescan_result: Arc<Mutex<Option<fret_render::SystemFontRescanResult>>>,
    system_font_rescan_in_flight: bool,
    system_font_rescan_pending: bool,
    last_window_surface_sizes: HashMap<fret_core::AppWindowId, (u32, u32)>,
    last_window_surface_size_changed_at: Option<Instant>,
    no_services: NoUiServices,
    diag_bundle_screenshots: DiagBundleScreenshotCapture,
    #[cfg(feature = "webview-wry")]
    webviews_wry: fret_webview_wry::wry_host::WryWebViewHost,

    windows: SlotMap<fret_core::AppWindowId, WindowRuntime<D::WindowState>>,
    window_registry: fret_runner_winit::window_registry::WinitWindowRegistry,
    main_window: Option<fret_core::AppWindowId>,
    menu_bar: Option<fret_runtime::MenuBar>,
    windows_pending_front: HashMap<fret_core::AppWindowId, PendingFrontRequest>,
    /// Best-effort z-order for windows (most recently focused last).
    ///
    /// This is used as a tie-breaker when multiple windows overlap the cursor and the platform
    /// cannot provide reliable z-order/hover routing.
    windows_z_order: Vec<fret_core::AppWindowId>,

    /// True if this event-loop turn already observed a left mouse release via `WindowEvent`.
    /// On macOS we may also see the same release as a `DeviceEvent`, so this prevents double-drop.
    saw_left_mouse_release_this_turn: bool,
    left_mouse_down: bool,
    dock_tearoff_follow: Option<DockTearoffFollow>,
    dock_floating_windows: HashSet<fret_core::AppWindowId>,

    tick_id: TickId,
    frame_id: FrameId,

    next_environment_poll_at: Instant,

    #[cfg(target_os = "linux")]
    linux_portal_settings_listener_started: bool,

    raf_windows: HashSet<fret_core::AppWindowId>,
    timers: HashMap<fret_runtime::TimerToken, TimerEntry>,
    clipboard: NativeClipboard,
    diag_clipboard_force_unavailable_windows: HashSet<fret_core::AppWindowId>,
    open_url: NativeOpenUrl,
    file_dialog: NativeFileDialog,
    diag_incoming_open_next_token: u64,
    diag_incoming_open_payloads: HashMap<fret_core::IncomingOpenToken, DiagIncomingOpenPayload>,
    startup_incoming_open_paths: Vec<std::path::PathBuf>,
    startup_incoming_open_delivered: bool,
    incoming_open_path_payloads: HashMap<fret_core::IncomingOpenToken, IncomingOpenPathPayload>,
    #[cfg(target_os = "ios")]
    ios_keyboard: Option<ios_keyboard::IosKeyboardTracker>,
    diag_window_insets_overrides: HashMap<fret_core::AppWindowId, DiagWindowInsetsOverride>,
    diag_cursor_screen_pos_override: Option<diag_cursor_override::DiagCursorScreenPosOverride>,
    cursor_screen_pos: Option<PhysicalPosition<f64>>,
    #[cfg(target_os = "macos")]
    macos_cursor_transform: MacCursorTransformTable,
    internal_drag_hover_window: Option<fret_core::AppWindowId>,
    internal_drag_hover_pos: Option<Point>,
    internal_drag_pointer_id: Option<fret_core::PointerId>,

    external_drop: NativeExternalDrop,

    uploaded_images: HashMap<fret_core::ImageId, UploadedImageEntry>,
    streaming_uploads: StreamingUploadQueue,
    nv12_gpu: Option<super::super::yuv_gpu::Nv12GpuConverter>,

    #[cfg(feature = "dev-state")]
    dev_state: dev_state::DevStateController,

    #[cfg(feature = "hotpatch-subsecond")]
    hotpatch: Option<HotpatchTrigger>,
    #[cfg(feature = "hotpatch-subsecond")]
    hot_reload_generation: u64,

    watch_restart_trigger: Option<RestartTrigger>,
    watch_restart_requested: bool,

    #[cfg(feature = "diag-screenshots")]
    diag_screenshots: Option<diag_screenshots::DiagScreenshotCapture>,
}

impl<D: WinitAppDriver> WinitRunner<D> {
    #[cfg(target_os = "android")]
    fn set_android_app(&mut self, app: winit::platform::android::activity::AndroidApp) {
        self.android_app = Some(app);
    }

    #[cfg(target_os = "android")]
    fn android_force_soft_input(&self, enabled: bool) {
        let Some(app) = self.android_app.as_ref() else {
            return;
        };

        // Some OEM builds appear to ignore "implicit" IME show requests. When a text input is
        // focused we want the keyboard to appear reliably.
        if enabled {
            app.show_soft_input(false);
        } else {
            app.hide_soft_input(false);
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
                // in-window floating fallback over OS tear-off UX (ADR 0054 / ADR 0083).
                if platform_prefs::linux_is_wayland_session() {
                    caps.ui.window_tear_off = false;
                    caps.ui.window_hover_detection =
                        fret_runtime::WindowHoverDetectionQuality::None;
                    caps.ui.window_z_level = fret_runtime::WindowZLevelQuality::None;
                }
            }

            caps.clipboard.text.read = true;
            caps.clipboard.text.write = true;
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

            caps.clipboard.text.read = false;
            caps.clipboard.text.write = false;
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

            caps.clipboard.text.read = false;
            caps.clipboard.text.write = false;
            caps.clipboard.files = false;

            caps.dnd.external = false;
            caps.dnd.external_payload = ExternalDragPayloadKind::None;
            caps.dnd.external_position = ExternalDragPositionQuality::None;

            caps.ime.enabled = true;
            caps.ime.set_cursor_area = true;

            caps.fs.real_paths = false;
            caps.fs.file_dialogs = false;

            caps.shell.open_url = false;

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

        caps.clipboard.text.read &= available.clipboard.text.read;
        caps.clipboard.text.write &= available.clipboard.text.write;
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
        #[cfg(target_os = "linux")]
        let linux_settings_waker = proxy.clone();

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

        #[cfg(target_os = "linux")]
        self.maybe_start_linux_portal_settings_listener(linux_settings_waker);
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
        self.system_font_rescan_in_flight = false;
        self.system_font_rescan_pending = false;
        self.last_window_surface_sizes.clear();
        self.last_window_surface_size_changed_at = None;
        self.publish_system_font_rescan_state();
        if let Ok(mut slot) = self.system_font_rescan_result.lock() {
            *slot = None;
        }

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

    fn poll_watch_restart_trigger(&mut self, now: Instant) -> bool {
        let Some(trigger) = self.watch_restart_trigger.as_mut() else {
            return false;
        };
        if self.watch_restart_requested {
            return false;
        }
        if now < trigger.next_poll_at() {
            return false;
        }
        if !trigger.poll(now) {
            return false;
        }

        self.watch_restart_requested = true;
        tracing::info!("watch_restart: trigger file changed (requesting quit)");
        self.app.push_effect(Effect::QuitApp);
        true
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
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub fn new_app(config: WinitRunnerConfig, app: App, driver: D) -> Self {
        Self::new(config, app, driver)
    }
}

impl<D: WinitAppDriver> WinitRunner<D> {
    fn allocate_incoming_open_token(&mut self) -> fret_core::IncomingOpenToken {
        let token = fret_core::IncomingOpenToken(self.diag_incoming_open_next_token);
        self.diag_incoming_open_next_token = self.diag_incoming_open_next_token.saturating_add(1);
        token
    }

    fn maybe_deliver_startup_incoming_open(&mut self, window: fret_core::AppWindowId) {
        if self.startup_incoming_open_delivered {
            return;
        }
        self.startup_incoming_open_delivered = true;

        if self.startup_incoming_open_paths.is_empty() {
            return;
        }

        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if !caps.shell.incoming_open {
            return;
        }

        let token = self.allocate_incoming_open_token();

        let mut items: Vec<fret_core::IncomingOpenItem> = Vec::new();
        for path in self.startup_incoming_open_paths.iter() {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "file".to_string());
            let size_bytes = std::fs::metadata(path).ok().map(|m| m.len());
            items.push(fret_core::IncomingOpenItem::File(
                fret_core::ExternalDragFile {
                    name,
                    size_bytes,
                    media_type: None,
                },
            ));
        }

        self.incoming_open_path_payloads.insert(
            token,
            IncomingOpenPathPayload {
                paths: std::mem::take(&mut self.startup_incoming_open_paths),
            },
        );
        self.deliver_window_event_now(window, &Event::IncomingOpenRequest { token, items });
    }
}
