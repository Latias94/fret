use std::any::TypeId;
use std::cell::Cell;
use std::rc::Rc;

use fret_core::{
    AppWindowId, ColorScheme, ContrastPreference, Edges, ForcedColorsMode, Point, Px, Rect, Size,
    WindowMetricsService,
};
use fret_render::RenderSceneParams;
use fret_runtime::apply_window_metrics_event;
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::closure::Closure;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::platform::web::WindowExtWeb;
use winit::window::Window;

use super::super::{RenderTargetUpdate, WinitEventContext, WinitRenderContext, WinitWindowContext};
use super::{GfxState, WinitAppDriver, WinitRunner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WebEnvironmentPreferenceSnapshot {
    color_scheme: Option<ColorScheme>,
    contrast_preference: Option<ContrastPreference>,
    forced_colors_mode: Option<ForcedColorsMode>,
    prefers_reduced_motion: Option<bool>,
    prefers_reduced_transparency: Option<bool>,
}

pub(super) struct WebEnvironmentMediaQueries {
    dirty: Rc<Cell<bool>>,
    prefers_reduced_motion: Option<web_sys::MediaQueryList>,
    prefers_reduced_transparency: Option<web_sys::MediaQueryList>,
    prefers_color_scheme_dark: Option<web_sys::MediaQueryList>,
    prefers_contrast_more: Option<web_sys::MediaQueryList>,
    prefers_contrast_less: Option<web_sys::MediaQueryList>,
    prefers_contrast_custom: Option<web_sys::MediaQueryList>,
    prefers_contrast_no_preference: Option<web_sys::MediaQueryList>,
    forced_colors_active: Option<web_sys::MediaQueryList>,
    _listeners: Vec<Closure<dyn FnMut(web_sys::MediaQueryListEvent)>>,
}

impl WebEnvironmentMediaQueries {
    fn new(window: &web_sys::Window) -> Self {
        let dirty = Rc::new(Cell::new(true));
        let mut listeners: Vec<Closure<dyn FnMut(web_sys::MediaQueryListEvent)>> = Vec::new();

        let mql = |q: &str| window.match_media(q).ok().flatten();

        let prefers_reduced_motion = mql("(prefers-reduced-motion: reduce)");
        let prefers_reduced_transparency = mql("(prefers-reduced-transparency: reduce)");
        let prefers_color_scheme_dark = mql("(prefers-color-scheme: dark)");

        let prefers_contrast_more = mql("(prefers-contrast: more)");
        let prefers_contrast_less = mql("(prefers-contrast: less)");
        let prefers_contrast_custom = mql("(prefers-contrast: custom)");
        let prefers_contrast_no_preference = mql("(prefers-contrast: no-preference)");

        let forced_colors_active = mql("(forced-colors: active)");

        let mut attach = |list: &Option<web_sys::MediaQueryList>| {
            let Some(list) = list.as_ref() else {
                return;
            };
            let dirty = dirty.clone();
            let cb = Closure::wrap(Box::new(move |_evt: web_sys::MediaQueryListEvent| {
                dirty.set(true);
            }) as Box<dyn FnMut(_)>);
            // Prefer the standard `change` event listener.
            // Fallback to the deprecated `addListener` API for older browsers.
            let callback: &js_sys::Function = cb.as_ref().unchecked_ref();
            if list
                .add_event_listener_with_callback("change", callback)
                .is_err()
            {
                let _ = list.add_listener_with_opt_callback(Some(callback));
            }
            listeners.push(cb);
        };

        attach(&prefers_reduced_motion);
        attach(&prefers_reduced_transparency);
        attach(&prefers_color_scheme_dark);
        attach(&prefers_contrast_more);
        attach(&prefers_contrast_less);
        attach(&prefers_contrast_custom);
        attach(&prefers_contrast_no_preference);
        attach(&forced_colors_active);

        Self {
            dirty,
            prefers_reduced_motion,
            prefers_reduced_transparency,
            prefers_color_scheme_dark,
            prefers_contrast_more,
            prefers_contrast_less,
            prefers_contrast_custom,
            prefers_contrast_no_preference,
            forced_colors_active,
            _listeners: listeners,
        }
    }

    fn take_dirty(&self) -> bool {
        self.dirty.replace(false)
    }

    fn snapshot(&self) -> WebEnvironmentPreferenceSnapshot {
        let prefers_reduced_motion = self.prefers_reduced_motion.as_ref().map(|m| m.matches());
        let prefers_reduced_transparency = self
            .prefers_reduced_transparency
            .as_ref()
            .map(|m| m.matches());

        let color_scheme = self.prefers_color_scheme_dark.as_ref().map(|m| {
            if m.matches() {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            }
        });

        let contrast_preference = {
            let any_supported = self.prefers_contrast_more.is_some()
                || self.prefers_contrast_less.is_some()
                || self.prefers_contrast_custom.is_some()
                || self.prefers_contrast_no_preference.is_some();
            if !any_supported {
                None
            } else if self
                .prefers_contrast_more
                .as_ref()
                .is_some_and(|m| m.matches())
            {
                Some(ContrastPreference::More)
            } else if self
                .prefers_contrast_less
                .as_ref()
                .is_some_and(|m| m.matches())
            {
                Some(ContrastPreference::Less)
            } else if self
                .prefers_contrast_custom
                .as_ref()
                .is_some_and(|m| m.matches())
            {
                Some(ContrastPreference::Custom)
            } else if self
                .prefers_contrast_no_preference
                .as_ref()
                .is_some_and(|m| m.matches())
            {
                Some(ContrastPreference::NoPreference)
            } else {
                None
            }
        };

        let forced_colors_mode = self.forced_colors_active.as_ref().map(|m| {
            if m.matches() {
                ForcedColorsMode::Active
            } else {
                ForcedColorsMode::None
            }
        });

        WebEnvironmentPreferenceSnapshot {
            color_scheme,
            contrast_preference,
            forced_colors_mode,
            prefers_reduced_motion,
            prefers_reduced_transparency,
        }
    }
}

impl<D: WinitAppDriver> WinitRunner<D> {
    fn update_window_environment_for_frame(&mut self, window: &dyn Window) {
        let Some(web_window) = web_sys::window() else {
            return;
        };

        let media_queries = self
            .environment_media_queries
            .get_or_insert_with(|| WebEnvironmentMediaQueries::new(&web_window));
        let pref_snapshot = media_queries.take_dirty().then(|| media_queries.snapshot());
        let safe_area_insets = read_safe_area_insets(&web_window, window);
        let occlusion_insets = read_occlusion_insets(&web_window);
        let text_scale_factor = read_text_scale_factor(&web_window);

        let (
            prev_scheme_known,
            prev_scheme,
            prev_contrast_known,
            prev_contrast,
            prev_forced_known,
            prev_forced,
            prev_motion_known,
            prev_motion,
            prev_transparency_known,
            prev_transparency,
            prev_text_scale_known,
            prev_text_scale,
            prev_safe_known,
            prev_safe_area_insets,
            prev_occlusion_known,
            prev_occlusion_insets,
        ) = if let Some(svc) = self.app.global::<WindowMetricsService>() {
            (
                svc.color_scheme_is_known(self.app_window),
                svc.color_scheme(self.app_window),
                svc.contrast_preference_is_known(self.app_window),
                svc.contrast_preference(self.app_window),
                svc.forced_colors_mode_is_known(self.app_window),
                svc.forced_colors_mode(self.app_window),
                svc.prefers_reduced_motion_is_known(self.app_window),
                svc.prefers_reduced_motion(self.app_window),
                svc.prefers_reduced_transparency_is_known(self.app_window),
                svc.prefers_reduced_transparency(self.app_window),
                svc.text_scale_factor_is_known(self.app_window),
                svc.text_scale_factor(self.app_window),
                svc.safe_area_insets_is_known(self.app_window),
                svc.safe_area_insets(self.app_window),
                svc.occlusion_insets_is_known(self.app_window),
                svc.occlusion_insets(self.app_window),
            )
        } else {
            (
                false, None, false, None, false, None, false, None, false, None, false, None,
                false, None, false, None,
            )
        };

        let (needs_scheme, needs_contrast, needs_forced, needs_motion, needs_transparency) =
            pref_snapshot
                .map(|pref_snapshot| {
                    (
                        !prev_scheme_known || prev_scheme != pref_snapshot.color_scheme,
                        !prev_contrast_known || prev_contrast != pref_snapshot.contrast_preference,
                        !prev_forced_known || prev_forced != pref_snapshot.forced_colors_mode,
                        !prev_motion_known || prev_motion != pref_snapshot.prefers_reduced_motion,
                        !prev_transparency_known
                            || prev_transparency != pref_snapshot.prefers_reduced_transparency,
                    )
                })
                .unwrap_or((false, false, false, false, false));

        let needs_text_scale = !prev_text_scale_known
            || match (prev_text_scale, text_scale_factor) {
                (Some(a), Some(b)) => (a - b).abs() > 0.0001,
                (None, None) => false,
                _ => true,
            };

        let needs_safe = !prev_safe_known || prev_safe_area_insets != safe_area_insets;
        let needs_occlusion = !prev_occlusion_known || prev_occlusion_insets != occlusion_insets;

        if needs_scheme
            || needs_contrast
            || needs_forced
            || needs_motion
            || needs_transparency
            || needs_text_scale
            || needs_safe
            || needs_occlusion
        {
            self.app
                .with_global_mut(WindowMetricsService::default, |svc, _app| {
                    if let Some(pref_snapshot) = pref_snapshot {
                        if needs_scheme {
                            svc.set_color_scheme(self.app_window, pref_snapshot.color_scheme);
                        }
                        if needs_contrast {
                            svc.set_contrast_preference(
                                self.app_window,
                                pref_snapshot.contrast_preference,
                            );
                        }
                        if needs_forced {
                            svc.set_forced_colors_mode(
                                self.app_window,
                                pref_snapshot.forced_colors_mode,
                            );
                        }
                        if needs_motion {
                            svc.set_prefers_reduced_motion(
                                self.app_window,
                                pref_snapshot.prefers_reduced_motion,
                            );
                        }
                        if needs_transparency {
                            svc.set_prefers_reduced_transparency(
                                self.app_window,
                                pref_snapshot.prefers_reduced_transparency,
                            );
                        }
                    }

                    if needs_text_scale {
                        svc.set_text_scale_factor(self.app_window, text_scale_factor);
                    }

                    if needs_safe {
                        svc.set_safe_area_insets(self.app_window, safe_area_insets);
                    }
                    if needs_occlusion {
                        svc.set_occlusion_insets(self.app_window, occlusion_insets);
                    }
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
            if changed_globals.contains(&TypeId::of::<fret_runtime::fret_i18n::I18nService>()) {
                let locale = self
                    .app
                    .global::<fret_runtime::fret_i18n::I18nService>()
                    .and_then(|service| service.preferred_locales().first())
                    .map(|locale| locale.to_string());
                if gfx.renderer.set_text_locale(locale.as_deref()) {
                    self.app.set_global::<fret_runtime::TextFontStackKey>(
                        fret_runtime::TextFontStackKey(gfx.renderer.text_font_stack_key()),
                    );
                    self.app.request_redraw(self.app_window);
                }
            }
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
            self.app
                .set_global(gfx.renderer.text_font_trace_snapshot(self.frame_id));
            self.app
                .set_global(gfx.renderer.text_fallback_policy_snapshot(self.frame_id));
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

fn read_text_scale_factor(window: &web_sys::Window) -> Option<f32> {
    const BASE_REM_PX: f32 = 16.0;

    let document = window.document()?;
    let root = document.document_element()?;
    let style = window.get_computed_style(&root).ok()??;
    let font_size = style.get_property_value("font-size").ok()?;
    let px = parse_px(&font_size);
    if px <= 0.0 {
        return None;
    }
    Some(px / BASE_REM_PX)
}

fn parse_px(value: &str) -> f32 {
    let value = value.trim();
    if value.is_empty() {
        return 0.0;
    }

    let px = value.strip_suffix("px").unwrap_or(value).trim();
    px.parse::<f32>().unwrap_or(0.0)
}
