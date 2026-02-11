use fret_app::Effect;
use fret_core::Event;
use fret_runtime::WindowRequest;
use winit::cursor::Cursor;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use super::super::streaming_upload::StreamingUploadAckKind;
use super::super::{WinitCommandContext, WinitGlobalContext};
use super::streaming_images;
use super::{GfxState, WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn drain_effects(
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

                    let entries = gfx
                        .renderer
                        .all_font_catalog_entries()
                        .into_iter()
                        .map(|e| fret_runtime::FontCatalogEntry {
                            family: e.family,
                            has_variable_axes: e.has_variable_axes,
                            known_variable_axes: e.known_variable_axes,
                            is_monospace_candidate: e.is_monospace_candidate,
                        })
                        .collect::<Vec<_>>();
                    let update = fret_runtime::apply_font_catalog_update_with_metadata(
                        &mut self.app,
                        entries,
                        fret_runtime::FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates,
                    );
                    let _ = gfx.renderer.set_text_font_families(&update.config);
                    let locale = self
                        .app
                        .global::<fret_runtime::fret_i18n::I18nService>()
                        .and_then(|service| service.preferred_locales().first())
                        .map(|locale| locale.to_string());
                    let _ = gfx.renderer.set_text_locale(locale.as_deref());
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
                Effect::WindowMetricsSetInsets {
                    window: target_window,
                    safe_area_insets,
                    occlusion_insets,
                } => {
                    if target_window != self.app_window {
                        continue;
                    }
                    let mut changed = false;
                    self.app.with_global_mut(
                        fret_core::WindowMetricsService::default,
                        |svc, _app| {
                            if let Some(value) = safe_area_insets {
                                let current = svc.safe_area_insets(target_window);
                                let current_known = svc.safe_area_insets_is_known(target_window);
                                let needs_set = if value.is_none() {
                                    !current_known || current.is_some()
                                } else {
                                    !current_known || current != value
                                };
                                if needs_set {
                                    svc.set_safe_area_insets(target_window, value);
                                    changed = true;
                                }
                            }
                            if let Some(value) = occlusion_insets {
                                let current = svc.occlusion_insets(target_window);
                                let current_known = svc.occlusion_insets_is_known(target_window);
                                let needs_set = if value.is_none() {
                                    !current_known || current.is_some()
                                } else {
                                    !current_known || current != value
                                };
                                if needs_set {
                                    svc.set_occlusion_insets(target_window, value);
                                    changed = true;
                                }
                            }
                        },
                    );
                    if changed {
                        window.request_redraw();
                    }
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
}
