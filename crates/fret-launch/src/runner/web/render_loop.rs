use fret_app::Effect;
use fret_core::{AppWindowId, Event, Point, Px, Rect, Size};
use fret_render::RenderSceneParams;
use fret_runtime::{WindowRequest, apply_window_metrics_event};
use winit::cursor::Cursor;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::platform::web::WindowExtWeb;
use winit::window::Window;

use super::super::streaming_upload::StreamingUploadAckKind;
use super::super::{
    RenderTargetUpdate, WinitCommandContext, WinitEventContext, WinitGlobalContext,
    WinitRenderContext, WinitWindowContext,
};
use super::streaming_images;
use super::{GfxState, WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    fn adopt_gfx_if_ready(&mut self) {
        if self.gfx.is_some() {
            return;
        }
        let pending = self.pending_gfx.borrow_mut().take();
        let Some(mut gfx) = pending else {
            return;
        };

        let renderer_caps = fret_render::RendererCapabilities::from_wgpu_context(&gfx.ctx);
        self.app
            .set_global::<fret_render::RendererCapabilities>(renderer_caps.clone());
        self.renderer_caps = Some(renderer_caps);

        self.app
            .set_global::<fret_core::TextFontFamilyConfig>(self.config.text_font_families.clone());
        let _ = gfx
            .renderer
            .set_text_font_families(&self.config.text_font_families);

        // Web/WASM cannot access system fonts. Load our bundled defaults as soon as the renderer
        // becomes available, then seed `TextFontFamilyConfig` deterministically.
        let default_fonts = fret_fonts::default_fonts()
            .iter()
            .map(|bytes| bytes.to_vec())
            .collect::<Vec<_>>();
        let _ = gfx.renderer.add_fonts(default_fonts);

        let update = fret_runtime::apply_font_catalog_update(
            &mut self.app,
            gfx.renderer.all_font_names(),
            fret_runtime::FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates,
        );
        let _ = gfx.renderer.set_text_font_families(&update.config);
        self.app
            .set_global::<fret_runtime::TextFontStackKey>(fret_runtime::TextFontStackKey(
                gfx.renderer.text_font_stack_key(),
            ));

        self.gfx = Some(gfx);
    }

    fn ensure_gpu_ready_hook(&mut self) {
        if self.gpu_ready_called {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        self.driver
            .gpu_ready(&mut self.app, &gfx.ctx, &mut gfx.renderer);
        self.gpu_ready_called = true;
    }

    pub(super) fn desired_surface_size(window: &dyn Window) -> Option<PhysicalSize<u32>> {
        let canvas: web_sys::HtmlCanvasElement = window.canvas()?.clone();
        let web_window = web_sys::window()?;
        let dpr = web_window.device_pixel_ratio().max(1.0);
        let css_w = canvas.client_width().max(0) as f64;
        let css_h = canvas.client_height().max(0) as f64;
        let physical = PhysicalSize::new(
            (css_w * dpr).round().max(1.0) as u32,
            (css_h * dpr).round().max(1.0) as u32,
        );

        if canvas.width() != physical.width {
            canvas.set_width(physical.width);
        }
        if canvas.height() != physical.height {
            canvas.set_height(physical.height);
        }

        Some(physical)
    }

    fn drain_effects(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window: &dyn Window,
        gfx: &mut GfxState,
        state: &mut D::WindowState,
    ) -> bool {
        let did_work = self.dispatcher.drain_turn() || self.drain_inboxes(Some(self.app_window));
        let effects = self.app.flush_effects();
        let effects = self.web_services.handle_effects(&mut self.app, effects);
        self.pending_events.extend(self.web_services.take_events());

        let (effects, mut stats, acks) = self.streaming_uploads.process_effects(
            self.frame_id,
            effects,
            self.config.streaming_upload_budget_bytes_per_frame,
            self.config.streaming_staging_budget_bytes,
            self.config.streaming_update_ack_enabled,
        );
        tracing::trace!(
            did_work,
            effects = effects.len(),
            acks = acks.len(),
            "driver: drain_effects turn"
        );
        if self.config.streaming_update_ack_enabled {
            for ack in acks {
                match ack.kind {
                    StreamingUploadAckKind::Dropped(reason) => {
                        self.pending_events.push(Event::ImageUpdateDropped {
                            token: ack.token,
                            image: ack.image,
                            reason,
                        });
                    }
                }
            }
        }
        let had_effects = !effects.is_empty();
        if !had_effects {
            if self.streaming_uploads.has_pending() {
                window.request_redraw();
            }

            let streaming_snapshot_enabled = self.config.streaming_perf_snapshot_enabled
                || std::env::var_os("FRET_STREAMING_DEBUG").is_some_and(|v| !v.is_empty());
            let streaming_stats_have_activity = stats.update_effects_seen > 0
                || stats.update_effects_enqueued > 0
                || stats.update_effects_replaced > 0
                || stats.update_effects_applied > 0
                || stats.update_effects_delayed_budget > 0
                || stats.update_effects_dropped_staging > 0
                || stats.upload_bytes_budgeted > 0
                || stats.upload_bytes_applied > 0
                || stats.pending_updates > 0
                || stats.pending_staging_bytes > 0
                || stats.yuv_conversions_attempted > 0
                || stats.yuv_convert_us > 0;
            if streaming_snapshot_enabled && streaming_stats_have_activity {
                self.app.set_global(fret_core::StreamingUploadPerfSnapshot {
                    frame_id: self.frame_id,
                    upload_budget_bytes_per_frame: stats.upload_budget_bytes_per_frame,
                    staging_budget_bytes: stats.staging_budget_bytes,
                    update_effects_seen: u64::from(stats.update_effects_seen),
                    update_effects_enqueued: u64::from(stats.update_effects_enqueued),
                    update_effects_replaced: u64::from(stats.update_effects_replaced),
                    update_effects_applied: u64::from(stats.update_effects_applied),
                    update_effects_delayed_budget: u64::from(stats.update_effects_delayed_budget),
                    update_effects_dropped_staging: u64::from(stats.update_effects_dropped_staging),
                    upload_bytes_budgeted: stats.upload_bytes_budgeted,
                    upload_bytes_applied: stats.upload_bytes_applied,
                    pending_updates: u64::from(stats.pending_updates),
                    pending_staging_bytes: stats.pending_staging_bytes,
                    yuv_convert_us: stats.yuv_convert_us,
                    yuv_convert_output_bytes: stats.yuv_convert_output_bytes,
                    yuv_conversions_attempted: u64::from(stats.yuv_conversions_attempted),
                    yuv_conversions_applied: u64::from(stats.yuv_conversions_applied),
                });
            }

            if std::env::var_os("FRET_STREAMING_DEBUG").is_some_and(|v| !v.is_empty())
                && (stats.update_effects_delayed_budget > 0
                    || stats.update_effects_dropped_staging > 0
                    || stats.update_effects_replaced > 0
                    || stats.yuv_conversions_attempted > 0)
            {
                tracing::debug!(
                    seen = stats.update_effects_seen,
                    enqueued = stats.update_effects_enqueued,
                    replaced = stats.update_effects_replaced,
                    applied = stats.update_effects_applied,
                    delayed_budget = stats.update_effects_delayed_budget,
                    dropped_staging = stats.update_effects_dropped_staging,
                    upload_bytes_budgeted = stats.upload_bytes_budgeted,
                    upload_bytes_applied = stats.upload_bytes_applied,
                    upload_budget_bytes_per_frame = stats.upload_budget_bytes_per_frame,
                    staging_budget_bytes = stats.staging_budget_bytes,
                    pending_updates = stats.pending_updates,
                    pending_staging_bytes = stats.pending_staging_bytes,
                    yuv_attempted = stats.yuv_conversions_attempted,
                    yuv_applied = stats.yuv_conversions_applied,
                    yuv_convert_us = stats.yuv_convert_us,
                    yuv_output_bytes = stats.yuv_convert_output_bytes,
                    "streaming image updates queued/budgeted"
                );
            }
            return did_work;
        }

        for effect in effects {
            match effect {
                Effect::TextAddFonts { fonts } => {
                    let added = gfx.renderer.add_fonts(fonts);
                    if added == 0 {
                        continue;
                    }

                    let update = fret_runtime::apply_font_catalog_update(
                        &mut self.app,
                        gfx.renderer.all_font_names(),
                        fret_runtime::FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates,
                    );
                    let _ = gfx.renderer.set_text_font_families(&update.config);
                    self.app.set_global::<fret_runtime::TextFontStackKey>(
                        fret_runtime::TextFontStackKey(gfx.renderer.text_font_stack_key()),
                    );
                    window.request_redraw();
                }
                Effect::CursorSetIcon { icon, .. } => {
                    let cursor = match icon {
                        fret_core::CursorIcon::Default => winit::cursor::CursorIcon::Default,
                        fret_core::CursorIcon::Pointer => winit::cursor::CursorIcon::Pointer,
                        fret_core::CursorIcon::Text => winit::cursor::CursorIcon::Text,
                        fret_core::CursorIcon::ColResize => winit::cursor::CursorIcon::ColResize,
                        fret_core::CursorIcon::RowResize => winit::cursor::CursorIcon::RowResize,
                        fret_core::CursorIcon::NwseResize => winit::cursor::CursorIcon::NwseResize,
                        fret_core::CursorIcon::NeswResize => winit::cursor::CursorIcon::NeswResize,
                    };
                    window.set_cursor(Cursor::Icon(cursor));
                }
                Effect::ImageRegisterRgba8 {
                    window: target_window,
                    token,
                    width,
                    height,
                    bytes,
                    color_info,
                    alpha_mode,
                } => {
                    if target_window != self.app_window {
                        continue;
                    }

                    if width == 0 || height == 0 {
                        self.pending_events.push(Event::ImageRegisterFailed {
                            token,
                            message: format!("invalid image size: {width}x{height}"),
                        });
                        continue;
                    }

                    let expected_len = (width as usize)
                        .saturating_mul(height as usize)
                        .saturating_mul(4);
                    if bytes.len() != expected_len {
                        self.pending_events.push(Event::ImageRegisterFailed {
                            token,
                            message: format!(
                                "invalid rgba8 byte length: got {} expected {}",
                                bytes.len(),
                                expected_len
                            ),
                        });
                        continue;
                    }

                    let color_space = match color_info.encoding {
                        fret_core::ImageEncoding::Srgb => fret_render::ImageColorSpace::Srgb,
                        fret_core::ImageEncoding::Linear => fret_render::ImageColorSpace::Linear,
                    };

                    let uploaded = fret_render::upload_rgba8_image(
                        &gfx.ctx.device,
                        &gfx.ctx.queue,
                        (width, height),
                        &bytes,
                        color_space,
                    );

                    let view = uploaded
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let image = gfx.renderer.register_image(fret_render::ImageDescriptor {
                        view,
                        size: uploaded.size,
                        format: uploaded.format,
                        color_space: uploaded.color_space,
                        alpha_mode,
                    });
                    self.uploaded_images.insert(
                        image,
                        streaming_images::UploadedImageEntry {
                            uploaded,
                            stream_generation: 0,
                            alpha_mode,
                            nv12_planes: None,
                        },
                    );

                    self.pending_events.push(Event::ImageRegistered {
                        token,
                        image,
                        width,
                        height,
                    });
                    window.request_redraw();
                }
                Effect::ImageUpdateRgba8 {
                    window: target_window,
                    token,
                    image,
                    stream_generation,
                    width,
                    height,
                    update_rect_px,
                    bytes_per_row,
                    bytes,
                    color_info,
                    alpha_mode,
                } => {
                    self.apply_streaming_image_update_rgba8(
                        window,
                        gfx,
                        &mut stats,
                        target_window,
                        token,
                        image,
                        stream_generation,
                        width,
                        height,
                        update_rect_px,
                        bytes_per_row,
                        &bytes,
                        color_info,
                        alpha_mode,
                    );
                }
                Effect::ImageUpdateNv12 {
                    window: target_window,
                    token,
                    image,
                    stream_generation,
                    width,
                    height,
                    update_rect_px,
                    y_bytes_per_row,
                    y_plane,
                    uv_bytes_per_row,
                    uv_plane,
                    color_info,
                    alpha_mode: _,
                } => {
                    stats.yuv_conversions_attempted =
                        stats.yuv_conversions_attempted.saturating_add(1);
                    if self.try_apply_streaming_image_update_nv12_gpu(
                        window,
                        gfx,
                        &mut stats,
                        target_window,
                        token,
                        image,
                        stream_generation,
                        width,
                        height,
                        update_rect_px,
                        y_bytes_per_row,
                        &y_plane,
                        uv_bytes_per_row,
                        &uv_plane,
                        color_info,
                    ) {
                        continue;
                    }
                    let t0 = fret_core::time::Instant::now();
                    match super::super::yuv::nv12_to_rgba8_rect(
                        super::super::yuv::Nv12ToRgba8RectInput {
                            width,
                            height,
                            update_rect_px,
                            y_bytes_per_row,
                            y_plane: &y_plane,
                            uv_bytes_per_row,
                            uv_plane: &uv_plane,
                            range: color_info.range,
                            matrix: color_info.matrix,
                        },
                    ) {
                        Ok((rect, rgba)) => {
                            stats.yuv_conversions_applied =
                                stats.yuv_conversions_applied.saturating_add(1);
                            stats.yuv_convert_us = stats
                                .yuv_convert_us
                                .saturating_add(t0.elapsed().as_micros() as u64);
                            stats.yuv_convert_output_bytes = stats
                                .yuv_convert_output_bytes
                                .saturating_add(rgba.len() as u64);

                            self.apply_streaming_image_update_rgba8(
                                window,
                                gfx,
                                &mut stats,
                                target_window,
                                token,
                                image,
                                stream_generation,
                                width,
                                height,
                                Some(rect),
                                rect.w.saturating_mul(4),
                                &rgba,
                                fret_core::ImageColorInfo::srgb_rgba(),
                                fret_core::AlphaMode::Opaque,
                            );
                        }
                        Err(_message) => {
                            if self.config.streaming_update_ack_enabled {
                                self.pending_events.push(Event::ImageUpdateDropped {
                                    token,
                                    image,
                                    reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                                });
                            }
                        }
                    }
                }
                Effect::ImageUpdateI420 {
                    window: target_window,
                    token,
                    image,
                    stream_generation,
                    width,
                    height,
                    update_rect_px,
                    y_bytes_per_row,
                    y_plane,
                    u_bytes_per_row,
                    u_plane,
                    v_bytes_per_row,
                    v_plane,
                    color_info,
                    alpha_mode: _,
                } => {
                    stats.yuv_conversions_attempted =
                        stats.yuv_conversions_attempted.saturating_add(1);
                    let t0 = fret_core::time::Instant::now();
                    match super::super::yuv::i420_to_rgba8_rect(
                        super::super::yuv::I420ToRgba8RectInput {
                            width,
                            height,
                            update_rect_px,
                            y_bytes_per_row,
                            y_plane: &y_plane,
                            u_bytes_per_row,
                            u_plane: &u_plane,
                            v_bytes_per_row,
                            v_plane: &v_plane,
                            range: color_info.range,
                            matrix: color_info.matrix,
                        },
                    ) {
                        Ok((rect, rgba)) => {
                            stats.yuv_conversions_applied =
                                stats.yuv_conversions_applied.saturating_add(1);
                            stats.yuv_convert_us = stats
                                .yuv_convert_us
                                .saturating_add(t0.elapsed().as_micros() as u64);
                            stats.yuv_convert_output_bytes = stats
                                .yuv_convert_output_bytes
                                .saturating_add(rgba.len() as u64);

                            self.apply_streaming_image_update_rgba8(
                                window,
                                gfx,
                                &mut stats,
                                target_window,
                                token,
                                image,
                                stream_generation,
                                width,
                                height,
                                Some(rect),
                                rect.w.saturating_mul(4),
                                &rgba,
                                fret_core::ImageColorInfo::srgb_rgba(),
                                fret_core::AlphaMode::Opaque,
                            );
                        }
                        Err(_message) => {
                            if self.config.streaming_update_ack_enabled {
                                self.pending_events.push(Event::ImageUpdateDropped {
                                    token,
                                    image,
                                    reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                                });
                            }
                        }
                    }
                }
                Effect::ImageUnregister { image } => {
                    self.uploaded_images.remove(&image);
                    if gfx.renderer.unregister_image(image) {
                        window.request_redraw();
                    }
                }
                Effect::ViewportInput(event) => {
                    self.driver.viewport_input(&mut self.app, event);
                }
                Effect::Dock(op) => {
                    self.driver.dock_op(&mut self.app, op);
                }
                Effect::Window(req) => match req {
                    WindowRequest::Close(target) => {
                        if target != self.app_window {
                            continue;
                        }
                        self.exiting = true;
                        self.dispatcher.shutdown();
                        self.web_cursor.take();
                        event_loop.exit();
                        return true;
                    }
                    WindowRequest::Create(_)
                    | WindowRequest::Raise { .. }
                    | WindowRequest::SetInnerSize { .. } => {}
                },
                Effect::QuitApp => {
                    self.exiting = true;
                    self.dispatcher.shutdown();
                    self.web_cursor.take();
                    event_loop.exit();
                    return true;
                }
                Effect::HideApp | Effect::HideOtherApps | Effect::UnhideAllApps => {}
                Effect::Command { window, command } => match window {
                    Some(target) if target == self.app_window => {
                        self.driver.handle_command(
                            WinitCommandContext {
                                app: &mut self.app,
                                services: &mut gfx.renderer,
                                window: self.app_window,
                                state,
                            },
                            command,
                        );
                    }
                    None => {
                        self.driver.handle_global_command(
                            WinitGlobalContext {
                                app: &mut self.app,
                                services: &mut gfx.renderer,
                            },
                            command,
                        );
                    }
                    _ => {}
                },
                Effect::Redraw(target) | Effect::RequestAnimationFrame(target) => {
                    if target == self.app_window {
                        window.request_redraw();
                    }
                }
                _ => {}
            }
        }

        let streaming_snapshot_enabled = self.config.streaming_perf_snapshot_enabled
            || std::env::var_os("FRET_STREAMING_DEBUG").is_some_and(|v| !v.is_empty());
        let streaming_stats_have_activity = stats.update_effects_seen > 0
            || stats.update_effects_enqueued > 0
            || stats.update_effects_replaced > 0
            || stats.update_effects_applied > 0
            || stats.update_effects_delayed_budget > 0
            || stats.update_effects_dropped_staging > 0
            || stats.upload_bytes_budgeted > 0
            || stats.upload_bytes_applied > 0
            || stats.pending_updates > 0
            || stats.pending_staging_bytes > 0
            || stats.yuv_conversions_attempted > 0
            || stats.yuv_convert_us > 0;
        if streaming_snapshot_enabled && streaming_stats_have_activity {
            self.app.set_global(fret_core::StreamingUploadPerfSnapshot {
                frame_id: self.frame_id,
                upload_budget_bytes_per_frame: stats.upload_budget_bytes_per_frame,
                staging_budget_bytes: stats.staging_budget_bytes,
                update_effects_seen: u64::from(stats.update_effects_seen),
                update_effects_enqueued: u64::from(stats.update_effects_enqueued),
                update_effects_replaced: u64::from(stats.update_effects_replaced),
                update_effects_applied: u64::from(stats.update_effects_applied),
                update_effects_delayed_budget: u64::from(stats.update_effects_delayed_budget),
                update_effects_dropped_staging: u64::from(stats.update_effects_dropped_staging),
                upload_bytes_budgeted: stats.upload_bytes_budgeted,
                upload_bytes_applied: stats.upload_bytes_applied,
                pending_updates: u64::from(stats.pending_updates),
                pending_staging_bytes: stats.pending_staging_bytes,
                yuv_convert_us: stats.yuv_convert_us,
                yuv_convert_output_bytes: stats.yuv_convert_output_bytes,
                yuv_conversions_attempted: u64::from(stats.yuv_conversions_attempted),
                yuv_conversions_applied: u64::from(stats.yuv_conversions_applied),
            });
        }

        if std::env::var_os("FRET_STREAMING_DEBUG").is_some_and(|v| !v.is_empty())
            && (stats.update_effects_delayed_budget > 0
                || stats.update_effects_dropped_staging > 0
                || stats.update_effects_replaced > 0
                || stats.yuv_conversions_attempted > 0)
        {
            tracing::debug!(
                seen = stats.update_effects_seen,
                enqueued = stats.update_effects_enqueued,
                replaced = stats.update_effects_replaced,
                applied = stats.update_effects_applied,
                delayed_budget = stats.update_effects_delayed_budget,
                dropped_staging = stats.update_effects_dropped_staging,
                upload_bytes_budgeted = stats.upload_bytes_budgeted,
                upload_bytes_applied = stats.upload_bytes_applied,
                upload_budget_bytes_per_frame = stats.upload_budget_bytes_per_frame,
                staging_budget_bytes = stats.staging_budget_bytes,
                pending_updates = stats.pending_updates,
                pending_staging_bytes = stats.pending_staging_bytes,
                yuv_attempted = stats.yuv_conversions_attempted,
                yuv_applied = stats.yuv_conversions_applied,
                yuv_convert_us = stats.yuv_convert_us,
                yuv_output_bytes = stats.yuv_convert_output_bytes,
                "streaming image updates queued/budgeted"
            );
        }

        true
    }

    fn drain_inboxes(&mut self, window: Option<AppWindowId>) -> bool {
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
