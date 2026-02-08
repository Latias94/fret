use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_render::RenderSceneParams;
use fret_runtime::apply_window_metrics_event;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use super::super::{RenderTargetUpdate, WinitEventContext, WinitRenderContext, WinitWindowContext};
use super::{GfxState, WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
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
