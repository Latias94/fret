use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};

use fret_app::{App, CreateWindowKind, CreateWindowRequest, Effect, WindowRequest};
use fret_core::{
    Event, ExternalDragEvent, ExternalDragFile, ExternalDragFiles, ExternalDragKind,
    ExternalDropDataEvent, ExternalDropFileData, ExternalDropReadError, ExternalDropToken,
    InternalDragEvent, InternalDragKind, Modifiers, MouseButton, PlatformCapabilities, Point, Px,
    Rect, Scene, Size, UiServices, ViewportInputEvent, WindowMetricsService,
};
use fret_render::{ClearColor, Renderer, SurfaceState, WgpuContext};
use slotmap::SlotMap;
use tracing::error;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, Position},
    event::{
        DeviceEvent, ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent,
    },
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Window, WindowId, WindowLevel},
};

use crate::error::RunnerError;

type WindowAnchor = fret_core::WindowAnchor;

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
fn bring_window_to_front(window: &Window, _sender: Option<&Window>) -> bool {
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

type WgpuFactoryFn =
    dyn FnOnce(Arc<Window>) -> Result<(WgpuContext, wgpu::Surface<'static>), RunnerError> + 'static;

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
        tick_id: fret_core::TickId,
        frame_id: fret_core::FrameId,
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
        _tick_id: fret_core::TickId,
        _frame_id: fret_core::FrameId,
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
}

struct WindowRuntime<S> {
    window: Arc<Window>,
    accessibility: Option<fret_platform::accessibility::WinitAccessibility>,
    surface: SurfaceState<'static>,
    scene: Scene,
    cursor_pos: Point,
    pressed_buttons: fret_core::MouseButtons,
    is_focused: bool,
    ime_allowed: bool,
    cursor_icon: fret_core::CursorIcon,
    external_drag_files: Vec<std::path::PathBuf>,
    external_drag_token: Option<ExternalDropToken>,
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

    context: Option<WgpuContext>,
    renderer: Option<Renderer>,
    no_services: NoUiServices,

    windows: SlotMap<fret_core::AppWindowId, WindowRuntime<D::WindowState>>,
    winit_to_app: HashMap<WindowId, fret_core::AppWindowId>,
    main_window: Option<fret_core::AppWindowId>,
    windows_pending_front: HashMap<fret_core::AppWindowId, PendingFrontRequest>,

    modifiers: Modifiers,
    raw_modifiers: ModifiersState,
    alt_gr_down: bool,
    /// True if this event-loop turn already observed a left mouse release via `WindowEvent`.
    /// On macOS we may also see the same release as a `DeviceEvent`, so this prevents double-drop.
    saw_left_mouse_release_this_turn: bool,
    left_mouse_down: bool,
    dock_tearoff_follow: Option<DockTearoffFollow>,

    tick_id: fret_core::TickId,
    frame_id: fret_core::FrameId,

    raf_windows: HashSet<fret_core::AppWindowId>,
    timers: HashMap<fret_core::TimerToken, TimerEntry>,
    clipboard: Option<arboard::Clipboard>,
    cursor_screen_pos: Option<PhysicalPosition<f64>>,
    internal_drag_hover_window: Option<fret_core::AppWindowId>,
    internal_drag_hover_pos: Option<Point>,

    external_drop_next_token: u64,
    external_drop_payloads: HashMap<ExternalDropToken, Vec<std::path::PathBuf>>,
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

    fn virtual_desktop_bounds(window: &Window) -> Option<MonitorRectF64> {
        let mut monitors = window.available_monitors();
        let first = monitors.next()?;

        let first_pos = first.position();
        let first_size = first.size();
        let mut min_x = first_pos.x as f64;
        let mut min_y = first_pos.y as f64;
        let mut max_x = first_pos.x as f64 + first_size.width as f64;
        let mut max_y = first_pos.y as f64 + first_size.height as f64;

        for monitor in monitors {
            let pos = monitor.position();
            let size = monitor.size();
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

    fn monitor_rects_physical(window: &Window) -> Vec<MonitorRectF64> {
        window
            .available_monitors()
            .map(|m| {
                let pos = m.position();
                let size = m.size();
                MonitorRectF64 {
                    min_x: pos.x as f64,
                    min_y: pos.y as f64,
                    max_x: pos.x as f64 + size.width as f64,
                    max_y: pos.y as f64 + size.height as f64,
                }
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
        window: &Window,
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

    fn map_cursor_icon(icon: fret_core::CursorIcon) -> winit::window::CursorIcon {
        match icon {
            fret_core::CursorIcon::Default => winit::window::CursorIcon::Default,
            fret_core::CursorIcon::Pointer => winit::window::CursorIcon::Pointer,
            fret_core::CursorIcon::Text => winit::window::CursorIcon::Text,
            fret_core::CursorIcon::ColResize => winit::window::CursorIcon::ColResize,
            fret_core::CursorIcon::RowResize => winit::window::CursorIcon::RowResize,
        }
    }

    pub fn new(config: WinitRunnerConfig, app: App, driver: D) -> Self {
        let mut app = app;
        let caps = match app.global::<PlatformCapabilities>().cloned() {
            Some(caps) => caps,
            None => {
                let caps = PlatformCapabilities::default();
                app.set_global(caps.clone());
                caps
            }
        };
        tracing::info!(caps = ?caps, "platform capabilities");

        let raw_modifiers = ModifiersState::empty();
        let alt_gr_down = false;
        Self {
            config,
            app,
            driver,
            context: None,
            renderer: None,
            no_services: NoUiServices,
            windows: SlotMap::with_key(),
            winit_to_app: HashMap::new(),
            main_window: None,
            windows_pending_front: HashMap::new(),
            modifiers: map_modifiers(raw_modifiers, alt_gr_down),
            raw_modifiers,
            alt_gr_down,
            saw_left_mouse_release_this_turn: false,
            left_mouse_down: false,
            dock_tearoff_follow: None,
            tick_id: fret_core::TickId::default(),
            frame_id: fret_core::FrameId::default(),
            raf_windows: HashSet::new(),
            timers: HashMap::new(),
            clipboard: arboard::Clipboard::new().ok(),
            cursor_screen_pos: None,
            internal_drag_hover_window: None,
            internal_drag_hover_pos: None,
            external_drop_next_token: 1,
            external_drop_payloads: HashMap::new(),
        }
    }

    fn allocate_external_drop_token(&mut self) -> ExternalDropToken {
        let token = ExternalDropToken(self.external_drop_next_token);
        self.external_drop_next_token = self.external_drop_next_token.saturating_add(1);
        token
    }

    fn external_drag_files(
        token: ExternalDropToken,
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
        event_loop: &ActiveEventLoop,
        spec: WindowCreateSpec,
    ) -> Result<
        (
            Arc<Window>,
            Option<fret_platform::accessibility::WinitAccessibility>,
        ),
        RunnerError,
    > {
        let mut attrs = Window::default_attributes()
            .with_title(spec.title)
            .with_inner_size(spec.size)
            .with_visible(if self.config.accessibility_enabled {
                false
            } else {
                spec.visible
            });
        if let Some(position) = spec.position {
            attrs = attrs.with_position(position);
        }
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .map_err(|source| RunnerError::CreateWindowFailed { source })?,
        );

        macos_window_log(format_args!("[create] winit={:?}", window.id()));

        let accessibility = self
            .config
            .accessibility_enabled
            .then(|| fret_platform::accessibility::WinitAccessibility::new(event_loop, &window));

        if self.config.accessibility_enabled && spec.visible {
            window.set_visible(true);
        }

        Ok((window, accessibility))
    }

    fn insert_window(
        &mut self,
        window: Arc<Window>,
        accessibility: Option<fret_platform::accessibility::WinitAccessibility>,
        surface: wgpu::Surface<'static>,
    ) -> Result<fret_core::AppWindowId, RunnerError> {
        let Some(context) = self.context.as_ref() else {
            return Err(RunnerError::WgpuNotInitialized);
        };

        let size = window.inner_size();
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
                surface,
                scene: Scene::default(),
                cursor_pos: Point::new(Px(0.0), Px(0.0)),
                pressed_buttons: fret_core::MouseButtons::default(),
                is_focused: false,
                ime_allowed: false,
                cursor_icon: fret_core::CursorIcon::Default,
                external_drag_files: Vec::new(),
                external_drag_token: None,
                user,
            }
        });

        if let Some(state) = self.windows.get(id) {
            let size_phys = state.window.inner_size();
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
        self.winit_to_app.insert(winit_id, id);
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
            self.winit_to_app.remove(&state.window.id());
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
        // Use the client-area origin as the base; `WindowAnchor::position` is expressed in
        // window-local logical coordinates (matching pointer events), which aligns with
        // `inner_position` rather than `outer_position` (decorations).
        let inner = anchor_state.window.inner_position().ok()?;
        let scale = anchor_state.window.scale_factor();

        let (ox, oy) = self.config.new_window_anchor_offset;
        let mut x = inner.x as f64 + anchor.position.x.0 as f64 * scale + ox;
        let mut y = inner.y as f64 + anchor.position.y.0 as f64 * scale + oy;

        // Best-effort clamping: avoid creating "off-screen" floating windows due to
        // platform-specific coordinate spaces and DPI conversions.
        if let Some(monitor) = anchor_state.window.current_monitor() {
            let pos = monitor.position();
            let size = monitor.size();

            let min_x = pos.x as f64;
            let min_y = pos.y as f64;
            // Leave a small margin so the window stays reachable even if its size is larger than
            // the monitor work area.
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

        if let Some(monitor) = ref_state.window.current_monitor() {
            let pos = monitor.position();
            let size = monitor.size();

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
        let target_inner = state.window.inner_size();
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
        let deco_offset = if let (Ok(outer), Ok(inner)) =
            (state.window.outer_position(), state.window.inner_position())
        {
            (inner.x - outer.x, inner.y - outer.y)
        } else {
            (0, 0)
        };

        let grab_client_x = grab_x as f64 * scale;
        let grab_client_y = grab_y as f64 * scale;
        let grab_outer_x = deco_offset.0 as f64 + grab_client_x;
        let grab_outer_y = deco_offset.1 as f64 + grab_client_y;

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
        event_loop: &ActiveEventLoop,
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
        let mut due: Vec<fret_core::TimerToken> = Vec::new();
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

    fn drain_effects(&mut self, event_loop: &ActiveEventLoop) {
        const MAX_EFFECT_DRAIN_TURNS: usize = 8;

        for _ in 0..MAX_EFFECT_DRAIN_TURNS {
            let now = Instant::now();
            let effects = self.app.flush_effects();

            let mut did_work = !effects.is_empty();

            for effect in effects {
                match effect {
                    Effect::Redraw(window) => {
                        if let Some(state) = self.windows.get(window) {
                            state.window.request_redraw();
                        }
                    }
                    Effect::ImeAllow { window, enabled } => {
                        if let Some(state) = self.windows.get_mut(window) {
                            state.window.set_ime_allowed(enabled);
                            state.ime_allowed = enabled;
                        }
                    }
                    Effect::ImeSetCursorArea { window, rect } => {
                        if let Some(state) = self.windows.get(window) {
                            state.window.set_ime_cursor_area(
                                winit::dpi::LogicalPosition::new(rect.origin.x.0, rect.origin.y.0),
                                winit::dpi::LogicalSize::new(rect.size.width.0, rect.size.height.0),
                            );
                        }
                    }
                    Effect::CursorSetIcon { window, icon } => {
                        let Some(state) = self.windows.get_mut(window) else {
                            continue;
                        };
                        if state.cursor_icon == icon {
                            continue;
                        }
                        state.cursor_icon = icon;
                        state.window.set_cursor(Self::map_cursor_icon(icon));
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
                        let Some(clipboard) = self.clipboard.as_mut() else {
                            continue;
                        };
                        if let Err(err) = clipboard.set_text(text) {
                            tracing::debug!(?err, "failed to set clipboard text");
                        }
                    }
                    Effect::ClipboardGetText { window } => {
                        let Some(text) = self.clipboard.as_mut().and_then(|cb| cb.get_text().ok())
                        else {
                            continue;
                        };
                        if let Some(state) = self.windows.get_mut(window) {
                            let services =
                                Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                            self.driver.handle_event(
                                &mut self.app,
                                services,
                                window,
                                &mut state.user,
                                &Event::ClipboardText(text),
                            );
                        }
                    }
                    Effect::ExternalDropReadAll { window, token } => {
                        let Some(paths) = self.external_drop_payloads.get(&token).cloned() else {
                            continue;
                        };

                        let mut files: Vec<ExternalDropFileData> = Vec::new();
                        let mut errors: Vec<ExternalDropReadError> = Vec::new();
                        let mut total: u64 = 0;

                        for path in paths.into_iter().take(self.config.external_drop_max_files) {
                            let name = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| path.to_string_lossy().to_string());

                            let meta_len = match std::fs::metadata(&path) {
                                Ok(m) => Some(m.len()),
                                Err(err) => {
                                    errors.push(ExternalDropReadError {
                                        name,
                                        message: format!("metadata failed: {err}"),
                                    });
                                    continue;
                                }
                            };

                            if let Some(len) = meta_len {
                                if len > self.config.external_drop_max_file_bytes {
                                    errors.push(ExternalDropReadError {
                                        name,
                                        message: format!(
                                            "file too large ({} bytes > max {})",
                                            len, self.config.external_drop_max_file_bytes
                                        ),
                                    });
                                    continue;
                                }
                                if total.saturating_add(len)
                                    > self.config.external_drop_max_total_bytes
                                {
                                    errors.push(ExternalDropReadError {
                                        name,
                                        message: format!(
                                            "total size limit exceeded (max {} bytes)",
                                            self.config.external_drop_max_total_bytes
                                        ),
                                    });
                                    break;
                                }
                            }

                            let bytes = match std::fs::read(&path) {
                                Ok(bytes) => bytes,
                                Err(err) => {
                                    errors.push(ExternalDropReadError {
                                        name,
                                        message: format!("read failed: {err}"),
                                    });
                                    continue;
                                }
                            };

                            total = total.saturating_add(bytes.len() as u64);
                            files.push(ExternalDropFileData { name, bytes });
                        }

                        if let Some(state) = self.windows.get_mut(window) {
                            let services =
                                Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                            self.driver.handle_event(
                                &mut self.app,
                                services,
                                window,
                                &mut state.user,
                                &Event::ExternalDropData(ExternalDropDataEvent {
                                    token,
                                    files,
                                    errors,
                                }),
                            );
                        }
                    }
                    Effect::ExternalDropRelease { token } => {
                        self.external_drop_payloads.remove(&token);
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
                                let _ = bring_window_to_front(&state.window, sender_window);
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

            did_work |= self.fire_due_timers(now);
            did_work |= self.clear_internal_drag_hover_if_needed();
            did_work |= self.propagate_model_changes();

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
                let _ = bring_window_to_front(&state.window, sender);
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

    fn dispatch_pointer_event(
        &mut self,
        window: fret_core::AppWindowId,
        pe: fret_core::PointerEvent,
    ) {
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
        self.driver.handle_event(
            &mut self.app,
            services,
            window,
            &mut state.user,
            &Event::Pointer(pe),
        );
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
        let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
        self.driver.handle_event(
            &mut self.app,
            services,
            window,
            &mut state.user,
            &Event::InternalDrag(InternalDragEvent {
                position,
                kind,
                modifiers: self.modifiers,
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
            let _ = bring_window_to_front(&runtime.window, sender);
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
        let inner = state.window.inner_position().ok()?;
        let scale = state.window.scale_factor();
        let x = inner.x as f64 + state.cursor_pos.x.0 as f64 * scale;
        let y = inner.y as f64 + state.cursor_pos.y.0 as f64 * scale;
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
        let Ok(inner) = state.window.inner_position() else {
            return false;
        };
        let size = state.window.inner_size();
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
        let inner = state.window.inner_position().ok()?;
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
            let Ok(inner) = state.window.inner_position() else {
                continue;
            };
            let size = state.window.inner_size();
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

    fn map_wheel_delta(
        config: &WinitRunnerConfig,
        window: &Window,
        delta: MouseScrollDelta,
    ) -> Point {
        match delta {
            MouseScrollDelta::LineDelta(x, y) => Point::new(
                Px(x * config.wheel_line_delta_px),
                Px(y * config.wheel_line_delta_px),
            ),
            MouseScrollDelta::PixelDelta(p) => {
                let logical = p.to_logical::<f32>(window.scale_factor());
                Point::new(
                    Px(logical.x * config.wheel_pixel_delta_scale),
                    Px(logical.y * config.wheel_pixel_delta_scale),
                )
            }
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
                .is_some_and(|w| w.pressed_buttons.left)
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

struct NoUiServices;

impl fret_core::TextService for NoUiServices {
    fn prepare(
        &mut self,
        _text: &str,
        _style: fret_core::TextStyle,
        _constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        (
            fret_core::TextBlobId::default(),
            fret_core::TextMetrics {
                size: fret_core::Size::default(),
                baseline: fret_core::Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for NoUiServices {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for NoUiServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

impl<D: WinitDriver> ApplicationHandler for WinitRunner<D> {
    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if !self.app.drag().is_some_and(|d| d.cross_window_hover)
            && self.dock_tearoff_follow.is_none()
        {
            return;
        }

        match event {
            DeviceEvent::MouseMotion { delta } => {
                #[cfg(target_os = "windows")]
                {
                    if let Some(p) = win32::cursor_pos_physical() {
                        self.cursor_screen_pos = Some(p);
                    } else {
                        let Some(pos) = self.cursor_screen_pos else {
                            return;
                        };
                        self.cursor_screen_pos =
                            Some(PhysicalPosition::new(pos.x + delta.0, pos.y + delta.1));
                    }
                }

                #[cfg(not(target_os = "windows"))]
                {
                    let Some(pos) = self.cursor_screen_pos else {
                        return;
                    };

                    self.cursor_screen_pos =
                        Some(PhysicalPosition::new(pos.x + delta.0, pos.y + delta.1));
                }
                self.route_internal_drag_hover_from_cursor();
                let _ = self.update_dock_tearoff_follow();
                self.drain_effects(event_loop);
            }
            DeviceEvent::Button {
                state: ElementState::Released,
                ..
            } => {
                #[cfg(target_os = "windows")]
                if let Some(p) = win32::cursor_pos_physical() {
                    self.cursor_screen_pos = Some(p);
                }

                // This fallback path is only for releases that occur outside all windows, where
                // winit may not emit `WindowEvent::MouseInput`. When releasing over any window,
                // prefer the regular window event path; otherwise we can incorrectly "force tear-off"
                // even when the user is trying to dock back into another window.
                if let Some(pos) = self.cursor_screen_pos
                    && self.window_under_cursor(pos, None).is_some()
                {
                    return;
                }

                // On macOS, releasing the mouse button outside any window may not deliver a
                // `WindowEvent::MouseInput` to the source window. Use device events to still
                // terminate cross-window dock drags (Unity/ImGui-style tear-off).
                let (source_window, current_window, dragging) = {
                    let Some(drag) = self.app.drag() else {
                        return;
                    };
                    if drag.kind != fret_app::DragKind::DockPanel {
                        return;
                    }
                    (drag.source_window, drag.current_window, drag.dragging)
                };
                dock_tearoff_log(format_args!(
                    "[device-up] source={:?} current={:?} screen_pos={:?} dragging={}",
                    source_window, current_window, self.cursor_screen_pos, dragging
                ));

                #[cfg(target_os = "macos")]
                {
                    if self.saw_left_mouse_release_this_turn || macos_is_left_mouse_down() {
                        return;
                    }
                    if let Some(d) = self.app.drag_mut()
                        && d.kind == fret_app::DragKind::DockPanel
                    {
                        d.dragging = true;
                    }
                    // Route the drop using the current cursor position, so docking into another
                    // window works even when the `MouseInput` event is missing.
                    self.route_internal_drag_drop_from_cursor();
                    dock_tearoff_log(format_args!(
                        "[device-drop] dispatched target={:?}",
                        source_window
                    ));
                }
                if self.app.drag().is_some_and(|d| d.cross_window_hover) {
                    self.app.cancel_drag();
                    let _ = self.clear_internal_drag_hover_if_needed();
                }
                // When a floating dock window is following the cursor, a mouse release may occur
                // outside any window and never produce `WindowEvent::MouseInput`.
                if self.dock_tearoff_follow.is_some() {
                    self.left_mouse_down = false;
                    self.stop_dock_tearoff_follow(Instant::now(), true);
                }
                self.drain_effects(event_loop);
            }
            _ => {}
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.context.is_some() {
            return;
        }

        let spec = self.config.main_window_spec();
        let window = match self.create_os_window(event_loop, spec) {
            Ok(w) => w,
            Err(e) => {
                error!(error = ?e, "failed to create main window");
                return;
            }
        };

        let (context, surface) =
            match std::mem::replace(&mut self.config.wgpu_init, WgpuInit::CreateDefault) {
                WgpuInit::CreateDefault => {
                    match pollster::block_on(WgpuContext::new_with_surface(window.0.clone())) {
                        Ok(v) => v,
                        Err(e) => {
                            error!(error = ?e, "failed to initialize wgpu context");
                            return;
                        }
                    }
                }
                WgpuInit::Provided(context) => {
                    let surface = match context.create_surface(window.0.clone()) {
                        Ok(v) => v,
                        Err(e) => {
                            error!(error = ?e, "failed to create surface from provided context");
                            return;
                        }
                    };
                    (context, surface)
                }
                WgpuInit::Factory(factory) => match factory(window.0.clone()) {
                    Ok(v) => v,
                    Err(e) => {
                        error!(error = ?e, "wgpu factory failed");
                        return;
                    }
                },
            };
        let mut renderer = Renderer::new(&context.adapter, &context.device);
        renderer.set_svg_raster_budget_bytes(self.config.svg_raster_budget_bytes);
        renderer.set_path_msaa_samples(self.config.path_msaa_samples);
        let _ = renderer.set_text_font_families(&self.config.text_font_families);

        self.context = Some(context);
        self.renderer = Some(renderer);
        if let (Some(context), Some(renderer)) = (self.context.as_ref(), self.renderer.as_mut()) {
            self.driver.gpu_ready(&mut self.app, context, renderer);
        }

        let main_window = match self.insert_window(window.0, window.1, surface) {
            Ok(id) => id,
            Err(e) => {
                error!(error = ?e, "failed to insert main window runtime");
                return;
            }
        };
        self.main_window = Some(main_window);
        self.driver.init(&mut self.app, main_window);
        self.app.request_redraw(main_window);
        self.drain_effects(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(app_window) = self.winit_to_app.get(&window_id).copied() else {
            return;
        };

        if let Some(state) = self.windows.get_mut(app_window)
            && let Some(a11y) = state.accessibility.as_mut()
        {
            a11y.process_event(&state.window, &event);
        }

        match event {
            WindowEvent::CloseRequested => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::WindowCloseRequested,
                    );
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.raw_modifiers = mods.state();
                self.modifiers = map_modifiers(self.raw_modifiers, self.alt_gr_down);

                if self.app.drag().is_some_and(|d| {
                    d.cross_window_hover && d.kind == fret_app::DragKind::DockPanel
                }) {
                    self.route_internal_drag_hover_from_cursor();
                    self.drain_effects(event_loop);
                }
            }
            WindowEvent::Focused(focused) => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    state.is_focused = focused;
                    if !focused {
                        state.pressed_buttons = fret_core::MouseButtons::default();
                    }
                }
                macos_window_log(format_args!(
                    "[focused] app_window={:?} focused={} winit={:?}",
                    app_window, focused, window_id
                ));
            }
            WindowEvent::Moved(position) => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    let logical = position.to_logical::<f32>(state.window.scale_factor());
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::WindowMoved {
                            x: logical.x.round() as i32,
                            y: logical.y.round() as i32,
                        },
                    );
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if is_alt_gr_key(&event.logical_key) {
                    self.alt_gr_down = event.state == ElementState::Pressed;
                    self.modifiers = map_modifiers(self.raw_modifiers, self.alt_gr_down);
                }
                if let Some(state) = self.windows.get_mut(app_window) {
                    let key = map_physical_key(event.physical_key);
                    let repeat = event.repeat;

                    match event.state {
                        ElementState::Pressed => {
                            // ADR 0072 (proposed): Escape cancels an active cross-window dock drag
                            // session (ImGui/Zed-class behavior). Handle it here so we can also
                            // clear internal drag hover state and stop any tear-off follow
                            // movement immediately.
                            if key == fret_core::KeyCode::Escape
                                && self.app.drag().is_some_and(|d| {
                                    d.cross_window_hover && d.kind == fret_app::DragKind::DockPanel
                                })
                            {
                                self.app.cancel_drag();
                                let _ = self.clear_internal_drag_hover_if_needed();
                                if self.dock_tearoff_follow.is_some() {
                                    self.stop_dock_tearoff_follow(Instant::now(), true);
                                }
                                self.drain_effects(event_loop);
                                return;
                            }

                            let services =
                                Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                            self.driver.handle_event(
                                &mut self.app,
                                services,
                                app_window,
                                &mut state.user,
                                &Event::KeyDown {
                                    key,
                                    modifiers: self.modifiers,
                                    repeat,
                                },
                            );
                            if let Some(text) = event.text
                                && let Some(text) = sanitize_text_input(text.as_str())
                            {
                                let services = Self::ui_services_mut(
                                    &mut self.renderer,
                                    &mut self.no_services,
                                );
                                self.driver.handle_event(
                                    &mut self.app,
                                    services,
                                    app_window,
                                    &mut state.user,
                                    &Event::TextInput(text),
                                );
                            }
                        }
                        ElementState::Released => {
                            let services =
                                Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                            self.driver.handle_event(
                                &mut self.app,
                                services,
                                app_window,
                                &mut state.user,
                                &Event::KeyUp {
                                    key,
                                    modifiers: self.modifiers,
                                },
                            );
                        }
                    }
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::Ime(ime) => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    let mapped = match ime {
                        winit::event::Ime::Enabled => fret_core::ImeEvent::Enabled,
                        winit::event::Ime::Disabled => fret_core::ImeEvent::Disabled,
                        winit::event::Ime::Commit(text) => fret_core::ImeEvent::Commit(text),
                        winit::event::Ime::Preedit(text, cursor) => {
                            fret_core::ImeEvent::Preedit { text, cursor }
                        }
                    };
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::Ime(mapped),
                    );
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::HoveredFile(path) => {
                tracing::debug!(path = %path.display(), "winit hovered file");
                let existing = self
                    .windows
                    .get(app_window)
                    .and_then(|s| s.external_drag_token);
                let token = existing.unwrap_or_else(|| self.allocate_external_drop_token());

                let (position, kind, files) = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        self.drain_effects(event_loop);
                        return;
                    };
                    if state.external_drag_token.is_none() {
                        state.external_drag_token = Some(token);
                    }
                    let position = state.cursor_pos;
                    state.external_drag_files.push(path);
                    let files = state.external_drag_files.clone();
                    let kind = if state.external_drag_files.len() == 1 {
                        ExternalDragKind::EnterFiles(Self::external_drag_files(token, &files))
                    } else {
                        ExternalDragKind::OverFiles(Self::external_drag_files(token, &files))
                    };
                    (position, kind, files)
                };

                self.external_drop_payloads.insert(token, files);

                if let Some(state) = self.windows.get_mut(app_window) {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::ExternalDrag(ExternalDragEvent { position, kind }),
                    );
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::DroppedFile(path) => {
                tracing::debug!(path = %path.display(), "winit dropped file");
                let existing = self
                    .windows
                    .get(app_window)
                    .and_then(|s| s.external_drag_token);
                let token = existing.unwrap_or_else(|| self.allocate_external_drop_token());

                let (position, kind, files) = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        self.drain_effects(event_loop);
                        return;
                    };
                    if state.external_drag_token.is_none() {
                        state.external_drag_token = Some(token);
                    }
                    let position = state.cursor_pos;
                    if state.external_drag_files.is_empty() {
                        state.external_drag_files.push(path);
                    }
                    let files = std::mem::take(&mut state.external_drag_files);
                    state.external_drag_token = None;
                    let kind =
                        ExternalDragKind::DropFiles(Self::external_drag_files(token, &files));
                    (position, kind, files)
                };

                self.external_drop_payloads.insert(token, files);

                if let Some(state) = self.windows.get_mut(app_window) {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::ExternalDrag(ExternalDragEvent { position, kind }),
                    );
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::HoveredFileCancelled => {
                tracing::debug!("winit hovered file cancelled");
                let (position, token) = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        self.drain_effects(event_loop);
                        return;
                    };
                    let position = state.cursor_pos;
                    state.external_drag_files.clear();
                    let token = state.external_drag_token.take();
                    (position, token)
                };

                if let Some(token) = token {
                    self.external_drop_payloads.remove(&token);
                }

                if let Some(state) = self.windows.get_mut(app_window) {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::ExternalDrag(ExternalDragEvent {
                            position,
                            kind: ExternalDragKind::Leave,
                        }),
                    );
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::Resized(size) => {
                self.resize_surface(app_window, size.width, size.height);
                if let Some(state) = self.windows.get_mut(app_window) {
                    let scale = state.window.scale_factor() as f32;
                    let logical: winit::dpi::LogicalSize<f32> = size.to_logical(scale as f64);
                    self.app
                        .with_global_mut(WindowMetricsService::default, |svc, _app| {
                            svc.set_inner_size(
                                app_window,
                                Size::new(Px(logical.width), Px(logical.height)),
                            );
                        });
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::WindowResized {
                            width: Px(logical.width),
                            height: Px(logical.height),
                        },
                    );
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::WindowScaleFactorChanged(scale),
                    );
                }
                self.app.request_redraw(app_window);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (pos, buttons, external_drag_token, screen_pos) = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        return;
                    };
                    let logical = position.to_logical::<f32>(state.window.scale_factor());
                    state.cursor_pos = Point::new(Px(logical.x), Px(logical.y));

                    let screen_pos = state.window.inner_position().ok().map(|inner| {
                        PhysicalPosition::new(
                            inner.x as f64 + position.x,
                            inner.y as f64 + position.y,
                        )
                    });

                    (
                        state.cursor_pos,
                        state.pressed_buttons,
                        state.external_drag_token,
                        screen_pos,
                    )
                };

                if let Some(p) = screen_pos {
                    self.cursor_screen_pos = Some(p);
                }

                let _ = self.update_dock_tearoff_follow();

                if let Some(token) = external_drag_token {
                    let paths = self
                        .external_drop_payloads
                        .get(&token)
                        .cloned()
                        .unwrap_or_default();
                    let kind =
                        ExternalDragKind::OverFiles(Self::external_drag_files(token, &paths));
                    if let Some(state) = self.windows.get_mut(app_window) {
                        let services =
                            Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                        self.driver.handle_event(
                            &mut self.app,
                            services,
                            app_window,
                            &mut state.user,
                            &Event::ExternalDrag(ExternalDragEvent {
                                position: pos,
                                kind,
                            }),
                        );
                    }
                }
                self.dispatch_pointer_event(
                    app_window,
                    fret_core::PointerEvent::Move {
                        position: pos,
                        buttons,
                        modifiers: self.modifiers,
                    },
                );
                self.route_internal_drag_hover_from_cursor();
                self.drain_effects(event_loop);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pos = {
                    let Some(runtime) = self.windows.get_mut(app_window) else {
                        return;
                    };
                    let pos = runtime.cursor_pos;
                    match state {
                        ElementState::Pressed => {
                            set_mouse_buttons(&mut runtime.pressed_buttons, button, true);
                        }
                        ElementState::Released => {
                            set_mouse_buttons(&mut runtime.pressed_buttons, button, false);
                        }
                    }
                    pos
                };

                let Some(button) = map_mouse_button(button) else {
                    return;
                };

                match state {
                    ElementState::Pressed => {
                        if button == fret_core::MouseButton::Left {
                            self.left_mouse_down = true;
                        }
                        self.dispatch_pointer_event(
                            app_window,
                            fret_core::PointerEvent::Down {
                                position: pos,
                                button,
                                modifiers: self.modifiers,
                            },
                        );
                    }
                    ElementState::Released => {
                        if button == fret_core::MouseButton::Left {
                            self.left_mouse_down = false;
                            self.saw_left_mouse_release_this_turn = true;
                            self.route_internal_drag_drop_from_cursor();
                            self.stop_dock_tearoff_follow(Instant::now(), true);
                            // Cross-window drags are runner-routed (Enter/Over/Drop), so ensure the
                            // drag session cannot get "stuck" if no widget ends it.
                            if self.app.drag().is_some_and(|d| d.cross_window_hover) {
                                self.app.cancel_drag();
                                let _ = self.clear_internal_drag_hover_if_needed();
                            }
                        }
                        self.dispatch_pointer_event(
                            app_window,
                            fret_core::PointerEvent::Up {
                                position: pos,
                                button,
                                modifiers: self.modifiers,
                            },
                        );
                    }
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (pos, scroll) = {
                    let Some(runtime) = self.windows.get(app_window) else {
                        return;
                    };
                    let pos = runtime.cursor_pos;
                    let scroll = Self::map_wheel_delta(&self.config, &runtime.window, delta);
                    (pos, scroll)
                };

                self.dispatch_pointer_event(
                    app_window,
                    fret_core::PointerEvent::Wheel {
                        position: pos,
                        delta: scroll,
                        modifiers: self.modifiers,
                    },
                );
                self.drain_effects(event_loop);
            }
            WindowEvent::RedrawRequested => {
                let (Some(context), Some(renderer)) =
                    (self.context.as_ref(), self.renderer.as_mut())
                else {
                    return;
                };
                let Some(state) = self.windows.get_mut(app_window) else {
                    return;
                };

                let (frame, view) = match state.surface.get_current_frame_view() {
                    Ok(v) => v,
                    Err(wgpu::SurfaceError::Lost) => {
                        let size = state.window.inner_size();
                        self.resize_surface(app_window, size.width, size.height);
                        return;
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        event_loop.exit();
                        return;
                    }
                    Err(
                        wgpu::SurfaceError::Outdated
                        | wgpu::SurfaceError::Timeout
                        | wgpu::SurfaceError::Other,
                    ) => return,
                };

                let scale_factor = state.window.scale_factor() as f32;
                let physical = state.window.inner_size();
                let logical: winit::dpi::LogicalSize<f32> =
                    physical.to_logical(state.window.scale_factor());

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(logical.width), Px(logical.height)),
                );

                self.driver.gpu_frame_prepare(
                    &mut self.app,
                    app_window,
                    &mut state.user,
                    context,
                    renderer,
                    scale_factor,
                );

                self.driver.render(
                    &mut self.app,
                    app_window,
                    &mut state.user,
                    bounds,
                    scale_factor,
                    renderer as &mut dyn fret_core::UiServices,
                    &mut state.scene,
                );

                if let Some(a11y) = state.accessibility.as_mut()
                    && a11y.is_active()
                    && let Some(snapshot) = self.driver.accessibility_snapshot(
                        &mut self.app,
                        app_window,
                        &mut state.user,
                    )
                {
                    let update = fret_platform::accessibility::tree_update_from_snapshot(
                        &snapshot,
                        state.window.scale_factor(),
                    );
                    a11y.update_if_active(|| update);
                }

                let engine_frame = self.driver.record_engine_frame(
                    &mut self.app,
                    app_window,
                    &mut state.user,
                    context,
                    renderer,
                    scale_factor,
                    self.tick_id,
                    self.frame_id,
                );

                for update in engine_frame.target_updates {
                    match update {
                        RenderTargetUpdate::Update { id, desc } => {
                            if !renderer.update_render_target(id, desc) {
                                error!(
                                    ?id,
                                    "engine frame update tried to update unknown render target"
                                );
                            }
                        }
                        RenderTargetUpdate::Unregister { id } => {
                            if !renderer.unregister_render_target(id) {
                                error!(
                                    ?id,
                                    "engine frame update tried to unregister unknown render target"
                                );
                            }
                        }
                    }
                }

                let ui_cmd = renderer.render_scene(
                    &context.device,
                    &context.queue,
                    fret_render::RenderSceneParams {
                        format: state.surface.format(),
                        target_view: &view,
                        scene: &state.scene,
                        clear: self.config.clear_color,
                        scale_factor,
                        viewport_size: state.surface.size(),
                    },
                );

                let mut cmd_buffers = engine_frame.command_buffers;
                cmd_buffers.push(ui_cmd);
                context.queue.submit(cmd_buffers);
                frame.present();

                self.frame_id.0 = self.frame_id.0.saturating_add(1);
                self.app.set_frame_id(self.frame_id);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.tick_id.0 = self.tick_id.0.saturating_add(1);
        self.app.set_tick_id(self.tick_id);
        self.saw_left_mouse_release_this_turn = false;

        for (app_window, state) in self.windows.iter_mut() {
            let Some(a11y) = state.accessibility.as_mut() else {
                continue;
            };

            if a11y.take_activation_request() {
                state.window.request_redraw();
            }

            let mut requests = Vec::new();
            a11y.drain_actions(&mut requests);
            a11y.drain_actions_fallback(&mut requests);

            for req in requests {
                if let Some(target) = fret_platform::accessibility::focus_target_from_action(&req) {
                    self.driver.accessibility_focus(
                        &mut self.app,
                        app_window,
                        &mut state.user,
                        target,
                    );
                    self.app.request_redraw(app_window);
                    continue;
                }

                if let Some(target) = fret_platform::accessibility::invoke_target_from_action(&req)
                {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.accessibility_invoke(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        target,
                    );
                    self.app.request_redraw(app_window);
                    continue;
                }

                if let Some((target, data)) =
                    fret_platform::accessibility::set_value_from_action(&req)
                {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    match data {
                        fret_platform::accessibility::SetValueData::Text(value) => {
                            self.driver.accessibility_set_value_text(
                                &mut self.app,
                                services,
                                app_window,
                                &mut state.user,
                                target,
                                &value,
                            );
                        }
                        fret_platform::accessibility::SetValueData::Numeric(value) => {
                            self.driver.accessibility_set_value_numeric(
                                &mut self.app,
                                services,
                                app_window,
                                &mut state.user,
                                target,
                                value,
                            );
                        }
                    }
                    self.app.request_redraw(app_window);
                }
            }
        }

        if let Some(follow) = self.dock_tearoff_follow
            && !self.is_left_mouse_down_for_window(follow.source_window)
        {
            self.stop_dock_tearoff_follow(Instant::now(), false);
        }

        self.drain_effects(event_loop);

        let now = Instant::now();

        #[cfg(target_os = "macos")]
        {
            if self.maybe_finish_dock_drag_released_outside() {
                self.drain_effects(event_loop);
            }
        }

        let did_pending_front_work = self.process_pending_front_requests(now);

        let mut next_deadline: Option<Instant> = None;
        for entry in self.timers.values() {
            next_deadline = Some(match next_deadline {
                Some(cur) => cur.min(entry.deadline),
                None => entry.deadline,
            });
        }

        if let Some(deadline) = self.next_pending_front_deadline() {
            next_deadline = Some(match next_deadline {
                Some(cur) => cur.min(deadline),
                None => deadline,
            });
        }

        let drag_poll = self.app.drag().is_some_and(|d| d.cross_window_hover);
        let follow_poll = self.dock_tearoff_follow.is_some();
        let wants_raf = !self.raf_windows.is_empty() || drag_poll || follow_poll;
        self.raf_windows.clear();

        let next = match (next_deadline, wants_raf) {
            (Some(deadline), true) => Some((now + self.config.frame_interval).min(deadline)),
            (Some(deadline), false) => Some(deadline),
            (None, true) => Some(now + self.config.frame_interval),
            (None, false) => None,
        };

        if drag_poll || follow_poll {
            event_loop.set_control_flow(ControlFlow::Poll);
        } else if let Some(next) = next {
            event_loop.set_control_flow(ControlFlow::WaitUntil(next));
        } else if did_pending_front_work {
            // Ensure we keep turning the event loop while we try to raise a window on macOS.
            event_loop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_millis(16)));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}

fn is_alt_gr_key(key: &Key) -> bool {
    matches!(key, Key::Named(NamedKey::AltGraph))
}

fn map_modifiers(state: ModifiersState, alt_gr_down: bool) -> Modifiers {
    let mut mods = Modifiers {
        shift: state.shift_key(),
        ctrl: state.control_key(),
        alt: state.alt_key(),
        alt_gr: alt_gr_down,
        meta: state.super_key(),
    };

    if mods.alt_gr {
        mods.ctrl = false;
        mods.alt = false;
    }

    mods
}

fn map_mouse_button(button: WinitMouseButton) -> Option<MouseButton> {
    Some(match button {
        WinitMouseButton::Left => MouseButton::Left,
        WinitMouseButton::Right => MouseButton::Right,
        WinitMouseButton::Middle => MouseButton::Middle,
        WinitMouseButton::Back => MouseButton::Back,
        WinitMouseButton::Forward => MouseButton::Forward,
        WinitMouseButton::Other(v) => MouseButton::Other(v),
    })
}

fn set_mouse_buttons(
    buttons: &mut fret_core::MouseButtons,
    button: WinitMouseButton,
    pressed: bool,
) {
    match button {
        WinitMouseButton::Left => buttons.left = pressed,
        WinitMouseButton::Right => buttons.right = pressed,
        WinitMouseButton::Middle => buttons.middle = pressed,
        WinitMouseButton::Back | WinitMouseButton::Forward | WinitMouseButton::Other(_) => {}
    }
}

fn map_physical_key(key: winit::keyboard::PhysicalKey) -> fret_core::KeyCode {
    use winit::keyboard::KeyCode as WinitKeyCode;

    let winit::keyboard::PhysicalKey::Code(code) = key else {
        return fret_core::KeyCode::Unknown;
    };

    match code {
        WinitKeyCode::Escape => fret_core::KeyCode::Escape,
        WinitKeyCode::Enter => fret_core::KeyCode::Enter,
        WinitKeyCode::Tab => fret_core::KeyCode::Tab,
        WinitKeyCode::Backspace => fret_core::KeyCode::Backspace,
        WinitKeyCode::Space => fret_core::KeyCode::Space,

        WinitKeyCode::ArrowUp => fret_core::KeyCode::ArrowUp,
        WinitKeyCode::ArrowDown => fret_core::KeyCode::ArrowDown,
        WinitKeyCode::ArrowLeft => fret_core::KeyCode::ArrowLeft,
        WinitKeyCode::ArrowRight => fret_core::KeyCode::ArrowRight,

        WinitKeyCode::Home => fret_core::KeyCode::Home,
        WinitKeyCode::End => fret_core::KeyCode::End,
        WinitKeyCode::PageUp => fret_core::KeyCode::PageUp,
        WinitKeyCode::PageDown => fret_core::KeyCode::PageDown,
        WinitKeyCode::Insert => fret_core::KeyCode::Insert,
        WinitKeyCode::Delete => fret_core::KeyCode::Delete,

        WinitKeyCode::CapsLock => fret_core::KeyCode::CapsLock,

        WinitKeyCode::ShiftLeft => fret_core::KeyCode::ShiftLeft,
        WinitKeyCode::ShiftRight => fret_core::KeyCode::ShiftRight,
        WinitKeyCode::ControlLeft => fret_core::KeyCode::ControlLeft,
        WinitKeyCode::ControlRight => fret_core::KeyCode::ControlRight,
        WinitKeyCode::AltLeft => fret_core::KeyCode::AltLeft,
        WinitKeyCode::AltRight => fret_core::KeyCode::AltRight,
        WinitKeyCode::SuperLeft => fret_core::KeyCode::SuperLeft,
        WinitKeyCode::SuperRight => fret_core::KeyCode::SuperRight,

        WinitKeyCode::Digit0 => fret_core::KeyCode::Digit0,
        WinitKeyCode::Digit1 => fret_core::KeyCode::Digit1,
        WinitKeyCode::Digit2 => fret_core::KeyCode::Digit2,
        WinitKeyCode::Digit3 => fret_core::KeyCode::Digit3,
        WinitKeyCode::Digit4 => fret_core::KeyCode::Digit4,
        WinitKeyCode::Digit5 => fret_core::KeyCode::Digit5,
        WinitKeyCode::Digit6 => fret_core::KeyCode::Digit6,
        WinitKeyCode::Digit7 => fret_core::KeyCode::Digit7,
        WinitKeyCode::Digit8 => fret_core::KeyCode::Digit8,
        WinitKeyCode::Digit9 => fret_core::KeyCode::Digit9,

        WinitKeyCode::KeyA => fret_core::KeyCode::KeyA,
        WinitKeyCode::KeyB => fret_core::KeyCode::KeyB,
        WinitKeyCode::KeyC => fret_core::KeyCode::KeyC,
        WinitKeyCode::KeyD => fret_core::KeyCode::KeyD,
        WinitKeyCode::KeyE => fret_core::KeyCode::KeyE,
        WinitKeyCode::KeyF => fret_core::KeyCode::KeyF,
        WinitKeyCode::KeyG => fret_core::KeyCode::KeyG,
        WinitKeyCode::KeyH => fret_core::KeyCode::KeyH,
        WinitKeyCode::KeyI => fret_core::KeyCode::KeyI,
        WinitKeyCode::KeyJ => fret_core::KeyCode::KeyJ,
        WinitKeyCode::KeyK => fret_core::KeyCode::KeyK,
        WinitKeyCode::KeyL => fret_core::KeyCode::KeyL,
        WinitKeyCode::KeyM => fret_core::KeyCode::KeyM,
        WinitKeyCode::KeyN => fret_core::KeyCode::KeyN,
        WinitKeyCode::KeyO => fret_core::KeyCode::KeyO,
        WinitKeyCode::KeyP => fret_core::KeyCode::KeyP,
        WinitKeyCode::KeyQ => fret_core::KeyCode::KeyQ,
        WinitKeyCode::KeyR => fret_core::KeyCode::KeyR,
        WinitKeyCode::KeyS => fret_core::KeyCode::KeyS,
        WinitKeyCode::KeyT => fret_core::KeyCode::KeyT,
        WinitKeyCode::KeyU => fret_core::KeyCode::KeyU,
        WinitKeyCode::KeyV => fret_core::KeyCode::KeyV,
        WinitKeyCode::KeyW => fret_core::KeyCode::KeyW,
        WinitKeyCode::KeyX => fret_core::KeyCode::KeyX,
        WinitKeyCode::KeyY => fret_core::KeyCode::KeyY,
        WinitKeyCode::KeyZ => fret_core::KeyCode::KeyZ,

        WinitKeyCode::Minus => fret_core::KeyCode::Minus,
        WinitKeyCode::Equal => fret_core::KeyCode::Equal,
        WinitKeyCode::BracketLeft => fret_core::KeyCode::BracketLeft,
        WinitKeyCode::BracketRight => fret_core::KeyCode::BracketRight,
        WinitKeyCode::Backslash => fret_core::KeyCode::Backslash,
        WinitKeyCode::Semicolon => fret_core::KeyCode::Semicolon,
        WinitKeyCode::Quote => fret_core::KeyCode::Quote,
        WinitKeyCode::Backquote => fret_core::KeyCode::Backquote,
        WinitKeyCode::Comma => fret_core::KeyCode::Comma,
        WinitKeyCode::Period => fret_core::KeyCode::Period,
        WinitKeyCode::Slash => fret_core::KeyCode::Slash,

        WinitKeyCode::F1 => fret_core::KeyCode::F1,
        WinitKeyCode::F2 => fret_core::KeyCode::F2,
        WinitKeyCode::F3 => fret_core::KeyCode::F3,
        WinitKeyCode::F4 => fret_core::KeyCode::F4,
        WinitKeyCode::F5 => fret_core::KeyCode::F5,
        WinitKeyCode::F6 => fret_core::KeyCode::F6,
        WinitKeyCode::F7 => fret_core::KeyCode::F7,
        WinitKeyCode::F8 => fret_core::KeyCode::F8,
        WinitKeyCode::F9 => fret_core::KeyCode::F9,
        WinitKeyCode::F10 => fret_core::KeyCode::F10,
        WinitKeyCode::F11 => fret_core::KeyCode::F11,
        WinitKeyCode::F12 => fret_core::KeyCode::F12,

        WinitKeyCode::Numpad0 => fret_core::KeyCode::Numpad0,
        WinitKeyCode::Numpad1 => fret_core::KeyCode::Numpad1,
        WinitKeyCode::Numpad2 => fret_core::KeyCode::Numpad2,
        WinitKeyCode::Numpad3 => fret_core::KeyCode::Numpad3,
        WinitKeyCode::Numpad4 => fret_core::KeyCode::Numpad4,
        WinitKeyCode::Numpad5 => fret_core::KeyCode::Numpad5,
        WinitKeyCode::Numpad6 => fret_core::KeyCode::Numpad6,
        WinitKeyCode::Numpad7 => fret_core::KeyCode::Numpad7,
        WinitKeyCode::Numpad8 => fret_core::KeyCode::Numpad8,
        WinitKeyCode::Numpad9 => fret_core::KeyCode::Numpad9,
        WinitKeyCode::NumpadAdd => fret_core::KeyCode::NumpadAdd,
        WinitKeyCode::NumpadSubtract => fret_core::KeyCode::NumpadSubtract,
        WinitKeyCode::NumpadMultiply => fret_core::KeyCode::NumpadMultiply,
        WinitKeyCode::NumpadDivide => fret_core::KeyCode::NumpadDivide,
        WinitKeyCode::NumpadDecimal => fret_core::KeyCode::NumpadDecimal,
        WinitKeyCode::NumpadEnter => fret_core::KeyCode::NumpadEnter,

        _ => fret_core::KeyCode::Unknown,
    }
}

fn sanitize_text_input(text: &str) -> Option<String> {
    // Contract: `Event::TextInput` represents committed insertion text and must not include
    // control characters. Keys like Backspace/Enter/Tab must be handled via `KeyDown` + commands.
    //
    // Some platform stacks report control keys in `KeyboardInput.text` (e.g. backspace on macOS).
    let filtered: String = text.chars().filter(|ch| !ch.is_control()).collect();
    if filtered.is_empty() {
        None
    } else {
        Some(filtered)
    }
}
