use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use fret_app::{App, CreateWindowRequest, Effect, WindowRequest};
use fret_core::{
    Event, ExternalDragEvent, ExternalDragKind, Modifiers, MouseButton, Point, Px, Rect, Scene,
    Size, TextService, ViewportInputEvent,
};
use fret_render::{ClearColor, Renderer, SurfaceState, WgpuContext};
use slotmap::SlotMap;
use tracing::error;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, Position},
    event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Window, WindowId},
};

use crate::error::RunnerError;

type WindowAnchor = fret_core::WindowAnchor;

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
    pub wgpu_init: WgpuInit,
}

pub enum WgpuInit {
    /// Create a `WgpuContext` internally using a surface-compatible adapter.
    CreateDefault,
    /// Use a host-provided GPU context. The runner will create surfaces via `context.instance`
    /// and assumes the adapter/device are compatible with those surfaces.
    Provided(WgpuContext),
    /// Create the GPU context via a host callback given the main window.
    Factory(
        Box<
            dyn FnOnce(Arc<Window>) -> Result<(WgpuContext, wgpu::Surface<'static>), RunnerError>
                + 'static,
        >,
    ),
}

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
}

impl WindowCreateSpec {
    pub fn new(title: impl Into<String>, size: LogicalSize<f64>) -> Self {
        Self {
            title: title.into(),
            size,
            position: None,
        }
    }

    pub fn with_position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }
}

pub trait WinitDriver {
    type WindowState;

    fn init(&mut self, _app: &mut App, _main_window: fret_core::AppWindowId) {}

    fn gpu_ready(&mut self, _app: &mut App, _context: &WgpuContext, _renderer: &mut Renderer) {}

    fn viewport_input(&mut self, _app: &mut App, _event: ViewportInputEvent) {}

    fn dock_op(&mut self, _app: &mut App, _op: fret_core::DockOp) {}

    fn handle_command(
        &mut self,
        _app: &mut App,
        _window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
        _command: fret_app::CommandId,
    ) {
    }

    fn handle_global_command(&mut self, _app: &mut App, _command: fret_app::CommandId) {}

    fn create_window_state(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
    ) -> Self::WindowState;

    fn handle_event(
        &mut self,
        app: &mut App,
        text: &mut dyn TextService,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        event: &Event,
    );

    fn render(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        bounds: Rect,
        scale_factor: f32,
        text: &mut dyn fret_core::TextService,
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
}

struct WindowRuntime<S> {
    window: Arc<Window>,
    surface: SurfaceState<'static>,
    scene: Scene,
    cursor_pos: Point,
    pressed_buttons: fret_core::MouseButtons,
    ime_allowed: bool,
    external_drag_files: Vec<std::path::PathBuf>,
    user: S,
}

pub struct WinitRunner<D: WinitDriver> {
    pub config: WinitRunnerConfig,
    pub app: App,
    pub driver: D,

    context: Option<WgpuContext>,
    renderer: Option<Renderer>,
    no_text: NoTextService,

    windows: SlotMap<fret_core::AppWindowId, WindowRuntime<D::WindowState>>,
    winit_to_app: HashMap<WindowId, fret_core::AppWindowId>,
    main_window: Option<fret_core::AppWindowId>,

    modifiers: Modifiers,
    raw_modifiers: ModifiersState,
    alt_gr_down: bool,

    tick_id: fret_core::TickId,
    frame_id: fret_core::FrameId,

    raf_windows: HashSet<fret_core::AppWindowId>,
    timers: HashMap<fret_core::TimerToken, TimerEntry>,
    clipboard: Option<arboard::Clipboard>,
}

#[derive(Debug, Clone)]
struct TimerEntry {
    window: Option<fret_core::AppWindowId>,
    deadline: Instant,
    repeat: Option<Duration>,
}

impl<D: WinitDriver> WinitRunner<D> {
    pub fn new(config: WinitRunnerConfig, app: App, driver: D) -> Self {
        let raw_modifiers = ModifiersState::empty();
        let alt_gr_down = false;
        Self {
            config,
            app,
            driver,
            context: None,
            renderer: None,
            no_text: NoTextService,
            windows: SlotMap::with_key(),
            winit_to_app: HashMap::new(),
            main_window: None,
            modifiers: map_modifiers(raw_modifiers, alt_gr_down),
            raw_modifiers,
            alt_gr_down,
            tick_id: fret_core::TickId::default(),
            frame_id: fret_core::FrameId::default(),
            raf_windows: HashSet::new(),
            timers: HashMap::new(),
            clipboard: arboard::Clipboard::new().ok(),
        }
    }

    fn text_service_mut_ptr(&mut self) -> *mut dyn TextService {
        match self.renderer.as_mut() {
            Some(renderer) => renderer as &mut dyn TextService as *mut dyn TextService,
            None => &mut self.no_text as &mut dyn TextService as *mut dyn TextService,
        }
    }

    fn create_os_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        spec: WindowCreateSpec,
    ) -> Result<Arc<Window>, RunnerError> {
        let mut attrs = Window::default_attributes()
            .with_title(spec.title)
            .with_inner_size(spec.size);
        if let Some(position) = spec.position {
            attrs = attrs.with_position(position);
        }
        Ok(Arc::new(event_loop.create_window(attrs).map_err(
            |source| RunnerError::CreateWindowFailed { source },
        )?))
    }

    fn insert_window(
        &mut self,
        window: Arc<Window>,
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
                surface,
                scene: Scene::default(),
                cursor_pos: Point::new(Px(0.0), Px(0.0)),
                pressed_buttons: fret_core::MouseButtons::default(),
                ime_allowed: false,
                external_drag_files: Vec::new(),
                user,
            }
        });

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
        if Some(window) == self.main_window {
            self.main_window = None;
        }
    }

    fn compute_window_position_from_anchor(&self, anchor: WindowAnchor) -> Option<Position> {
        let anchor_state = self.windows.get(anchor.window)?;
        let outer = anchor_state.window.outer_position().ok()?;
        let scale = anchor_state.window.scale_factor();

        let (ox, oy) = self.config.new_window_anchor_offset;
        let x = outer.x as f64 + anchor.position.x.0 as f64 * scale + ox;
        let y = outer.y as f64 + anchor.position.y.0 as f64 * scale + oy;
        Some(PhysicalPosition::new(x as i32, y as i32).into())
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
            if let Some(anchor) = request.anchor {
                spec.position = self.compute_window_position_from_anchor(anchor);
            }
        }

        let window = self.create_os_window(event_loop, spec)?;
        let surface = {
            let Some(context) = self.context.as_ref() else {
                return Err(RunnerError::WgpuNotInitialized);
            };
            context.create_surface(window.clone())?
        };
        self.insert_window(window, surface)
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
                let text_ptr = self.text_service_mut_ptr();
                if let Some(state) = self.windows.get_mut(window) {
                    self.driver.handle_event(
                        &mut self.app,
                        unsafe { &mut *text_ptr },
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
                                self.driver.handle_command(
                                    &mut self.app,
                                    window,
                                    &mut state.user,
                                    command,
                                );
                            }
                        }
                        None => {
                            self.driver.handle_global_command(&mut self.app, command);
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
                        let text_ptr = self.text_service_mut_ptr();
                        if let Some(state) = self.windows.get_mut(window) {
                            self.driver.handle_event(
                                &mut self.app,
                                unsafe { &mut *text_ptr },
                                window,
                                &mut state.user,
                                &Event::ClipboardText(text),
                            );
                        }
                    }
                    Effect::ViewportInput(event) => {
                        self.driver.viewport_input(&mut self.app, event);
                    }
                    Effect::Dock(op) => {
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
                            let new_window =
                                match self.create_window_from_request(event_loop, &create) {
                                    Ok(id) => id,
                                    Err(e) => {
                                        error!(error = ?e, "failed to create window from request");
                                        continue;
                                    }
                                };
                            self.driver
                                .window_created(&mut self.app, &create, new_window);
                            self.app.request_redraw(new_window);
                        }
                    },
                }
            }

            did_work |= self.fire_due_timers(now);

            if !did_work {
                break;
            }
        }
    }

    fn dispatch_pointer_event(
        &mut self,
        window: fret_core::AppWindowId,
        pe: fret_core::PointerEvent,
    ) {
        let text_ptr = self.text_service_mut_ptr();
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        self.driver.handle_event(
            &mut self.app,
            unsafe { &mut *text_ptr },
            window,
            &mut state.user,
            &Event::Pointer(pe),
        );
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
}

struct NoTextService;

impl TextService for NoTextService {
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

impl<D: WinitDriver> ApplicationHandler for WinitRunner<D> {
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
                    match pollster::block_on(WgpuContext::new_with_surface(window.clone())) {
                        Ok(v) => v,
                        Err(e) => {
                            error!(error = ?e, "failed to initialize wgpu context");
                            return;
                        }
                    }
                }
                WgpuInit::Provided(context) => {
                    let surface = match context.create_surface(window.clone()) {
                        Ok(v) => v,
                        Err(e) => {
                            error!(error = ?e, "failed to create surface from provided context");
                            return;
                        }
                    };
                    (context, surface)
                }
                WgpuInit::Factory(factory) => match factory(window.clone()) {
                    Ok(v) => v,
                    Err(e) => {
                        error!(error = ?e, "wgpu factory failed");
                        return;
                    }
                },
            };
        let renderer = Renderer::new(&context.device);

        self.context = Some(context);
        self.renderer = Some(renderer);
        if let (Some(context), Some(renderer)) = (self.context.as_ref(), self.renderer.as_mut()) {
            self.driver.gpu_ready(&mut self.app, context, renderer);
        }

        let main_window = match self.insert_window(window, surface) {
            Ok(id) => id,
            Err(e) => {
                error!(error = ?e, "failed to insert main window runtime");
                return;
            }
        };
        self.main_window = Some(main_window);
        self.driver.init(&mut self.app, main_window);
        self.app.request_redraw(main_window);
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

        let text_ptr = self.text_service_mut_ptr();

        match event {
            WindowEvent::CloseRequested => {
                let is_main = Some(app_window) == self.main_window;
                if is_main && self.config.exit_on_main_window_close {
                    event_loop.exit();
                    return;
                }
                self.close_window(app_window);
                if is_main && self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.raw_modifiers = mods.state();
                self.modifiers = map_modifiers(self.raw_modifiers, self.alt_gr_down);
            }
            WindowEvent::Focused(false) => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    state.pressed_buttons = fret_core::MouseButtons::default();
                }
            }
            WindowEvent::Moved(position) => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    let logical = position.to_logical::<f32>(state.window.scale_factor());
                    self.driver.handle_event(
                        &mut self.app,
                        unsafe { &mut *text_ptr },
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
                            self.driver.handle_event(
                                &mut self.app,
                                unsafe { &mut *text_ptr },
                                app_window,
                                &mut state.user,
                                &Event::KeyDown {
                                    key,
                                    modifiers: self.modifiers,
                                    repeat,
                                },
                            );
                            if let Some(text) = event.text {
                                if let Some(text) = sanitize_text_input(text.as_str()) {
                                    self.driver.handle_event(
                                        &mut self.app,
                                        unsafe { &mut *text_ptr },
                                        app_window,
                                        &mut state.user,
                                        &Event::TextInput(text),
                                    );
                                }
                            }
                        }
                        ElementState::Released => {
                            self.driver.handle_event(
                                &mut self.app,
                                unsafe { &mut *text_ptr },
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
                    self.driver.handle_event(
                        &mut self.app,
                        unsafe { &mut *text_ptr },
                        app_window,
                        &mut state.user,
                        &Event::Ime(mapped),
                    );
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::HoveredFile(path) => {
                tracing::debug!(path = %path.display(), "winit hovered file");
                if let Some(state) = self.windows.get_mut(app_window) {
                    let position = state.cursor_pos;
                    state.external_drag_files.push(path);
                    let files = state.external_drag_files.clone();
                    self.driver.handle_event(
                        &mut self.app,
                        unsafe { &mut *text_ptr },
                        app_window,
                        &mut state.user,
                        &Event::ExternalDrag(ExternalDragEvent {
                            position,
                            kind: if state.external_drag_files.len() == 1 {
                                ExternalDragKind::EnterFiles(files)
                            } else {
                                ExternalDragKind::OverFiles(files)
                            },
                        }),
                    );
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::DroppedFile(path) => {
                tracing::debug!(path = %path.display(), "winit dropped file");
                if let Some(state) = self.windows.get_mut(app_window) {
                    let position = state.cursor_pos;
                    if state.external_drag_files.is_empty() {
                        state.external_drag_files.push(path);
                    }
                    let files = std::mem::take(&mut state.external_drag_files);
                    self.driver.handle_event(
                        &mut self.app,
                        unsafe { &mut *text_ptr },
                        app_window,
                        &mut state.user,
                        &Event::ExternalDrag(ExternalDragEvent {
                            position,
                            kind: ExternalDragKind::DropFiles(files),
                        }),
                    );
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::HoveredFileCancelled => {
                tracing::debug!("winit hovered file cancelled");
                if let Some(state) = self.windows.get_mut(app_window) {
                    let position = state.cursor_pos;
                    state.external_drag_files.clear();
                    self.driver.handle_event(
                        &mut self.app,
                        unsafe { &mut *text_ptr },
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
                    self.driver.handle_event(
                        &mut self.app,
                        unsafe { &mut *text_ptr },
                        app_window,
                        &mut state.user,
                        &Event::WindowResized {
                            width: Px(logical.width),
                            height: Px(logical.height),
                        },
                    );
                    self.driver.handle_event(
                        &mut self.app,
                        unsafe { &mut *text_ptr },
                        app_window,
                        &mut state.user,
                        &Event::WindowScaleFactorChanged(scale),
                    );
                }
                self.app.request_redraw(app_window);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (pos, buttons, external_drag_files) = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        return;
                    };
                    let logical = position.to_logical::<f32>(state.window.scale_factor());
                    state.cursor_pos = Point::new(Px(logical.x), Px(logical.y));
                    (
                        state.cursor_pos,
                        state.pressed_buttons,
                        state.external_drag_files.clone(),
                    )
                };

                if !external_drag_files.is_empty() {
                    if let Some(state) = self.windows.get_mut(app_window) {
                        self.driver.handle_event(
                            &mut self.app,
                            unsafe { &mut *text_ptr },
                            app_window,
                            &mut state.user,
                            &Event::ExternalDrag(ExternalDragEvent {
                                position: pos,
                                kind: ExternalDragKind::OverFiles(external_drag_files),
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

                state.scene.clear();

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(logical.width), Px(logical.height)),
                );

                self.driver.render(
                    &mut self.app,
                    app_window,
                    &mut state.user,
                    bounds,
                    scale_factor,
                    renderer as &mut dyn fret_core::TextService,
                    &mut state.scene,
                );

                let cmd = renderer.render_scene(
                    &context.device,
                    &context.queue,
                    state.surface.format(),
                    &view,
                    &state.scene,
                    self.config.clear_color,
                    scale_factor,
                    state.surface.size(),
                );

                context.queue.submit([cmd]);
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

        self.drain_effects(event_loop);

        let now = Instant::now();

        let mut next_deadline: Option<Instant> = None;
        for entry in self.timers.values() {
            next_deadline = Some(match next_deadline {
                Some(cur) => cur.min(entry.deadline),
                None => entry.deadline,
            });
        }

        let wants_raf = !self.raf_windows.is_empty();
        self.raf_windows.clear();

        let next = match (next_deadline, wants_raf) {
            (Some(deadline), true) => Some((now + self.config.frame_interval).min(deadline)),
            (Some(deadline), false) => Some(deadline),
            (None, true) => Some(now + self.config.frame_interval),
            (None, false) => None,
        };

        if let Some(next) = next {
            event_loop.set_control_flow(ControlFlow::WaitUntil(next));
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
