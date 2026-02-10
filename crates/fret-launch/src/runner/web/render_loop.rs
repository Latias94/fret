use fret_core::{AppWindowId, ColorScheme, Edges, Point, Px, Rect, Size, WindowMetricsService};
use fret_render::RenderSceneParams;
use fret_runtime::apply_window_metrics_event;
use wasm_bindgen::JsCast;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::platform::web::WindowExtWeb;
use winit::window::Window;

use super::super::{RenderTargetUpdate, WinitEventContext, WinitRenderContext, WinitWindowContext};
use super::{GfxState, WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    fn update_window_environment_for_frame(&mut self, window: &dyn Window) {
        let Some(web_window) = web_sys::window() else {
            return;
        };

        let color_scheme = read_color_scheme(&web_window);
        let prefers_reduced_motion = read_prefers_reduced_motion(&web_window);
        let safe_area_insets = read_safe_area_insets(&web_window, window);
        let occlusion_insets = read_occlusion_insets(&web_window);

        let metrics = self.app.global::<WindowMetricsService>();

        let prev_scheme_known =
            metrics.is_some_and(|svc| svc.color_scheme_is_known(self.app_window));
        let prev_scheme = metrics.and_then(|svc| svc.color_scheme(self.app_window));
        if !prev_scheme_known || prev_scheme != color_scheme {
            self.app
                .with_global_mut(WindowMetricsService::default, |svc, _app| {
                    svc.set_color_scheme(self.app_window, color_scheme);
                });
        }

        let prev_motion_known =
            metrics.is_some_and(|svc| svc.prefers_reduced_motion_is_known(self.app_window));
        let prev_motion = metrics.and_then(|svc| svc.prefers_reduced_motion(self.app_window));
        if !prev_motion_known || prev_motion != prefers_reduced_motion {
            self.app
                .with_global_mut(WindowMetricsService::default, |svc, _app| {
                    svc.set_prefers_reduced_motion(self.app_window, prefers_reduced_motion);
                });
        }

        let prev_safe_known =
            metrics.is_some_and(|svc| svc.safe_area_insets_is_known(self.app_window));
        let prev_safe_area_insets = metrics.and_then(|svc| svc.safe_area_insets(self.app_window));
        if !prev_safe_known || prev_safe_area_insets != safe_area_insets {
            self.app
                .with_global_mut(WindowMetricsService::default, |svc, _app| {
                    svc.set_safe_area_insets(self.app_window, safe_area_insets);
                });
        }

        let prev_occlusion_known =
            metrics.is_some_and(|svc| svc.occlusion_insets_is_known(self.app_window));
        let prev_occlusion_insets = metrics.and_then(|svc| svc.occlusion_insets(self.app_window));
        if !prev_occlusion_known || prev_occlusion_insets != occlusion_insets {
            self.app
                .with_global_mut(WindowMetricsService::default, |svc, _app| {
                    svc.set_occlusion_insets(self.app_window, occlusion_insets);
                });
        }
    }

    pub(super) fn drain_inboxes(&mut self, window: Option<AppWindowId>) -> bool {
        let did_work = self.app.with_global_mut_untracked(
            fret_runtime::InboxDrainRegistry::default,
            |registry, app| registry.drain_all(app, window),
        );
        tracing::trace!(?window, did_work, "driver: drain_inboxes");
        did_work
    }

    fn dispatch_events(&mut self, gfx: &mut GfxState, state: &mut D::WindowState) -> bool {
        let events = std::mem::take(&mut self.pending_events);
        let mut did_work = !events.is_empty();
        for event in events {
            apply_window_metrics_event(&mut self.app, self.app_window, &event);
            self.driver.handle_event(
                WinitEventContext {
                    app: &mut self.app,
                    services: &mut gfx.renderer,
                    window: self.app_window,
                    state,
                },
                &event,
            );
        }

        let changed_models = self.app.take_changed_models();
        if !changed_models.is_empty() {
            did_work = true;
            self.driver.handle_model_changes(
                WinitWindowContext {
                    app: &mut self.app,
                    window: self.app_window,
                    state,
                },
                &changed_models,
            );
        }

        let changed_globals = self.app.take_changed_globals();
        if !changed_globals.is_empty() {
            did_work = true;
            self.driver.handle_global_changes(
                WinitWindowContext {
                    app: &mut self.app,
                    window: self.app_window,
                    state,
                },
                &changed_globals,
            );
        }

        did_work
    }

    pub(super) fn drain_turns(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window: &dyn Window,
        gfx: &mut GfxState,
        state: &mut D::WindowState,
    ) {
        // ADR 0034: coalesce and bound effect/event draining to prevent unbounded "effect storms"
        // while still allowing same-frame fixed-point progress for common chains.
        const MAX_EFFECT_DRAIN_TURNS: usize = 8;

        for _ in 0..MAX_EFFECT_DRAIN_TURNS {
            if self.exiting {
                break;
            }

            let mut did_work = self.drain_effects(event_loop, window, gfx, state);
            did_work |= self.dispatch_events(gfx, state);
            if !did_work {
                break;
            }
        }
    }

    pub(super) fn render_frame(&mut self, event_loop: &dyn ActiveEventLoop, window: &dyn Window) {
        if self.maybe_exit(event_loop) {
            return;
        }
        if self.exiting {
            return;
        }
        self.adopt_gfx_if_ready();
        self.ensure_gpu_ready_hook();

        let Some(mut gfx) = self.gfx.take() else {
            return;
        };
        let Some(mut state) = self.window_state.take() else {
            self.gfx = Some(gfx);
            return;
        };

        self.tick_id.0 = self.tick_id.0.saturating_add(1);
        self.frame_id.0 = self.frame_id.0.saturating_add(1);
        self.app.set_tick_id(self.tick_id);
        self.app.set_frame_id(self.frame_id);

        self.platform.prepare_frame(window);
        self.update_window_environment_for_frame(window);

        let scale = window.scale_factor();
        let physical = Self::desired_surface_size(window).unwrap_or_else(|| window.surface_size());
        let logical: LogicalSize<f32> = physical.to_logical(scale);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(logical.width), Px(logical.height)),
        );

        let (cur_w, cur_h) = gfx.surface_state.size();
        if (cur_w, cur_h) != (physical.width.max(1), physical.height.max(1)) {
            gfx.surface_state.resize(
                &gfx.ctx.device,
                physical.width.max(1),
                physical.height.max(1),
            );
        }

        self.drain_turns(event_loop, window, &mut gfx, &mut state);

        let scale_factor = scale as f32;
        self.driver.gpu_frame_prepare(
            &mut self.app,
            self.app_window,
            &mut state,
            &gfx.ctx,
            &mut gfx.renderer,
            scale_factor,
        );

        self.scene.clear();
        let render_text_diag_enabled = std::env::var_os("FRET_DIAG_DIR")
            .is_some_and(|v| !v.is_empty())
            || std::env::var_os("FRET_RENDER_TEXT_DEBUG").is_some_and(|v| !v.is_empty());
        if render_text_diag_enabled {
            gfx.renderer.begin_text_diagnostics_frame();
        }
        self.driver.render(WinitRenderContext {
            app: &mut self.app,
            services: &mut gfx.renderer,
            window: self.app_window,
            state: &mut state,
            bounds,
            scale_factor,
            scene: &mut self.scene,
        });

        let engine = self.driver.record_engine_frame(
            &mut self.app,
            self.app_window,
            &mut state,
            &gfx.ctx,
            &mut gfx.renderer,
            scale_factor,
            self.tick_id,
            self.frame_id,
        );
        for update in engine.target_updates {
            match update {
                RenderTargetUpdate::Update { id, desc } => {
                    let _ = gfx.renderer.update_render_target(id, desc);
                }
                RenderTargetUpdate::Unregister { id } => {
                    let _ = gfx.renderer.unregister_render_target(id);
                }
            }
        }

        let (frame, view) = match gfx.surface_state.get_current_frame_view() {
            Ok(v) => v,
            Err(err) => {
                if gfx.last_surface_error.as_ref() != Some(&err) {
                    gfx.last_surface_error = Some(err.clone());
                }
                match err {
                    wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                        let (w, h) = gfx.surface_state.size();
                        gfx.surface_state.resize(&gfx.ctx.device, w, h);
                    }
                    wgpu::SurfaceError::Timeout => {}
                    wgpu::SurfaceError::OutOfMemory => panic!("wgpu surface out of memory"),
                    wgpu::SurfaceError::Other => {}
                }
                return;
            }
        };

        let cmd = gfx.renderer.render_scene(
            &gfx.ctx.device,
            &gfx.ctx.queue,
            RenderSceneParams {
                format: gfx.surface_state.format(),
                target_view: &view,
                scene: &self.scene,
                clear: self.config.clear_color,
                scale_factor,
                viewport_size: gfx.surface_state.size(),
            },
        );
        if render_text_diag_enabled {
            self.app
                .set_global(gfx.renderer.text_diagnostics_snapshot(self.frame_id));
        }

        let mut submit: Vec<wgpu::CommandBuffer> = engine.command_buffers;
        submit.push(cmd);
        gfx.ctx.queue.submit(submit);
        frame.present();

        self.drain_turns(event_loop, window, &mut gfx, &mut state);

        self.window_state = Some(state);
        self.gfx = Some(gfx);
    }
}

fn read_safe_area_insets(
    window: &web_sys::Window,
    winit_window: &dyn winit::window::Window,
) -> Option<Edges> {
    let document = window.document()?;
    let root = document.document_element()?;

    let probe = match document.get_element_by_id("fret_safe_area_probe") {
        Some(existing) => existing,
        None => {
            let probe = document.create_element("div").ok()?;
            probe.set_id("fret_safe_area_probe");
            let probe_el: web_sys::HtmlElement = probe.clone().dyn_into().ok()?;
            let style = probe_el.style();
            let _ = style.set_property("position", "fixed");
            let _ = style.set_property("left", "0");
            let _ = style.set_property("top", "0");
            let _ = style.set_property("width", "0");
            let _ = style.set_property("height", "0");
            let _ = style.set_property("pointer-events", "none");
            let _ = style.set_property("visibility", "hidden");
            let _ = style.set_property(
                "padding",
                "env(safe-area-inset-top) env(safe-area-inset-right) env(safe-area-inset-bottom) env(safe-area-inset-left)",
            );

            // Prefer mounting to the winit canvas parent so we don't depend on `document.body`.
            // This also keeps the probe in the same DOM subtree as the actual UI surface.
            let mounted = winit_window
                .canvas()
                .and_then(|canvas| canvas.parent_node())
                .and_then(|parent| parent.append_child(&probe).ok())
                .is_some();
            if !mounted {
                let _ = root.append_child(&probe);
            }

            probe
        }
    };

    let style = window.get_computed_style(&probe).ok()??;
    let top = parse_px(&style.get_property_value("padding-top").ok()?);
    let right = parse_px(&style.get_property_value("padding-right").ok()?);
    let bottom = parse_px(&style.get_property_value("padding-bottom").ok()?);
    let left = parse_px(&style.get_property_value("padding-left").ok()?);

    Some(Edges {
        top: Px(top),
        right: Px(right),
        bottom: Px(bottom),
        left: Px(left),
    })
}

fn read_occlusion_insets(window: &web_sys::Window) -> Option<Edges> {
    let viewport = window.visual_viewport()?;

    let inner_width = window.inner_width().ok()?.as_f64()?;
    let inner_height = window.inner_height().ok()?.as_f64()?;

    let offset_left = viewport.offset_left().max(0.0);
    let offset_top = viewport.offset_top().max(0.0);
    let visible_width = viewport.width().max(0.0);
    let visible_height = viewport.height().max(0.0);

    let right = (inner_width - (offset_left + visible_width)).max(0.0);
    let bottom = (inner_height - (offset_top + visible_height)).max(0.0);

    Some(Edges {
        top: Px(offset_top as f32),
        right: Px(right as f32),
        bottom: Px(bottom as f32),
        left: Px(offset_left as f32),
    })
}

fn read_prefers_reduced_motion(window: &web_sys::Window) -> Option<bool> {
    let list = window
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()??;
    Some(list.matches())
}

fn read_color_scheme(window: &web_sys::Window) -> Option<ColorScheme> {
    let list = window.match_media("(prefers-color-scheme: dark)").ok()??;
    if list.matches() {
        Some(ColorScheme::Dark)
    } else {
        Some(ColorScheme::Light)
    }
}

fn parse_px(value: &str) -> f32 {
    let value = value.trim();
    if value.is_empty() {
        return 0.0;
    }

    let px = value.strip_suffix("px").unwrap_or(value).trim();
    px.parse::<f32>().unwrap_or(0.0)
}
