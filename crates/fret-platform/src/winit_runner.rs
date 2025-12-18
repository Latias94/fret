use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use fret_app::{App, CreateWindowRequest, Effect, WindowAnchor, WindowRequest};
use fret_core::{Event, Modifiers, MouseButton, Point, Px, Rect, Scene, Size, ViewportInputEvent};
use fret_render::{ClearColor, Renderer, SurfaceState, WgpuContext};
use slotmap::SlotMap;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, Position},
    event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::ModifiersState,
    window::{Window, WindowId},
};

pub struct WinitRunnerConfig {
    pub main_window_title: String,
    pub main_window_size: LogicalSize<f64>,
    pub default_window_title: String,
    pub default_window_size: LogicalSize<f64>,
    /// Physical pixel offset applied when positioning a new window from an anchor point.
    pub new_window_anchor_offset: (f64, f64),
    /// When the main window requests close, exit the event loop.
    pub exit_on_main_window_close: bool,
    /// Line-based wheel delta unit to logical pixels.
    pub wheel_line_delta_px: f32,
    /// Pixel-based wheel delta scale in logical pixels.
    pub wheel_pixel_delta_scale: f32,
    pub frame_interval: Duration,
    pub activity_timeout: Duration,
    pub clear_color: ClearColor,
    pub wgpu_init: WgpuInit,
}

pub enum WgpuInit {
    /// Create a `WgpuContext` internally using a surface-compatible adapter.
    CreateDefault,
    /// Use a host-provided GPU context. The platform layer will create surfaces via
    /// `context.instance` and assumes the adapter/device are compatible with those surfaces.
    Provided(WgpuContext),
    /// Create the GPU context via a host callback given the main window.
    ///
    /// The callback may choose an adapter that is compatible with the window surface.
    Factory(
        Box<
            dyn FnOnce(Arc<Window>) -> anyhow::Result<(WgpuContext, wgpu::Surface<'static>)>
                + 'static,
        >,
    ),
}

impl Default for WinitRunnerConfig {
    fn default() -> Self {
        Self {
            main_window_title: "fret".to_string(),
            main_window_size: LogicalSize::new(1280.0, 720.0),
            default_window_title: "fret".to_string(),
            default_window_size: LogicalSize::new(640.0, 480.0),
            new_window_anchor_offset: (-40.0, -20.0),
            exit_on_main_window_close: true,
            wheel_line_delta_px: 20.0,
            wheel_pixel_delta_scale: 1.0,
            frame_interval: Duration::from_millis(8),
            activity_timeout: Duration::from_secs(1),
            clear_color: ClearColor::default(),
            wgpu_init: WgpuInit::CreateDefault,
        }
    }
}

impl WinitRunnerConfig {
    fn main_window_spec(&self) -> WindowCreateSpec {
        WindowCreateSpec::new(self.main_window_title.clone(), self.main_window_size)
    }

    fn default_window_spec(&self) -> WindowCreateSpec {
        WindowCreateSpec::new(self.default_window_title.clone(), self.default_window_size)
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

    fn create_window_state(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
    ) -> Self::WindowState;

    fn handle_event(
        &mut self,
        app: &mut App,
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
        scene: &mut Scene,
    );

    fn window_create_spec(
        &mut self,
        app: &mut App,
        request: CreateWindowRequest,
    ) -> Option<WindowCreateSpec>;

    fn window_created(
        &mut self,
        app: &mut App,
        request: CreateWindowRequest,
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
    user: S,
}

pub struct WinitRunner<D: WinitDriver> {
    pub config: WinitRunnerConfig,
    pub app: App,
    pub driver: D,

    context: Option<WgpuContext>,
    renderer: Option<Renderer>,

    windows: SlotMap<fret_core::AppWindowId, WindowRuntime<D::WindowState>>,
    winit_to_app: HashMap<WindowId, fret_core::AppWindowId>,
    main_window: Option<fret_core::AppWindowId>,

    modifiers: Modifiers,
    keep_alive_deadlines: HashMap<fret_core::AppWindowId, Instant>,
}

impl<D: WinitDriver> WinitRunner<D> {
    pub fn new(config: WinitRunnerConfig, app: App, driver: D) -> Self {
        Self {
            config,
            app,
            driver,
            context: None,
            renderer: None,
            windows: SlotMap::with_key(),
            winit_to_app: HashMap::new(),
            main_window: None,
            modifiers: Modifiers::default(),
            keep_alive_deadlines: HashMap::new(),
        }
    }

    fn mark_activity(&mut self, window: fret_core::AppWindowId) {
        self.keep_alive_deadlines
            .insert(window, Instant::now() + self.config.activity_timeout);
        self.app.request_redraw(window);
    }

    fn create_os_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        spec: WindowCreateSpec,
    ) -> anyhow::Result<Arc<Window>> {
        let mut attrs = Window::default_attributes()
            .with_title(spec.title)
            .with_inner_size(spec.size);
        if let Some(position) = spec.position {
            attrs = attrs.with_position(position);
        }
        Ok(Arc::new(event_loop.create_window(attrs)?))
    }

    fn insert_window(
        &mut self,
        window: Arc<Window>,
        surface: wgpu::Surface<'static>,
    ) -> anyhow::Result<fret_core::AppWindowId> {
        let Some(context) = self.context.as_ref() else {
            anyhow::bail!("wgpu context not initialized");
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
        self.keep_alive_deadlines.remove(&window);

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
        request: CreateWindowRequest,
    ) -> anyhow::Result<fret_core::AppWindowId> {
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
                anyhow::bail!("wgpu context not initialized");
            };
            context.create_surface(window.clone())?
        };
        self.insert_window(window, surface)
    }

    fn drain_effects(&mut self, event_loop: &ActiveEventLoop) {
        const MAX_EFFECT_DRAIN_TURNS: usize = 16;

        for _ in 0..MAX_EFFECT_DRAIN_TURNS {
            let effects = self.app.flush_effects();
            if effects.is_empty() {
                break;
            }

            for effect in effects {
                match effect {
                    Effect::Redraw(window) => {
                        if let Some(state) = self.windows.get(window) {
                            state.window.request_redraw();
                            self.mark_activity(window);
                        }
                    }
                    Effect::Command(_) => {}
                    Effect::ViewportInput(event) => {
                        self.driver.viewport_input(&mut self.app, event);
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
                                match self.create_window_from_request(event_loop, create) {
                                    Ok(id) => id,
                                    Err(_) => continue,
                                };
                            self.driver
                                .window_created(&mut self.app, create, new_window);
                            self.mark_activity(new_window);
                        }
                    },
                }
            }
        }
    }

    fn dispatch_pointer_event(
        &mut self,
        window: fret_core::AppWindowId,
        pe: fret_core::PointerEvent,
    ) {
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        self.driver
            .handle_event(&mut self.app, window, &mut state.user, &Event::Pointer(pe));
        self.mark_activity(window);
    }

    fn map_wheel_delta(&self, window: &Window, delta: MouseScrollDelta) -> Point {
        match delta {
            MouseScrollDelta::LineDelta(x, y) => Point::new(
                Px(x * self.config.wheel_line_delta_px),
                Px(y * self.config.wheel_line_delta_px),
            ),
            MouseScrollDelta::PixelDelta(p) => {
                let logical = p.to_logical::<f32>(window.scale_factor());
                Point::new(
                    Px(logical.x * self.config.wheel_pixel_delta_scale),
                    Px(logical.y * self.config.wheel_pixel_delta_scale),
                )
            }
        }
    }
}

impl<D: WinitDriver> ApplicationHandler for WinitRunner<D> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.context.is_some() {
            return;
        }

        let spec = self.config.main_window_spec();
        let window = match self.create_os_window(event_loop, spec) {
            Ok(w) => w,
            Err(_) => return,
        };

        let (context, surface) =
            match std::mem::replace(&mut self.config.wgpu_init, WgpuInit::CreateDefault) {
                WgpuInit::CreateDefault => {
                    match pollster::block_on(WgpuContext::new_with_surface(window.clone())) {
                        Ok(v) => v,
                        Err(_) => return,
                    }
                }
                WgpuInit::Provided(context) => {
                    let surface = match context.create_surface(window.clone()) {
                        Ok(v) => v,
                        Err(_) => return,
                    };
                    (context, surface)
                }
                WgpuInit::Factory(factory) => match factory(window.clone()) {
                    Ok(v) => v,
                    Err(_) => return,
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
            Err(_) => return,
        };
        self.main_window = Some(main_window);
        self.driver.init(&mut self.app, main_window);
        self.mark_activity(main_window);
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
                self.modifiers = map_modifiers(mods.state());
            }
            WindowEvent::Focused(false) => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    state.pressed_buttons = fret_core::MouseButtons::default();
                }
            }
            WindowEvent::Resized(size) => {
                self.resize_surface(app_window, size.width, size.height);
                if let Some(state) = self.windows.get_mut(app_window) {
                    let scale = state.window.scale_factor() as f32;
                    let logical: winit::dpi::LogicalSize<f32> = size.to_logical(scale as f64);
                    self.driver.handle_event(
                        &mut self.app,
                        app_window,
                        &mut state.user,
                        &Event::WindowResized {
                            width: Px(logical.width),
                            height: Px(logical.height),
                        },
                    );
                    self.driver.handle_event(
                        &mut self.app,
                        app_window,
                        &mut state.user,
                        &Event::WindowScaleFactorChanged(scale),
                    );
                }
                self.mark_activity(app_window);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (pos, buttons) = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        return;
                    };
                    let logical = position.to_logical::<f32>(state.window.scale_factor());
                    state.cursor_pos = Point::new(Px(logical.x), Px(logical.y));
                    (state.cursor_pos, state.pressed_buttons)
                };
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
                let Some(button) = map_mouse_button(button) else {
                    return;
                };
                let pos = {
                    let Some(runtime) = self.windows.get_mut(app_window) else {
                        return;
                    };
                    let pressed = matches!(state, ElementState::Pressed);
                    match button {
                        fret_core::MouseButton::Left => runtime.pressed_buttons.left = pressed,
                        fret_core::MouseButton::Right => runtime.pressed_buttons.right = pressed,
                        fret_core::MouseButton::Middle => runtime.pressed_buttons.middle = pressed,
                        fret_core::MouseButton::Back
                        | fret_core::MouseButton::Forward
                        | fret_core::MouseButton::Other(_) => {}
                    }
                    runtime.cursor_pos
                };
                let pe = match state {
                    ElementState::Pressed => fret_core::PointerEvent::Down {
                        position: pos,
                        button,
                        modifiers: self.modifiers,
                    },
                    ElementState::Released => fret_core::PointerEvent::Up {
                        position: pos,
                        button,
                        modifiers: self.modifiers,
                    },
                };
                self.dispatch_pointer_event(app_window, pe);
                self.drain_effects(event_loop);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let Some(state) = self.windows.get(app_window) else {
                    return;
                };
                let delta = self.map_wheel_delta(&state.window, delta);
                let pos = state.cursor_pos;
                self.dispatch_pointer_event(
                    app_window,
                    fret_core::PointerEvent::Wheel {
                        position: pos,
                        delta,
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
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.drain_effects(event_loop);

        let now = Instant::now();
        let mut should_keep_alive = false;
        for (window, deadline) in &self.keep_alive_deadlines {
            if *deadline > now {
                should_keep_alive = true;
                if let Some(state) = self.windows.get(*window) {
                    state.window.request_redraw();
                }
            }
        }

        if should_keep_alive {
            event_loop.set_control_flow(ControlFlow::WaitUntil(now + self.config.frame_interval));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}

fn map_modifiers(state: ModifiersState) -> Modifiers {
    Modifiers {
        shift: state.shift_key(),
        ctrl: state.control_key(),
        alt: state.alt_key(),
        meta: state.super_key(),
    }
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
