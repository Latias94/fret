use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret_components_shadcn as shadcn;
use fret_core::{AppWindowId, Event as FretEvent, Point, Px, Rect, Scene, Size, UiServices};
use fret_render::{ClearColor, RenderSceneParams, Renderer, SurfaceState, WgpuContext};
use fret_runtime::{Effect, FrameId, Model, PlatformCapabilities, TickId};
use fret_ui_app::declarative;
use fret_ui_app::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, PressableProps,
};
use fret_ui_app::{Invalidation, Theme, UiFrameCx, UiTree};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use winit::application::ApplicationHandler;
use winit::cursor::Cursor;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowAttributes, WindowId};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::{WindowAttributesWeb, WindowExtWeb};

#[cfg(target_arch = "wasm32")]
use fret_platform_web::WebPlatformServices;

struct GfxState {
    ctx: WgpuContext,
    surface_state: SurfaceState<'static>,
    renderer: Renderer,
    clear: ClearColor,
    scene: Scene,
    scale_factor: f32,
    last_surface_error: Option<wgpu::SurfaceError>,
}

impl GfxState {
    fn services_and_scene_mut(&mut self) -> (&mut dyn UiServices, &mut Scene) {
        (&mut self.renderer, &mut self.scene)
    }

    fn ui_bounds(&self, logical: LogicalSize<f32>) -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(logical.width), Px(logical.height)),
        )
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_state
            .resize(&self.ctx.device, size.width.max(1), size.height.max(1));
    }

    fn render(&mut self) -> Result<(), fret_render::RenderError> {
        let (frame, view) = match self.surface_state.get_current_frame_view() {
            Ok(v) => v,
            Err(err) => {
                if self.last_surface_error.as_ref() != Some(&err) {
                    self.last_surface_error = Some(err.clone());
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::error_1(&JsValue::from_str(&format!(
                        "surface.get_current_texture failed: {:?}",
                        self.last_surface_error
                    )));
                }

                match err {
                    wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                        let (w, h) = self.surface_state.size();
                        self.surface_state.resize(&self.ctx.device, w, h);
                    }
                    wgpu::SurfaceError::Timeout => {}
                    wgpu::SurfaceError::OutOfMemory => panic!("wgpu surface out of memory"),
                    wgpu::SurfaceError::Other => {}
                }

                return Ok(());
            }
        };
        let cmd = self.renderer.render_scene(
            &self.ctx.device,
            &self.ctx.queue,
            RenderSceneParams {
                format: self.surface_state.format(),
                target_view: &view,
                scene: &self.scene,
                clear: self.clear,
                scale_factor: self.scale_factor,
                viewport_size: self.surface_state.size(),
            },
        );
        self.ctx.queue.submit([cmd]);
        frame.present();
        Ok(())
    }
}

struct WebDemoApp {
    canvas_id: String,
    window: Option<Rc<dyn Window>>,
    window_id: Option<WindowId>,
    gfx: Rc<RefCell<Option<GfxState>>>,
    pending_events: Vec<FretEvent>,
    tick_id: TickId,
    frame_id: FrameId,

    app: fret_ui_app::App,
    ui: UiTree,
    fret_window: AppWindowId,
    counter: Model<u32>,
    last_input: Model<Arc<str>>,
    shadcn_checked: Model<bool>,
    platform: fret_runner_winit::WinitPlatform,

    #[cfg(target_arch = "wasm32")]
    web_cursor: Option<fret_runner_winit::WebCursorListener>,

    #[cfg(target_arch = "wasm32")]
    web_services: WebPlatformServices,

    #[cfg(target_arch = "wasm32")]
    event_loop_proxy: EventLoopProxy,
}

impl WebDemoApp {
    fn new(canvas_id: String, event_loop_proxy: EventLoopProxy) -> Self {
        let mut app = fret_ui_app::App::new();
        let mut caps = PlatformCapabilities::default();
        caps.ui.multi_window = false;
        caps.ui.window_tear_off = false;
        caps.gfx.webgpu = true;
        caps.gfx.native_gpu = false;
        caps.ime.enabled = false;
        caps.ime.set_cursor_area = false;
        caps.fs.real_paths = false;
        app.set_global(caps);

        let fret_window = AppWindowId::default();
        let mut ui = UiTree::new();
        ui.set_window(fret_window);

        let counter = app.models_mut().insert(0u32);
        let last_input = app.models_mut().insert(Arc::<str>::from("input: <none>"));
        let shadcn_checked = app.models_mut().insert(false);

        let mut web_services = WebPlatformServices::default();
        let proxy = event_loop_proxy.clone();
        web_services.set_waker(move || proxy.wake_up());

        Self {
            canvas_id,
            window: None,
            window_id: None,
            gfx: Rc::new(RefCell::new(None)),
            pending_events: Vec::new(),
            tick_id: TickId::default(),
            frame_id: FrameId::default(),
            app,
            ui,
            fret_window,
            counter,
            last_input,
            shadcn_checked,
            platform: fret_runner_winit::WinitPlatform::default(),
            #[cfg(target_arch = "wasm32")]
            web_cursor: None,
            #[cfg(target_arch = "wasm32")]
            web_services,
            #[cfg(target_arch = "wasm32")]
            event_loop_proxy,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn desired_surface_size(window: &dyn Window) -> Option<PhysicalSize<u32>> {
        fn url_debug_enabled() -> bool {
            web_sys::window()
                .and_then(|w| w.location().hash().ok())
                .is_some_and(|h| h.contains("debug"))
        }

        let canvas: web_sys::HtmlCanvasElement = window.canvas()?.clone();
        let web_window = web_sys::window()?;
        let dpr = web_window.device_pixel_ratio().max(1.0);
        let css_w = canvas.client_width().max(0) as f64;
        let css_h = canvas.client_height().max(0) as f64;
        let physical = PhysicalSize::new(
            (css_w * dpr).round().max(1.0) as u32,
            (css_h * dpr).round().max(1.0) as u32,
        );

        // Important: wgpu's web surface is backed by the canvas *attribute* size, not CSS size.
        if canvas.width() != physical.width {
            canvas.set_width(physical.width);
        }
        if canvas.height() != physical.height {
            canvas.set_height(physical.height);
        }

        if url_debug_enabled() {
            web_sys::console::log_1(&JsValue::from_str(&format!(
                "canvas css={}x{} dpr={:.2} backing={}x{}",
                css_w,
                css_h,
                dpr,
                canvas.width(),
                canvas.height()
            )));
        }

        Some(physical)
    }

    fn ensure_root(&mut self, bounds: Rect, services: &mut dyn UiServices) {
        let counter = self.counter.clone();
        let last_input = self.last_input.clone();
        let shadcn_checked = self.shadcn_checked.clone();
        let window = self.fret_window;

        let root = declarative::RenderRootContext::new(
            &mut self.ui,
            &mut self.app,
            services,
            window,
            bounds,
        )
        .render_root("demo-web", move |cx| {
            cx.observe_model(&counter, Invalidation::Layout);
            cx.observe_model(&last_input, Invalidation::Layout);
            cx.observe_model(&shadcn_checked, Invalidation::Layout);

            let theme = Theme::global(&*cx.app).clone();

            let mut fill = LayoutStyle::default();
            fill.size.width = Length::Fill;
            fill.size.height = Length::Fill;

            let counter_value = cx.app.models().read(&counter, |v| *v).unwrap_or(0);
            let last_input_value = cx
                .app
                .models()
                .read(&last_input, |v| v.clone())
                .unwrap_or_else(|_| Arc::<str>::from("input: <error>"));

            let label: Arc<str> = Arc::from(format!(
                "fret web demo (winit) — clicks: {counter_value} (click or press Enter)",
            ));

            vec![cx.container(
                ContainerProps {
                    layout: fill,
                    background: Some(theme.colors.surface_background),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: fill,
                            direction: fret_core::Axis::Vertical,
                            gap: fret_core::Px(12.0),
                            padding: fret_core::Edges::all(theme.metrics.padding_md),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        |cx| {
                            let mut button_layout = LayoutStyle::default();
                            button_layout.size.width = Length::Px(fret_core::Px(360.0));

                            vec![
                                cx.text(label.clone()),
                                cx.text(last_input_value.clone()),
                                shadcn::Checkbox::new(shadcn_checked.clone())
                                    .a11y_label("Enable shadcn checkbox")
                                    .into_element(cx),
                                cx.pressable(
                                    PressableProps {
                                        layout: button_layout,
                                        ..Default::default()
                                    },
                                    |cx, _state| {
                                        let counter = counter.clone();
                                        #[allow(clippy::arc_with_non_send_sync)]
                                        cx.pressable_on_activate(Arc::new(
                                            move |host, _acx, _reason| {
                                                let _ = host.models_mut().update(&counter, |v| {
                                                    *v = v.saturating_add(1);
                                                });
                                            },
                                        ));

                                        let mut inner = LayoutStyle::default();
                                        inner.size.width = Length::Fill;
                                        inner.size.height = Length::Px(fret_core::Px(48.0));

                                        vec![cx.container(
                                            ContainerProps {
                                                layout: inner,
                                                padding: fret_core::Edges::all(
                                                    theme.metrics.padding_md,
                                                ),
                                                background: Some(theme.colors.panel_background),
                                                border: fret_core::Edges::all(fret_core::Px(1.0)),
                                                border_color: Some(theme.colors.panel_border),
                                                corner_radii: fret_core::Corners::all(
                                                    theme.metrics.radius_md,
                                                ),
                                                ..Default::default()
                                            },
                                            |cx| vec![cx.text("Click me")],
                                        )]
                                    },
                                ),
                            ]
                        },
                    )]
                },
            )]
        });

        self.ui.set_root(root);
    }

    fn handle_effects(&mut self, window: &dyn Window, gfx: &mut GfxState) {
        let effects = self.app.flush_effects();
        if effects.is_empty() {
            return;
        }

        #[cfg(target_arch = "wasm32")]
        let effects = self.web_services.handle_effects(&mut self.app, effects);

        #[cfg(not(target_arch = "wasm32"))]
        let effects = effects;

        #[cfg(target_arch = "wasm32")]
        self.pending_events.extend(self.web_services.take_events());

        for effect in effects {
            match effect {
                Effect::TextAddFonts { fonts } => {
                    let added = gfx.renderer.add_fonts(fonts);
                    if added == 0 {
                        continue;
                    }

                    let families = gfx.renderer.all_font_names();
                    let update = fret_runtime::apply_font_catalog_update(
                        &mut self.app,
                        families,
                        fret_runtime::FontFamilyDefaultsPolicy::FillIfEmpty,
                    );
                    let _ = gfx.renderer.set_text_font_families(&update.config);
                    window.request_redraw();
                }
                Effect::CursorSetIcon { icon, .. } => {
                    let cursor = match icon {
                        fret_core::CursorIcon::Default => winit::cursor::CursorIcon::Default,
                        fret_core::CursorIcon::Pointer => winit::cursor::CursorIcon::Pointer,
                        fret_core::CursorIcon::Text => winit::cursor::CursorIcon::Text,
                        fret_core::CursorIcon::ColResize => winit::cursor::CursorIcon::ColResize,
                        fret_core::CursorIcon::RowResize => winit::cursor::CursorIcon::RowResize,
                    };
                    window.set_cursor(Cursor::Icon(cursor));
                }
                Effect::Redraw(_) | Effect::RequestAnimationFrame(_) => {
                    window.request_redraw();
                }
                _ => {}
            }
        }
    }

    fn dispatch_fret_event(&mut self, services: &mut dyn UiServices, event: &FretEvent) {
        self.ui.dispatch_event(&mut self.app, services, event);
    }

    fn update_last_input(&mut self, event: &FretEvent) {
        let Some(msg) = (match event {
            FretEvent::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => Some(format!(
                    "input: pointer move @ ({:.1}, {:.1})",
                    position.x.0, position.y.0
                )),
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => Some(format!(
                    "input: pointer down {button:?} @ ({:.1}, {:.1})",
                    position.x.0, position.y.0
                )),
                fret_core::PointerEvent::Up {
                    position, button, ..
                } => Some(format!(
                    "input: pointer up {button:?} @ ({:.1}, {:.1})",
                    position.x.0, position.y.0
                )),
                fret_core::PointerEvent::Wheel {
                    position, delta, ..
                } => Some(format!(
                    "input: wheel @ ({:.1}, {:.1}) Δ({:.1}, {:.1})",
                    position.x.0, position.y.0, delta.x.0, delta.y.0
                )),
            },
            FretEvent::KeyDown { key, .. } => Some(format!("input: key down {key:?}")),
            FretEvent::KeyUp { key, .. } => Some(format!("input: key up {key:?}")),
            _ => None,
        }) else {
            return;
        };

        let msg: Arc<str> = Arc::from(msg);
        let _ = self.app.models_mut().update(&self.last_input, |v| {
            *v = msg.clone();
        });
    }

    fn tick_ui(&mut self, window: &dyn Window, gfx: &mut GfxState) {
        self.tick_id.0 = self.tick_id.0.saturating_add(1);
        self.frame_id.0 = self.frame_id.0.saturating_add(1);
        self.app.set_tick_id(self.tick_id);
        self.app.set_frame_id(self.frame_id);

        let scale = window.scale_factor();

        #[cfg(target_arch = "wasm32")]
        let physical = Self::desired_surface_size(window).unwrap_or_else(|| window.surface_size());
        #[cfg(not(target_arch = "wasm32"))]
        let physical = window.surface_size();

        let logical: LogicalSize<f32> = physical.to_logical(scale);
        gfx.scale_factor = scale as f32;
        let scale_factor = gfx.scale_factor;

        let (cur_w, cur_h) = gfx.surface_state.size();
        if (cur_w, cur_h) != (physical.width.max(1), physical.height.max(1)) {
            gfx.resize(physical);
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&JsValue::from_str(&format!(
                "surface resize -> {}x{} (from {}x{})",
                physical.width, physical.height, cur_w, cur_h
            )));
        }

        let bounds = gfx.ui_bounds(logical);

        // Some effects (notably `TextAddFonts`) must be applied before first paint.
        self.handle_effects(window, gfx);

        let (services, scene) = gfx.services_and_scene_mut();

        let events = std::mem::take(&mut self.pending_events);
        for event in events {
            self.update_last_input(&event);
            self.dispatch_fret_event(services, &event);
        }

        let changed_models = self.app.take_changed_models();
        let _ = self
            .ui
            .propagate_model_changes(&mut self.app, &changed_models);
        let changed_globals = self.app.take_changed_globals();
        let _ = self
            .ui
            .propagate_global_changes(&mut self.app, &changed_globals);

        self.ensure_root(bounds, services);

        self.ui.request_semantics_snapshot();
        self.ui.ingest_paint_cache_source(scene);
        scene.clear();

        let mut frame = UiFrameCx::new(
            &mut self.ui,
            &mut self.app,
            services,
            self.fret_window,
            bounds,
            scale_factor,
        );
        frame.layout_all();
        frame.paint_all(scene);

        #[cfg(target_arch = "wasm32")]
        if scene.ops_len() == 0 {
            web_sys::console::warn_1(&JsValue::from_str("paint produced 0 scene ops"));
        }
        #[cfg(target_arch = "wasm32")]
        if web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .is_some_and(|h| h.contains("debug"))
        {
            use fret_core::scene::{DrawOrder, SceneOp};
            use fret_core::{Color, Corners, Edges};

            // Draw a tiny debug quad (unclipped, no text) to validate the wgpu pipeline end-to-end.
            scene.push(SceneOp::Quad {
                order: DrawOrder(u32::MAX),
                rect: Rect::new(
                    Point::new(Px(24.0), Px(24.0)),
                    Size::new(Px(120.0), Px(120.0)),
                ),
                background: Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                border: Edges::all(Px(0.0)),
                border_color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        let _ = gfx.render();

        self.handle_effects(window, gfx);
    }
}

impl ApplicationHandler for WebDemoApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.resumed(event_loop);
    }

    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        #[cfg(target_arch = "wasm32")]
        let canvas = match fret_runner_winit::canvas_by_id(&self.canvas_id) {
            Ok(c) => Some(c),
            Err(err) => {
                web_sys::console::error_1(&JsValue::from_str(&err.to_string()));
                None
            }
        };

        #[cfg(not(target_arch = "wasm32"))]
        let canvas = None;

        let mut attrs = WindowAttributes::default().with_title("fret demo (web)");

        #[cfg(target_arch = "wasm32")]
        {
            attrs = attrs.with_platform_attributes(Box::new(
                WindowAttributesWeb::default()
                    .with_canvas(canvas)
                    .with_append(false)
                    .with_focusable(true)
                    .with_prevent_default(true),
            ));
        }

        let window = match event_loop.create_window(attrs) {
            Ok(w) => w,
            Err(err) => {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::error_1(&JsValue::from_str(&format!(
                    "failed to create window: {err:?}"
                )));
                return;
            }
        };

        let window: Rc<dyn Window> = window.into();
        self.window_id = Some(window.id());

        #[cfg(target_arch = "wasm32")]
        if self.web_cursor.is_none() {
            let proxy = self.event_loop_proxy.clone();
            match fret_runner_winit::install_web_cursor_listener(window.as_ref(), move || {
                proxy.wake_up();
            }) {
                Ok(listener) => self.web_cursor = Some(listener),
                Err(err) => {
                    web_sys::console::error_1(&JsValue::from_str(&format!(
                        "failed to install web cursor listener: {err}"
                    )));
                }
            }
        }

        // Kick off wgpu init async. We build the surface from the canvas handle, which allows
        // storing `SurfaceState<'static>` without tying it to the winit `Window` lifetime.
        #[cfg(target_arch = "wasm32")]
        if let Some(canvas) = window.canvas().map(|c| c.clone()) {
            let gfx_slot = self.gfx.clone();
            let proxy = self.event_loop_proxy.clone();
            spawn_local(async move {
                // Resize the canvas backing store before creating/configuring the surface.
                let (width, height) = {
                    let web_window = web_sys::window().unwrap();
                    let dpr = web_window.device_pixel_ratio().max(1.0);
                    let css_w = canvas.client_width().max(0) as f64;
                    let css_h = canvas.client_height().max(0) as f64;
                    let w = (css_w * dpr).round().max(1.0) as u32;
                    let h = (css_h * dpr).round().max(1.0) as u32;
                    canvas.set_width(w);
                    canvas.set_height(h);
                    (w, h)
                };
                let (ctx, surface) = match WgpuContext::new_with_surface(
                    wgpu::SurfaceTarget::Canvas(canvas),
                )
                .await
                {
                    Ok(v) => v,
                    Err(err) => {
                        web_sys::console::error_1(&JsValue::from_str(&format!(
                            "wgpu init failed: {err}"
                        )));
                        return;
                    }
                };

                let surface_state =
                    match SurfaceState::new(&ctx.adapter, &ctx.device, surface, width, height) {
                        Ok(v) => v,
                        Err(err) => {
                            web_sys::console::error_1(&JsValue::from_str(&format!(
                                "surface init failed: {err}"
                            )));
                            return;
                        }
                    };

                let renderer = Renderer::new(&ctx.adapter, &ctx.device);
                let clear = ClearColor::default();
                let scene = Scene::default();
                let scale_factor = 1.0;

                web_sys::console::log_1(&JsValue::from_str(&format!(
                    "wgpu surface: {}x{} format={:?} present={:?} alpha={:?}",
                    surface_state.config.width,
                    surface_state.config.height,
                    surface_state.config.format,
                    surface_state.config.present_mode,
                    surface_state.config.alpha_mode,
                )));

                *gfx_slot.borrow_mut() = Some(GfxState {
                    ctx,
                    surface_state,
                    renderer,
                    clear,
                    scene,
                    scale_factor,
                    last_surface_error: None,
                });
                proxy.wake_up();
            });
        }

        // Load a small default font so text shaping doesn't panic before user-provided fonts.
        let fonts = fret_fonts::default_fonts()
            .iter()
            .map(|bytes| bytes.to_vec())
            .collect::<Vec<_>>();
        self.app.push_effect(Effect::TextAddFonts { fonts });

        self.window = Some(window);

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn proxy_wake_up(&mut self, _event_loop: &dyn ActiveEventLoop) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let window = window.as_ref();
        let scale = window.scale_factor();

        self.platform
            .input
            .poll_web_cursor_updates(scale, &mut self.pending_events);

        self.web_services.tick();
        self.pending_events.extend(self.web_services.take_events());

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        _event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if Some(window_id) != self.window_id {
            return;
        }
        let Some(window) = self.window.as_ref().cloned() else {
            return;
        };
        let window = window.as_ref();

        #[cfg(target_arch = "wasm32")]
        if let WindowEvent::PointerButton { state, button, .. } = &event {
            web_sys::console::log_1(&JsValue::from_str(&format!(
                "winit mouse input: {state:?} {button:?}"
            )));
        }

        match &event {
            WindowEvent::CloseRequested => {
                self.pending_events.push(FretEvent::WindowCloseRequested);
                window.request_redraw();
            }
            WindowEvent::SurfaceResized(size) => {
                if let Some(gfx) = self.gfx.borrow_mut().as_mut() {
                    gfx.resize(*size);
                }
                self.platform.handle_window_event(
                    window.scale_factor(),
                    &event,
                    &mut self.pending_events,
                );
                window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(gfx) = self.gfx.borrow_mut().as_mut() {
                    #[cfg(target_arch = "wasm32")]
                    let size =
                        Self::desired_surface_size(window).unwrap_or_else(|| window.surface_size());
                    #[cfg(not(target_arch = "wasm32"))]
                    let size = window.surface_size();
                    gfx.resize(size);
                }
                self.platform.handle_window_event(
                    window.scale_factor(),
                    &event,
                    &mut self.pending_events,
                );
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let mut gfx = {
                    let mut slot = self.gfx.borrow_mut();
                    slot.take()
                };
                let Some(mut gfx) = gfx.take() else {
                    return;
                };

                self.tick_ui(window, &mut gfx);

                *self.gfx.borrow_mut() = Some(gfx);
            }
            _ => {
                self.platform.handle_window_event(
                    window.scale_factor(),
                    &event,
                    &mut self.pending_events,
                );
                if !self.pending_events.is_empty() {
                    window.request_redraw();
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let event_loop = EventLoop::new().map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
    let proxy = event_loop.create_proxy();
    let app = WebDemoApp::new("fret-canvas".to_string(), proxy);

    #[cfg(target_arch = "wasm32")]
    {
        event_loop
            .run_app(app)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}
