use fret_core::AppWindowId;
use fret_core::time::Instant;

use super::redraw_hitch::redraw_hitch_config;
use super::{ActiveEventLoop, EngineFrameUpdate, WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_window_redraw_requested(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        app_window: AppWindowId,
    ) {
        let redraw_span = tracing::info_span!(
            "fret.runner.redraw",
            window = ?app_window,
            tick_id = self.app.tick_id().0,
            frame_id = self.app.frame_id().0,
        );
        let _redraw_guard = redraw_span.enter();

        self.handle_window_redraw_pending_wheel(app_window);

        let window_ref = self.windows.get(app_window).map(|s| s.window.clone());
        if let Some(window_ref) = window_ref {
            let _ = self.update_window_environment_for_window_ref(app_window, window_ref.as_ref());
        }

        let hitch_config = redraw_hitch_config();
        let hitch_total_started = hitch_config.map(|_| Instant::now());
        let mut hitch_prepare_ms: Option<u64> = None;
        let mut hitch_render_ms: Option<u64> = None;
        let mut hitch_record_ms: Option<u64> = None;
        let mut hitch_present_ms: Option<u64> = None;

        // Drain effects before rendering so dock ops, invalidation bumps, and window requests
        // apply deterministically to the frame being drawn (ADR 0013).
        self.drain_effects(event_loop);

        #[cfg(feature = "diag-screenshots")]
        super::window_redraw_diag_screenshots::poll_window_redraw_diag_screenshot_requests(
            self.diag_screenshots.as_mut(),
        );

        self.handle_window_redraw_pending_surface_resize(app_window);

        #[cfg(target_os = "android")]
        let mut android_soft_input_request: Option<bool> = None;

        {
            let (Some(context), Some(renderer)) = (self.context.as_ref(), self.renderer.as_mut())
            else {
                return;
            };
            let Some(state) = self.windows.get_mut(app_window) else {
                return;
            };
            let Some(surface) = state.surface.as_mut() else {
                return;
            };

            let capturing =
                super::window_redraw_renderdoc_capture::begin_window_redraw_renderdoc_capture(
                    self.renderdoc.as_mut(),
                );

            let (prepared, prepare_elapsed) =
                super::window_redraw_frame_prepare::prepare_window_redraw_frame(
                    super::window_redraw_frame_prepare::WindowRedrawFramePrepareInput {
                        app: &mut self.app,
                        driver: &mut self.driver,
                        app_window,
                        user: &mut state.user,
                        platform: &mut state.platform,
                        window: state.window.as_ref(),
                        context,
                        renderer,
                        hitch_enabled: hitch_config.is_some(),
                    },
                );
            let scale_factor = prepared.scale_factor;
            let bounds = prepared.bounds;
            if let Some(elapsed) = prepare_elapsed {
                hitch_prepare_ms = Some(elapsed.as_millis() as u64);
            }

            let (rendered, render_elapsed) =
                super::window_redraw_render::render_window_redraw_frame(
                    super::window_redraw_render::WindowRedrawRenderInput {
                        app: &mut self.app,
                        driver: &mut self.driver,
                        renderer,
                        app_window,
                        user: &mut state.user,
                        scene: &mut state.scene,
                        bounds,
                        scale_factor,
                        hitch_enabled: hitch_config.is_some(),
                    },
                );
            let text_diagnostics = rendered.text_diagnostics;
            if let Some(elapsed) = render_elapsed {
                hitch_render_ms = Some(elapsed.as_millis() as u64);
            }

            #[cfg(target_os = "android")]
            super::window_redraw_text_input::apply_window_redraw_text_input_snapshot(
                &self.app,
                app_window,
                &mut state.platform,
                state.window.as_ref(),
                &mut android_soft_input_request,
            );
            #[cfg(not(target_os = "android"))]
            super::window_redraw_text_input::apply_window_redraw_text_input_snapshot(
                &self.app,
                app_window,
                &mut state.platform,
                state.window.as_ref(),
            );

            super::render::validate_scene_if_enabled(&state.scene);

            let accessibility_scale_factor = state.window.scale_factor();
            super::window_redraw_accessibility::update_window_redraw_accessibility_snapshot(
                &mut self.driver,
                &mut self.app,
                app_window,
                &mut state.user,
                &mut state.accessibility,
                &mut state.last_semantics_snapshot,
                accessibility_scale_factor,
            );

            let (engine_frame, record_elapsed) =
                super::window_redraw_record::record_window_redraw_frame(
                    super::window_redraw_record::WindowRedrawRecordInput {
                        app: &mut self.app,
                        driver: &mut self.driver,
                        app_window,
                        user: &mut state.user,
                        context,
                        renderer,
                        scale_factor,
                        tick_id: self.tick_id,
                        frame_id: self.frame_id,
                        scene_ops: state.scene.ops_len(),
                        hitch_enabled: hitch_config.is_some(),
                    },
                );
            let EngineFrameUpdate {
                target_updates,
                command_buffers: engine_command_buffers,
                keepalive: engine_keepalive,
            } = engine_frame;
            if let Some(elapsed) = record_elapsed {
                hitch_record_ms = Some(elapsed.as_millis() as u64);
            }

            super::window_redraw_webviews::sync_window_redraw_webviews(
                super::window_redraw_webviews::WindowRedrawWebViewSyncInput {
                    app: &mut self.app,
                    driver: &mut self.driver,
                    webviews: &mut self.webviews,
                    frame_id: self.frame_id,
                    app_window,
                    user: &mut state.user,
                    window: state.window.as_ref(),
                    last_semantics_snapshot: &state.last_semantics_snapshot,
                },
            );

            super::window_redraw_target_updates::apply_window_redraw_target_updates(
                renderer,
                target_updates,
            );

            let (draw_result, present_elapsed) =
                super::window_redraw_present::present_window_redraw_frame(
                    super::window_redraw_present::WindowRedrawPresentInput {
                        app: &mut self.app,
                        driver: &mut self.driver,
                        renderer,
                        context,
                        surface,
                        app_window,
                        user: &mut state.user,
                        scene: &state.scene,
                        tick_id: self.tick_id,
                        frame_id: &mut self.frame_id,
                        scale_factor,
                        clear_color: self.config.clear_color,
                        engine_command_buffers,
                        engine_keepalive,
                        text_diagnostics,
                        #[cfg(feature = "diag-screenshots")]
                        diag_screenshots: &mut self.diag_screenshots,
                        bundle_screenshots: &mut self.diag_bundle_screenshots,
                        hitch_enabled: hitch_config.is_some(),
                    },
                );
            if let Some(elapsed) = present_elapsed {
                hitch_present_ms = Some(elapsed.as_millis() as u64);
            }

            super::window_redraw_renderdoc_capture::end_window_redraw_renderdoc_capture(
                self.renderdoc.as_mut(),
                capturing,
            );

            let scene_ops = state.scene.ops_len();
            if let Err(err) = draw_result {
                let _ = surface;
                let _ = state;
                self.handle_window_redraw_present_error(event_loop, app_window, err);
                return;
            }

            super::window_redraw_hitch_summary::maybe_write_window_redraw_hitch_summary(
                super::window_redraw_hitch_summary::WindowRedrawHitchSummaryInput {
                    config: hitch_config,
                    started: hitch_total_started,
                    app_window,
                    tick_id: self.tick_id,
                    frame_id: self.frame_id,
                    prepare_ms: hitch_prepare_ms,
                    render_ms: hitch_render_ms,
                    record_ms: hitch_record_ms,
                    present_ms: hitch_present_ms,
                    scene_ops,
                    bounds,
                    scale_factor,
                },
            );
        }

        #[cfg(target_os = "android")]
        if let Some(enabled) = android_soft_input_request {
            self.android_force_soft_input(enabled);
        }

        // Drain effects produced during rendering so they do not lag by a frame (for example IME
        // cursor updates, timer-driven docking invalidations, window raise/create effects).
        self.drain_effects(event_loop);
    }
}
