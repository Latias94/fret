//! Desktop launcher implementation (winit + wgpu).

pub use super::super::common::*;

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

pub(super) fn diag_dock_drag_trace(args: std::fmt::Arguments<'_>) {
    use std::{
        io::Write as _,
        sync::{Mutex as StdMutex, OnceLock},
    };

    if std::env::var_os("FRET_DOCK_DRAG_TRACE").is_none() {
        return;
    }

    static LOG_FILE: OnceLock<StdMutex<std::fs::File>> = OnceLock::new();
    let file = LOG_FILE.get_or_init(|| {
        let out_dir = std::env::var_os("FRET_DIAG_DIR")
            .filter(|v| !v.is_empty())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("target").join("fret-diag"));
        let _ = std::fs::create_dir_all(&out_dir);
        let path = out_dir.join("dock_drag_runtime_trace.log");
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("open dock_drag_runtime_trace.log");
        StdMutex::new(file)
    });
    let Ok(mut file) = file.lock() else {
        return;
    };
    let _ = writeln!(file, "{}", args);
}

#[cfg(feature = "hotpatch-subsecond")]
mod hotpatch;

use fret_app::{App, CreateWindowKind, CreateWindowRequest, Effect};
use fret_core::time::{Duration, Instant};
use fret_core::{
    Event, InternalDragEvent, InternalDragKind, Point, Px, Rect, Size, UiServices,
    WindowMetricsService,
};
use fret_platform_native::clipboard::NativeClipboard;
use fret_platform_native::external_drop::NativeExternalDrop;
use fret_platform_native::file_dialog::NativeFileDialog;
use fret_platform_native::open_url::NativeOpenUrl;
use fret_render::{Renderer, SurfaceState, WgpuContext};
use fret_runner_winit::accessibility;
use fret_runtime::{
    FrameId, PlatformCapabilities, PlatformCompletion, TickId, WindowStyleRequest, WindowZLevel,
};
use slotmap::SlotMap;
use tracing::error;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, Position},
    event::{DeviceEvent, ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoopProxy},
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

#[derive(Debug, Default, Clone)]
struct DiagWindowPreferenceOverride {
    /// `None` means "no override".
    ///
    /// `Some(None)` means "known-but-none" (cleared).
    ///
    /// `Some(Some(v))` means "override to v".
    color_scheme: Option<Option<fret_core::ColorScheme>>,
    /// See `color_scheme`.
    prefers_reduced_motion: Option<Option<bool>>,
    /// See `color_scheme`.
    text_scale_factor: Option<Option<f32>>,
}

mod app_handler;
mod asset_reload;
#[cfg(feature = "dev-state")]
mod dev_state;
mod device_events;
mod diag_bundle_screenshots;
mod diag_cursor_override;
mod diag_mouse_buttons_override;
#[cfg(feature = "diag-screenshots")]
mod diag_screenshots;
mod diag_wheel_burst_inject;
mod dispatcher;
mod docking;
mod effects;
mod event_routing;
#[cfg(target_os = "ios")]
mod ios_keyboard;
#[cfg(all(target_os = "macos", feature = "macos-hit-test-regions"))]
mod macos_hit_test;
#[cfg(target_os = "macos")]
mod macos_menu;
mod no_services;
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
mod renderdoc_capture;
mod restart_trigger;
mod scheduling_diagnostics;
#[cfg(windows)]
mod windows_menu;

mod change_propagation;
mod clipboard_effects;
mod command_effects;
mod cursor_effects;
mod driver_effects;
mod effect_queue;
mod event_loop;
mod file_transfer_effects;
mod frame_effects;
mod image_effects;
mod ime_effects;
mod incoming_open_effects;
mod macos_cursor;
mod menu_effects;
mod monitor_topology;
mod platform_capabilities;
mod platform_prefs;
mod quit_effects;
mod redraw_hitch;
mod render;
mod renderer_bootstrap;
mod run;
mod shell_effects;
mod streaming_effects;
mod streaming_images;
mod surface_lifecycle;
mod text_effects;
mod timers;
mod webview;
mod wgpu_adapter_diagnostics;
mod wheel_coalescing;
#[cfg(target_os = "windows")]
mod win32;
mod window;
mod window_close;
mod window_create_request;
mod window_external_drag;
mod window_front;
mod window_geometry;
mod window_insert;
mod window_mapped_events;
mod window_metrics;
#[cfg(all(target_os = "macos", feature = "macos-hit-test-regions"))]
mod window_moved_events;
mod window_os_create;
mod window_platform;
mod window_pointer_button;
mod window_pointer_move;
mod window_position;
mod window_pre_dispatch_events;
mod window_redraw_accessibility;
mod window_redraw_renderer_perf;
mod window_redraw_text_input;
mod window_requests;
mod window_state_events;
mod window_style;
mod window_turn;
mod window_under_cursor;

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
use webview::RunnerWebViewState;

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
use window::{DockTearoffFollow, TimerEntry, WindowRuntime};
use window_front::PendingFrontRequest;
use window_platform::bring_window_to_front;

pub struct WinitRunner<D: WinitAppDriver> {
    pub config: WinitRunnerConfig,
    pub app: App,
    pub driver: D,
    asset_reload: Option<asset_reload::AssetReloadController>,
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
    webviews: RunnerWebViewState,

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
    dock_drag_pointer_capture: Option<(fret_core::PointerId, fret_core::AppWindowId)>,

    tick_id: TickId,
    frame_id: FrameId,

    next_environment_poll_at: Instant,

    #[cfg(target_os = "linux")]
    linux_portal_settings_listener_started: bool,

    raf_windows: crate::runner::common::frame_requests::AnimationFrameRequests,
    next_raf_deadline: Option<Instant>,
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
    diag_window_preference_overrides: HashMap<fret_core::AppWindowId, DiagWindowPreferenceOverride>,
    diag_cursor_screen_pos_override: Option<diag_cursor_override::DiagCursorScreenPosOverride>,
    diag_last_cursor_override_tick: Option<TickId>,
    diag_mouse_buttons_override: Option<diag_mouse_buttons_override::DiagMouseButtonsOverride>,
    diag_last_mouse_buttons_override_tick: Option<TickId>,
    diag_mouse_buttons_override_active: bool,
    diag_wheel_burst_inject: Option<diag_wheel_burst_inject::DiagWheelBurstInject>,
    diag_isolate_pointer_input: bool,
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

    #[cfg(target_os = "windows")]
    pub(super) fn refresh_platform_window_receiver_at_cursor_diagnostics(&mut self) {
        use fret_runtime::{
            RunnerPlatformWindowReceiverAtCursorSnapshotV1,
            RunnerPlatformWindowReceiverAtCursorSourceV1,
            RunnerPlatformWindowReceiverDiagnosticsStore,
        };
        use std::collections::HashMap;
        use winit::raw_window_handle::{HasWindowHandle as _, RawWindowHandle};

        let mut hwnd_to_window: HashMap<isize, fret_core::AppWindowId> = HashMap::new();
        for (window, state) in self.windows.iter() {
            let Ok(handle) = state.window.window_handle() else {
                continue;
            };
            let RawWindowHandle::Win32(handle) = handle.as_raw() else {
                continue;
            };
            let hwnd = win32::root_hwnd(handle.hwnd.get());
            hwnd_to_window.insert(hwnd, window);
        }

        let receiver_window = self
            .cursor_screen_pos
            .and_then(win32::window_under_cursor_root)
            .and_then(|hwnd| hwnd_to_window.get(&hwnd).copied());

        let snapshot = RunnerPlatformWindowReceiverAtCursorSnapshotV1 {
            receiver_window,
            source: RunnerPlatformWindowReceiverAtCursorSourceV1::Win32WindowFromPoint,
        };
        self.app.with_global_mut(
            RunnerPlatformWindowReceiverDiagnosticsStore::default,
            |store, _app| {
                store.set_latest_at_cursor(snapshot);
            },
        );
    }

    #[cfg(target_os = "macos")]
    pub(super) fn refresh_platform_window_receiver_at_cursor_diagnostics(&mut self) {
        use fret_runtime::{
            RunnerPlatformWindowReceiverAtCursorSnapshotV1,
            RunnerPlatformWindowReceiverAtCursorSourceV1,
            RunnerPlatformWindowReceiverDiagnosticsStore, WindowHitTestRegionV1,
            WindowHitTestRequestV1,
        };

        fn regions_contain_point(regions: &[WindowHitTestRegionV1], px: f32, py: f32) -> bool {
            fn rect_contains(x: f32, y: f32, w: f32, h: f32, px: f32, py: f32) -> bool {
                let x = if x.is_finite() { x } else { 0.0 };
                let y = if y.is_finite() { y } else { 0.0 };
                let w = if w.is_finite() { w.max(0.0) } else { 0.0 };
                let h = if h.is_finite() { h.max(0.0) } else { 0.0 };
                if w <= 0.0 || h <= 0.0 {
                    return false;
                }
                px >= x && px < x + w && py >= y && py < y + h
            }

            fn rrect_contains(x: f32, y: f32, w: f32, h: f32, r: f32, px: f32, py: f32) -> bool {
                if !rect_contains(x, y, w, h, px, py) {
                    return false;
                }
                let r = if r.is_finite() { r.max(0.0) } else { 0.0 };
                if r <= 0.0 {
                    return true;
                }

                let max_r = 0.5 * w.min(h);
                let r = r.min(max_r);

                let left = x + r;
                let right = x + w - r;
                let top = y + r;
                let bottom = y + h - r;

                if (px >= left && px < right) || (py >= top && py < bottom) {
                    return true;
                }

                let cx = if px < left { left } else { right };
                let cy = if py < top { top } else { bottom };
                let dx = px - cx;
                let dy = py - cy;
                dx * dx + dy * dy <= r * r
            }

            for r in regions {
                match *r {
                    WindowHitTestRegionV1::Rect {
                        x,
                        y,
                        width,
                        height,
                    } => {
                        if rect_contains(x, y, width, height, px, py) {
                            return true;
                        }
                    }
                    WindowHitTestRegionV1::RRect {
                        x,
                        y,
                        width,
                        height,
                        radius,
                    } => {
                        if rrect_contains(x, y, width, height, radius, px, py) {
                            return true;
                        }
                    }
                }
            }
            false
        }

        let style_store = self
            .app
            .global::<fret_runtime::RunnerWindowStyleDiagnosticsStore>();

        let receiver_window = self.cursor_screen_pos.and_then(|screen_pos| {
            // Best-effort: prefer the runner's known z-order, since AppKit ordering can diverge for
            // non-activating/auxiliary windows. This is a scripted-gate probe, not an OS truth.
            for &window in self.windows_z_order.iter().rev() {
                if !self.screen_pos_in_window(window, screen_pos) {
                    continue;
                }
                let local = self.local_pos_for_window(window, screen_pos)?;
                let hit_test = style_store
                    .and_then(|s| s.effective_snapshot(window))
                    .map(|s| s.hit_test)
                    .unwrap_or(WindowHitTestRequestV1::Normal);

                let interactive = match hit_test {
                    WindowHitTestRequestV1::Normal => true,
                    WindowHitTestRequestV1::PassthroughAll => false,
                    WindowHitTestRequestV1::PassthroughRegions { regions } => {
                        regions_contain_point(&regions, local.x.0, local.y.0)
                    }
                };
                if interactive {
                    return Some(window);
                }
            }

            // Fallback for drifted z-order (should be rare, but keeps the probe deterministic).
            for window in self.windows.keys() {
                if self.windows_z_order.contains(&window) {
                    continue;
                }
                if !self.screen_pos_in_window(window, screen_pos) {
                    continue;
                }
                let local = self.local_pos_for_window(window, screen_pos)?;
                let hit_test = style_store
                    .and_then(|s| s.effective_snapshot(window))
                    .map(|s| s.hit_test)
                    .unwrap_or(WindowHitTestRequestV1::Normal);

                let interactive = match hit_test {
                    WindowHitTestRequestV1::Normal => true,
                    WindowHitTestRequestV1::PassthroughAll => false,
                    WindowHitTestRequestV1::PassthroughRegions { regions } => {
                        regions_contain_point(&regions, local.x.0, local.y.0)
                    }
                };
                if interactive {
                    return Some(window);
                }
            }
            None
        });

        let snapshot = RunnerPlatformWindowReceiverAtCursorSnapshotV1 {
            receiver_window,
            source: RunnerPlatformWindowReceiverAtCursorSourceV1::MacosOrderedWindowsBestEffort,
        };
        self.app.with_global_mut(
            RunnerPlatformWindowReceiverDiagnosticsStore::default,
            |store, _app| {
                store.set_latest_at_cursor(snapshot);
            },
        );
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
        #[cfg(all(target_os = "macos", feature = "macos-hit-test-regions"))]
        macos_hit_test::set_event_loop_proxy(proxy.clone(), self.proxy_events.clone());
        self.dispatcher.set_event_loop_proxy(proxy.clone());
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        if let Some(asset_reload) = self.asset_reload.as_mut() {
            asset_reload.set_event_loop_proxy(
                &mut self.app,
                Instant::now(),
                &mut self.timers,
                proxy.clone(),
                self.proxy_events.clone(),
            );
        }
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

            state.last_semantics_snapshot = None;
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

        let _ = window;
        self.left_mouse_down
    }

    fn diag_pointer_input_isolation_active(&self) -> bool {
        if !self.diag_isolate_pointer_input {
            return false;
        }
        self.diag_cursor_screen_pos_override.is_some() || self.diag_mouse_buttons_override.is_some()
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
