//! Desktop launcher implementation (winit + wgpu).

pub use super::super::common::*;

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[cfg(feature = "hotpatch-subsecond")]
mod hotpatch;

use fret_app::{App, CreateWindowKind, CreateWindowRequest};
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

type WindowAnchor = fret_core::WindowAnchor;

mod app_handler;
mod diag_bundle_screenshots;
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
    MacCursorScreenKey, MacCursorTransform, MacCursorTransformTable, macos_cursor_trace_enabled,
    macos_dockfloating_parenting_enabled, macos_is_left_mouse_down, macos_mouse_location,
};
use macos_cursor::{dock_tearoff_log, macos_window_log};
#[cfg(target_os = "linux")]
use platform_prefs::LINUX_PORTAL_ENV_DIRTY;
use render::validate_scene_if_enabled;
use streaming_images::UploadedImageEntry;
use window::{
    DockTearoffFollow, MonitorRectF64, PendingFrontRequest, TimerEntry, WindowRuntime,
    bring_window_to_front, client_origin_screen, local_pos_for_screen_pos,
    outer_pos_for_cursor_grab, screen_pos_in_client,
};

pub struct WinitRunner<D: WinitAppDriver> {
    pub config: WinitRunnerConfig,
    pub app: App,
    pub driver: D,
    dispatcher: DesktopDispatcher,
    event_loop_proxy: Option<EventLoopProxy>,
    proxy_events: Arc<Mutex<Vec<RunnerUserEvent>>>,
    is_suspended: bool,
    #[cfg(target_os = "android")]
    android_app: Option<winit::platform::android::activity::AndroidApp>,

    renderdoc: Option<RenderDocCapture>,
    context: Option<WgpuContext>,
    renderer: Option<Renderer>,
    renderer_caps: Option<fret_render::RendererCapabilities>,
    system_font_rescan_result: Arc<Mutex<Option<fret_render::SystemFontRescanResult>>>,
    system_font_rescan_in_flight: bool,
    system_font_rescan_pending: bool,
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

    tick_id: TickId,
    frame_id: FrameId,

    next_environment_poll_at: Instant,

    #[cfg(target_os = "linux")]
    linux_portal_settings_listener_started: bool,

    raf_windows: HashSet<fret_core::AppWindowId>,
    timers: HashMap<fret_runtime::TimerToken, TimerEntry>,
    clipboard: NativeClipboard,
    open_url: NativeOpenUrl,
    file_dialog: NativeFileDialog,
    #[cfg(target_os = "ios")]
    ios_keyboard: Option<ios_keyboard::IosKeyboardTracker>,
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

    #[cfg(feature = "hotpatch-subsecond")]
    hotpatch: Option<HotpatchTrigger>,
    #[cfg(feature = "hotpatch-subsecond")]
    hot_reload_generation: u64,

    #[cfg(feature = "diag-screenshots")]
    diag_screenshots: Option<diag_screenshots::DiagScreenshotCapture>,
}

impl<D: WinitAppDriver> WinitRunner<D> {
    const WINDOW_VISIBILITY_PADDING_PX: f64 = 40.0;

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

            caps.clipboard.text = false;
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

    #[cfg(target_os = "linux")]
    fn maybe_start_linux_portal_settings_listener(&mut self, waker: EventLoopProxy) {
        if self.linux_portal_settings_listener_started {
            return;
        }
        self.linux_portal_settings_listener_started = true;

        std::thread::spawn(move || {
            use zbus::blocking::{Connection, Proxy};

            const SETTINGS_SERVICE: &str = "org.freedesktop.portal.Desktop";
            const SETTINGS_PATH: &str = "/org/freedesktop/portal/desktop";
            const SETTINGS_INTERFACE: &str = "org.freedesktop.portal.Settings";

            let Ok(connection) = Connection::session() else {
                return;
            };
            let Ok(proxy) = Proxy::new(
                &connection,
                SETTINGS_SERVICE,
                SETTINGS_PATH,
                SETTINGS_INTERFACE,
            ) else {
                return;
            };
            let Ok(signals) = proxy.receive_signal("SettingChanged") else {
                return;
            };

            for msg in signals {
                let Ok((namespace, key, _value)) =
                    msg.body()
                        .deserialize::<(String, String, zbus::zvariant::OwnedValue)>()
                else {
                    continue;
                };
                if namespace != linux_portal_settings::APPEARANCE_NAMESPACE {
                    continue;
                }
                if !matches!(
                    key.as_str(),
                    "color-scheme" | "contrast" | "reduce-motion" | "reduced-motion"
                ) {
                    continue;
                }
                if LINUX_PORTAL_ENV_DIRTY.swap(true, std::sync::atomic::Ordering::SeqCst) {
                    continue;
                }
                waker.wake_up();
            }
        });
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
            && let Some(state) = self.windows.get(current)
        {
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
        for &w in self.windows_z_order.iter().rev() {
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
        // Fallback if the z-order list has drifted.
        for w in self.windows.keys() {
            if self.windows_z_order.iter().any(|tracked| *tracked == w) {
                continue;
            }
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

    fn bump_window_z_order(&mut self, window: fret_core::AppWindowId) {
        if self.windows.get(window).is_none() {
            return;
        }
        self.windows_z_order.retain(|w| *w != window);
        self.windows_z_order.push(window);

        #[cfg(target_os = "macos")]
        {
            self.enqueue_window_front(window, None, None, Instant::now());
        }
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

#[cfg(target_os = "linux")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::{PhysicalPosition, PhysicalSize};

    #[test]
    #[cfg(target_os = "linux")]
    fn is_wayland_session_true_for_xdg_session_type_wayland() {
        assert!(is_wayland_session(Some("wayland"), None));
        assert!(is_wayland_session(Some("Wayland"), None));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn is_wayland_session_true_for_wayland_display() {
        assert!(is_wayland_session(None, Some("wayland-0")));
    }

    #[test]
    #[cfg(target_os = "linux")]
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
