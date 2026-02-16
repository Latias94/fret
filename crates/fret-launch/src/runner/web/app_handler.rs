use std::sync::Arc;

use fret_core::Event;
use fret_render::{Renderer, SurfaceState, WgpuContext};
use wasm_bindgen::JsCast as _;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::spawn_local;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::platform::web::{WindowAttributesWeb, WindowExtWeb};
use winit::window::{Window, WindowAttributes, WindowId};

use super::{GfxState, WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> ApplicationHandler for WinitRunner<D> {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.resumed(event_loop);
    }

    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let canvas = fret_runner_web::canvas_by_id(&self.config.web_canvas_id).ok();
        let append = canvas.is_none();

        let mut attrs =
            WindowAttributes::default().with_title(self.config.main_window_title.clone());
        attrs = attrs.with_platform_attributes(Box::new(
            WindowAttributesWeb::default()
                .with_canvas(canvas)
                .with_append(append)
                .with_focusable(true)
                .with_prevent_default(true),
        ));

        let window = match event_loop.create_window(attrs) {
            Ok(w) => w,
            Err(_) => return,
        };
        let window: Arc<dyn Window> = Arc::<dyn Window>::from(window);
        self.window_id = Some(window.id());

        // DevTools WS messages can arrive while the web app is idle (no continuous frame loop).
        // Wake the runner so diagnostics scripts can start without requiring "always animating".
        if self.devtools_ws_inbox_waker.is_none()
            && let Some(web_window) = web_sys::window()
        {
            let w = window.clone();
            let proxy = self.event_loop_proxy.clone();
            let cb = Closure::wrap(Box::new(move |_evt: web_sys::Event| {
                w.request_redraw();
                if let Some(proxy) = proxy.as_ref() {
                    proxy.wake_up();
                }
            }) as Box<dyn FnMut(_)>);
            let _ = web_window.add_event_listener_with_callback(
                "fret_devtools_ws_inbox",
                cb.as_ref().unchecked_ref(),
            );
            self.devtools_ws_inbox_waker = Some(cb);
        }

        if self.window_state.is_none() {
            let state = self
                .driver
                .create_window_state(&mut self.app, self.app_window);
            self.window_state = Some(state);
            self.driver.init(&mut self.app, self.app_window);
        }

        if self.web_cursor.is_none() {
            if let Some(proxy) = self.event_loop_proxy.clone() {
                if let Some(canvas) = window.canvas().map(|c| c.clone()) {
                    if let Ok(listener) =
                        fret_runner_web::install_canvas_cursor_listener(canvas, move || {
                            proxy.wake_up();
                        })
                    {
                        self.web_cursor = Some(listener);
                    }
                }
            }
        }

        if let Some(canvas) = window.canvas().map(|c| c.clone())
            && let Some(mount) = super::ime_mount::ensure_canvas_ime_mount(&canvas)
        {
            self.web_services.register_ime_mount(self.app_window, mount);
        }

        if let Some(canvas) = window.canvas().map(|c| c.clone()) {
            let gfx_slot = self.pending_gfx.clone();
            let proxy = self.event_loop_proxy.clone();
            let svg_budget = self.config.svg_raster_budget_bytes;
            let intermediate_budget = self.config.renderer_intermediate_budget_bytes;
            let msaa = self.config.path_msaa_samples;
            let font_config = self.config.text_font_families.clone();
            spawn_local(async move {
                struct WebDiagFlags {
                    devtools_ws_configured: bool,
                    renderer_perf_enabled: bool,
                }

                fn diag_flags_from_location() -> WebDiagFlags {
                    let Some(window) = web_sys::window() else {
                        return WebDiagFlags {
                            devtools_ws_configured: false,
                            renderer_perf_enabled: false,
                        };
                    };

                    let location = window.location();
                    let search = location.search().unwrap_or_default();
                    let hash = location.hash().unwrap_or_default();

                    fn parse_query_params(query: &str) -> Option<web_sys::UrlSearchParams> {
                        let query = query.trim();
                        if query.is_empty() {
                            return None;
                        }
                        let query = query.trim_start_matches('?');
                        web_sys::UrlSearchParams::new_with_str(query).ok()
                    }

                    fn parse_hash_query_params(hash: &str) -> Option<web_sys::UrlSearchParams> {
                        let hash = hash.trim();
                        if hash.is_empty() {
                            return None;
                        }

                        let hash = hash.trim_start_matches('#');
                        let query = hash.split_once('?').map(|(_, q)| q).unwrap_or(hash);
                        parse_query_params(query)
                    }

                    fn is_truthy_flag(v: Option<String>) -> bool {
                        v.as_deref()
                            .is_some_and(|v| matches!(v.trim(), "1" | "true" | "yes" | "on"))
                    }

                    fn has_devtools_params(params: &web_sys::UrlSearchParams) -> bool {
                        params.get("fret_devtools_ws").is_some()
                            && params.get("fret_devtools_token").is_some()
                    }

                    let mut devtools_ws_configured = false;
                    let mut renderer_perf_enabled = false;

                    if let Some(params) = parse_query_params(&search) {
                        devtools_ws_configured |= has_devtools_params(&params);
                        renderer_perf_enabled |=
                            is_truthy_flag(params.get("fret_diag_renderer_perf"));
                    }
                    if let Some(params) = parse_hash_query_params(&hash) {
                        devtools_ws_configured |= has_devtools_params(&params);
                        renderer_perf_enabled |=
                            is_truthy_flag(params.get("fret_diag_renderer_perf"));
                    }

                    if !devtools_ws_configured {
                        let ws_global = js_sys::Reflect::get(
                            window.as_ref(),
                            &wasm_bindgen::JsValue::from_str("__FRET_DEVTOOLS_WS"),
                        )
                        .ok()
                        .and_then(|v| v.as_string());
                        let token_global = js_sys::Reflect::get(
                            window.as_ref(),
                            &wasm_bindgen::JsValue::from_str("__FRET_DEVTOOLS_TOKEN"),
                        )
                        .ok()
                        .and_then(|v| v.as_string());
                        devtools_ws_configured |= ws_global.is_some() && token_global.is_some();
                    }

                    if devtools_ws_configured {
                        renderer_perf_enabled = true;
                    }

                    WebDiagFlags {
                        devtools_ws_configured,
                        renderer_perf_enabled,
                    }
                }

                let diag_flags = diag_flags_from_location();

                let (width, height) = {
                    let web_window = match web_sys::window() {
                        Some(w) => w,
                        None => return,
                    };
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
                    Err(_) => return,
                };

                let surface_state =
                    match SurfaceState::new(&ctx.adapter, &ctx.device, surface, width, height) {
                        Ok(v) => v,
                        Err(_) => return,
                    };

                let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
                renderer.set_svg_raster_budget_bytes(svg_budget);
                renderer.set_intermediate_budget_bytes(intermediate_budget);
                renderer.set_path_msaa_samples(msaa);
                let _ = renderer.set_text_font_families(&font_config);
                if diag_flags.renderer_perf_enabled {
                    renderer.set_perf_enabled(true);
                }

                *gfx_slot.borrow_mut() = Some(GfxState {
                    ctx,
                    surface_state,
                    renderer,
                    last_surface_error: None,
                    diag_keepalive_redraw: diag_flags.devtools_ws_configured,
                });
                if let Some(proxy) = proxy {
                    proxy.wake_up();
                }
            });
        }

        self.window = Some(window);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.maybe_exit(event_loop) {
            return;
        }
        if self.exiting {
            return;
        }
        let Some(window) = self.window.as_ref() else {
            return;
        };
        self.platform.input.poll_web_cursor_updates(
            window.scale_factor(),
            fret_runner_web::last_cursor_offset_px(),
            &mut self.pending_events,
        );

        self.pending_events
            .extend(self.pending_async_events.borrow_mut().drain(..));

        self.web_services.tick();
        self.pending_events.extend(self.web_services.take_events());

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.maybe_exit(event_loop) {
            return;
        }
        if self.exiting {
            return;
        }
        if Some(window_id) != self.window_id {
            return;
        }
        let Some(window) = self.window.as_ref().cloned() else {
            return;
        };
        let window = window.as_ref();

        match &event {
            WindowEvent::CloseRequested => {
                self.pending_events.push(Event::WindowCloseRequested);
                window.request_redraw();
            }
            WindowEvent::SurfaceResized(size) => {
                if let Some(gfx) = self.gfx.as_mut() {
                    gfx.surface_state.resize(
                        &gfx.ctx.device,
                        size.width.max(1),
                        size.height.max(1),
                    );
                }
                self.platform.handle_window_event(
                    window.scale_factor(),
                    &event,
                    &mut self.pending_events,
                );
                window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(gfx) = self.gfx.as_mut() {
                    let size =
                        Self::desired_surface_size(window).unwrap_or_else(|| window.surface_size());
                    gfx.surface_state.resize(
                        &gfx.ctx.device,
                        size.width.max(1),
                        size.height.max(1),
                    );
                }
                self.platform.handle_window_event(
                    window.scale_factor(),
                    &event,
                    &mut self.pending_events,
                );
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.render_frame(event_loop, window);
            }
            _ => {
                self.platform.handle_window_event(
                    window.scale_factor(),
                    &event,
                    &mut self.pending_events,
                );
                // On Web/WASM, focusing the hidden IME textarea must happen within the same
                // user-activation gesture that triggered the focus change (browser restrictions).
                //
                // The normal "queue events -> request redraw -> drain turns during RedrawRequested"
                // path can run outside of that activation window (e.g. next RAF), causing
                // `textarea.focus()` to be ignored and leaving IME disabled.
                //
                // Flush a bounded number of turns immediately for activation-carrying events so
                // `Effect::ImeAllow { enabled: true }` can be handled synchronously.
                let activation_event = matches!(
                    &event,
                    // Note: some focus behaviors (and thus `Effect::ImeAllow`) can be driven by
                    // the "click" completion semantics (pointer-up). Drain on both pressed and
                    // released to keep textarea `focus()` within the browser activation window.
                    WindowEvent::PointerButton { .. }
                );
                if activation_event
                    && self.gfx.is_some()
                    && self.window_state.is_some()
                    && let (Some(mut gfx), Some(mut state)) =
                        (self.gfx.take(), self.window_state.take())
                {
                    self.drain_turns(event_loop, window, &mut gfx, &mut state);
                    self.window_state = Some(state);
                    self.gfx = Some(gfx);
                }
                if !self.pending_events.is_empty() {
                    window.request_redraw();
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.maybe_exit(event_loop) {
            return;
        }
        event_loop.set_control_flow(ControlFlow::Wait);
    }
}
