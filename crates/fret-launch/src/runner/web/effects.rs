use fret_app::Effect;
use fret_core::Event;
use fret_runtime::{PlatformCapabilities, WindowRequest};
use js_sys::{Array, Function, Object, Promise, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{File, FilePropertyBag};
use winit::cursor::Cursor;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use super::super::streaming_upload::StreamingUploadAckKind;
use super::super::{WinitCommandContext, WinitGlobalContext};
use super::streaming_images;
use super::{GfxState, WinitAppDriver, WinitRunner};

fn share_items_to_web_share_data(
    items: &[fret_core::ShareItem],
) -> Result<Option<Object>, JsValue> {
    let mut text_parts: Vec<String> = Vec::new();
    let mut url: Option<String> = None;
    let files = Array::new();

    for item in items {
        match item {
            fret_core::ShareItem::Text(text) => {
                if !text.is_empty() {
                    text_parts.push(text.clone());
                }
            }
            fret_core::ShareItem::Url(u) => {
                if url.is_none() && !u.is_empty() {
                    url = Some(u.clone());
                } else if !u.is_empty() {
                    text_parts.push(u.clone());
                }
            }
            fret_core::ShareItem::Bytes { name, mime, bytes } => {
                let parts = Array::new();
                parts.push(&Uint8Array::from(bytes.as_slice()));
                let parts: JsValue = parts.into();
                let file = if let Some(mime) = mime.as_deref()
                    && !mime.is_empty()
                {
                    let opts = FilePropertyBag::new();
                    opts.set_type(mime);
                    File::new_with_u8_array_sequence_and_options(&parts, name, &opts)?
                } else {
                    File::new_with_u8_array_sequence(&parts, name)?
                };
                files.push(&file);
            }
        }
    }

    if text_parts.is_empty() && url.is_none() && files.length() == 0 {
        return Ok(None);
    }

    let data = Object::new();
    if !text_parts.is_empty() {
        let text = text_parts.join("\n");
        let _ = Reflect::set(
            data.as_ref(),
            &JsValue::from_str("text"),
            &JsValue::from_str(&text),
        );
    }
    if let Some(url) = url {
        let _ = Reflect::set(
            data.as_ref(),
            &JsValue::from_str("url"),
            &JsValue::from_str(&url),
        );
    }

    if files.length() != 0 {
        let _ = Reflect::set(data.as_ref(), &JsValue::from_str("files"), files.as_ref());
    }

    Ok(Some(data))
}

fn js_error_string(err: &JsValue) -> String {
    let name = Reflect::get(err, &JsValue::from_str("name"))
        .ok()
        .and_then(|v| v.as_string());
    let message = Reflect::get(err, &JsValue::from_str("message"))
        .ok()
        .and_then(|v| v.as_string());
    match (name, message) {
        (Some(name), Some(message)) if !message.is_empty() => format!("{name}: {message}"),
        (Some(name), _) => name,
        (_, Some(message)) if !message.is_empty() => message,
        _ => "navigator.share rejected".to_string(),
    }
}

fn share_outcome_from_error(err: JsValue) -> fret_core::ShareSheetOutcome {
    let name = Reflect::get(&err, &JsValue::from_str("name"))
        .ok()
        .and_then(|v| v.as_string());
    if name.as_deref() == Some("AbortError") {
        return fret_core::ShareSheetOutcome::Canceled;
    }
    if matches!(
        name.as_deref(),
        Some("TypeError") | Some("NotSupportedError")
    ) {
        return fret_core::ShareSheetOutcome::Unavailable;
    }
    fret_core::ShareSheetOutcome::Failed {
        message: js_error_string(&err),
    }
}

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
                            variable_axes: e
                                .variable_axes
                                .into_iter()
                                .map(|a| fret_runtime::FontVariableAxisInfo {
                                    tag: a.tag,
                                    min_bits: a.min_bits,
                                    max_bits: a.max_bits,
                                    default_bits: a.default_bits,
                                })
                                .collect(),
                            is_monospace_candidate: e.is_monospace_candidate,
                        })
                        .collect::<Vec<_>>();
                    // Font catalog refresh trigger (ADR 0258): `Effect::TextAddFonts`.
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
                Effect::TextRescanSystemFonts => {
                    // Web/WASM cannot access system fonts; ignore (ADR 0258).
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
                Effect::DiagClipboardForceUnavailable {
                    window: target_window,
                    enabled,
                } => {
                    if target_window != self.app_window {
                        continue;
                    }
                    self.diag_clipboard_force_unavailable = enabled;
                }
                Effect::DiagIncomingOpenInject {
                    window: target_window,
                    items,
                } => {
                    if target_window != self.app_window {
                        continue;
                    }
                    let token = fret_core::IncomingOpenToken(self.diag_incoming_open_next_token);
                    self.diag_incoming_open_next_token =
                        self.diag_incoming_open_next_token.saturating_add(1);

                    let mut payload = super::DiagIncomingOpenPayload::default();
                    let mut request_items: Vec<fret_core::IncomingOpenItem> = Vec::new();
                    for item in items {
                        match item {
                            fret_runtime::DiagIncomingOpenItem::File {
                                name,
                                bytes,
                                media_type,
                            } => {
                                request_items.push(fret_core::IncomingOpenItem::File(
                                    fret_core::ExternalDragFile {
                                        name: name.clone(),
                                        size_bytes: Some(bytes.len() as u64),
                                        media_type,
                                    },
                                ));
                                payload
                                    .files
                                    .push(fret_core::ExternalDropFileData { name, bytes });
                            }
                            fret_runtime::DiagIncomingOpenItem::Text { text, media_type } => {
                                let estimated_size_bytes = Some(text.len() as u64);
                                request_items.push(fret_core::IncomingOpenItem::Text {
                                    media_type,
                                    estimated_size_bytes,
                                });
                                payload.texts.push(text);
                            }
                        }
                    }
                    self.diag_incoming_open_payloads.insert(token, payload);
                    self.pending_events.push(Event::IncomingOpenRequest {
                        token,
                        items: request_items,
                    });
                    window.request_redraw();
                }
                Effect::ClipboardSetText { text: _ } => {
                    // Best-effort: clipboard access is platform-dependent on web and may be
                    // restricted. For now, treat as unsupported.
                }
                Effect::ClipboardGetText {
                    window: target_window,
                    token,
                } => {
                    if target_window != self.app_window {
                        continue;
                    }
                    self.pending_events
                        .push(Event::ClipboardTextUnavailable { token });
                    window.request_redraw();
                }
                Effect::PrimarySelectionSetText { text: _ } => {}
                Effect::PrimarySelectionGetText {
                    window: target_window,
                    token,
                } => {
                    if target_window != self.app_window {
                        continue;
                    }
                    self.pending_events
                        .push(Event::PrimarySelectionTextUnavailable { token });
                    window.request_redraw();
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
                Effect::ShareSheetShow {
                    window: target_window,
                    token,
                    items,
                } => {
                    if target_window != self.app_window {
                        continue;
                    }

                    let caps = self
                        .app
                        .global::<PlatformCapabilities>()
                        .cloned()
                        .unwrap_or_default();
                    if !caps.shell.share_sheet {
                        self.pending_events.push(Event::ShareSheetCompleted {
                            token,
                            outcome: fret_core::ShareSheetOutcome::Unavailable,
                        });
                        window.request_redraw();
                        continue;
                    }

                    let data = match share_items_to_web_share_data(&items) {
                        Ok(Some(v)) => v,
                        Ok(None) => {
                            self.pending_events.push(Event::ShareSheetCompleted {
                                token,
                                outcome: fret_core::ShareSheetOutcome::Unavailable,
                            });
                            window.request_redraw();
                            continue;
                        }
                        Err(err) => {
                            self.pending_events.push(Event::ShareSheetCompleted {
                                token,
                                outcome: share_outcome_from_error(err),
                            });
                            window.request_redraw();
                            continue;
                        }
                    };

                    let Some(web_window) = web_sys::window() else {
                        self.pending_events.push(Event::ShareSheetCompleted {
                            token,
                            outcome: fret_core::ShareSheetOutcome::Unavailable,
                        });
                        window.request_redraw();
                        continue;
                    };

                    let navigator =
                        match Reflect::get(web_window.as_ref(), &JsValue::from_str("navigator")) {
                            Ok(v) => v,
                            Err(_) => {
                                self.pending_events.push(Event::ShareSheetCompleted {
                                    token,
                                    outcome: fret_core::ShareSheetOutcome::Unavailable,
                                });
                                window.request_redraw();
                                continue;
                            }
                        };

                    let share_fn = match Reflect::get(&navigator, &JsValue::from_str("share"))
                        .ok()
                        .and_then(|v| v.dyn_into::<Function>().ok())
                    {
                        Some(v) => v,
                        None => {
                            self.pending_events.push(Event::ShareSheetCompleted {
                                token,
                                outcome: fret_core::ShareSheetOutcome::Unavailable,
                            });
                            window.request_redraw();
                            continue;
                        }
                    };

                    // Best-effort: if `navigator.canShare` exists, consult it for the computed payload.
                    if let Some(can_share_fn) =
                        Reflect::get(&navigator, &JsValue::from_str("canShare"))
                            .ok()
                            .and_then(|v| v.dyn_into::<Function>().ok())
                    {
                        if let Ok(v) = can_share_fn.call1(&navigator, data.as_ref())
                            && v.as_bool() == Some(false)
                        {
                            self.pending_events.push(Event::ShareSheetCompleted {
                                token,
                                outcome: fret_core::ShareSheetOutcome::Unavailable,
                            });
                            window.request_redraw();
                            continue;
                        }
                    }

                    // Important: call `navigator.share(...)` synchronously while draining effects so the
                    // invocation can inherit a browser user-activation gesture when applicable.
                    let promise = match share_fn.call1(&navigator, data.as_ref()) {
                        Ok(v) => match v.dyn_into::<Promise>() {
                            Ok(p) => p,
                            Err(_) => {
                                self.pending_events.push(Event::ShareSheetCompleted {
                                    token,
                                    outcome: fret_core::ShareSheetOutcome::Failed {
                                        message: "navigator.share did not return a Promise"
                                            .to_string(),
                                    },
                                });
                                window.request_redraw();
                                continue;
                            }
                        },
                        Err(err) => {
                            self.pending_events.push(Event::ShareSheetCompleted {
                                token,
                                outcome: share_outcome_from_error(err),
                            });
                            window.request_redraw();
                            continue;
                        }
                    };

                    let pending = self.pending_async_events.clone();
                    let proxy = self.event_loop_proxy.clone();
                    spawn_local(async move {
                        let outcome = match JsFuture::from(promise).await {
                            Ok(_) => fret_core::ShareSheetOutcome::Shared,
                            Err(err) => share_outcome_from_error(err),
                        };
                        pending
                            .borrow_mut()
                            .push(Event::ShareSheetCompleted { token, outcome });
                        if let Some(proxy) = proxy {
                            proxy.wake_up();
                        }
                    });
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
                Effect::IncomingOpenReadAll {
                    window: target_window,
                    token,
                } => {
                    if target_window != self.app_window {
                        continue;
                    }

                    let Some(payload) = self.diag_incoming_open_payloads.get(&token) else {
                        self.pending_events
                            .push(Event::IncomingOpenUnavailable { token });
                        window.request_redraw();
                        continue;
                    };

                    let cap = fret_core::ExternalDropReadLimits {
                        max_total_bytes: self.config.file_dialog_max_total_bytes,
                        max_file_bytes: self.config.file_dialog_max_file_bytes,
                        max_files: self.config.file_dialog_max_files,
                    };

                    let limits = cap;
                    let include_limits = false;

                    let mut total_bytes: u64 = 0;
                    let mut files: Vec<fret_core::ExternalDropFileData> = Vec::new();
                    let mut texts: Vec<String> = Vec::new();
                    let mut errors: Vec<fret_core::ExternalDropReadError> = Vec::new();

                    for file in payload.files.iter().take(limits.max_files) {
                        let len = file.bytes.len() as u64;
                        if len > limits.max_file_bytes {
                            errors.push(fret_core::ExternalDropReadError {
                                name: file.name.clone(),
                                message: "file exceeds max_file_bytes".to_string(),
                            });
                            continue;
                        }
                        if total_bytes.saturating_add(len) > limits.max_total_bytes {
                            errors.push(fret_core::ExternalDropReadError {
                                name: file.name.clone(),
                                message: "read exceeds max_total_bytes".to_string(),
                            });
                            continue;
                        }
                        total_bytes = total_bytes.saturating_add(len);
                        files.push(file.clone());
                    }

                    for (idx, text) in payload.texts.iter().enumerate() {
                        let len = text.len() as u64;
                        let name = format!("text[{idx}]");
                        if len > limits.max_file_bytes {
                            errors.push(fret_core::ExternalDropReadError {
                                name,
                                message: "text exceeds max_file_bytes".to_string(),
                            });
                            continue;
                        }
                        if total_bytes.saturating_add(len) > limits.max_total_bytes {
                            errors.push(fret_core::ExternalDropReadError {
                                name,
                                message: "read exceeds max_total_bytes".to_string(),
                            });
                            continue;
                        }
                        total_bytes = total_bytes.saturating_add(len);
                        texts.push(text.clone());
                    }

                    self.pending_events.push(Event::IncomingOpenData(
                        fret_core::IncomingOpenDataEvent {
                            token,
                            files,
                            texts,
                            errors,
                            limits: include_limits.then_some(limits),
                        },
                    ));
                    window.request_redraw();
                }
                Effect::IncomingOpenReadAllWithLimits {
                    window: target_window,
                    token,
                    limits,
                } => {
                    if target_window != self.app_window {
                        continue;
                    }

                    let Some(payload) = self.diag_incoming_open_payloads.get(&token) else {
                        self.pending_events
                            .push(Event::IncomingOpenUnavailable { token });
                        window.request_redraw();
                        continue;
                    };

                    let cap = fret_core::ExternalDropReadLimits {
                        max_total_bytes: self.config.file_dialog_max_total_bytes,
                        max_file_bytes: self.config.file_dialog_max_file_bytes,
                        max_files: self.config.file_dialog_max_files,
                    };

                    let limits = limits.capped_by(cap);
                    let include_limits = true;

                    let mut total_bytes: u64 = 0;
                    let mut files: Vec<fret_core::ExternalDropFileData> = Vec::new();
                    let mut texts: Vec<String> = Vec::new();
                    let mut errors: Vec<fret_core::ExternalDropReadError> = Vec::new();

                    for file in payload.files.iter().take(limits.max_files) {
                        let len = file.bytes.len() as u64;
                        if len > limits.max_file_bytes {
                            errors.push(fret_core::ExternalDropReadError {
                                name: file.name.clone(),
                                message: "file exceeds max_file_bytes".to_string(),
                            });
                            continue;
                        }
                        if total_bytes.saturating_add(len) > limits.max_total_bytes {
                            errors.push(fret_core::ExternalDropReadError {
                                name: file.name.clone(),
                                message: "read exceeds max_total_bytes".to_string(),
                            });
                            continue;
                        }
                        total_bytes = total_bytes.saturating_add(len);
                        files.push(file.clone());
                    }

                    for (idx, text) in payload.texts.iter().enumerate() {
                        let len = text.len() as u64;
                        let name = format!("text[{idx}]");
                        if len > limits.max_file_bytes {
                            errors.push(fret_core::ExternalDropReadError {
                                name,
                                message: "text exceeds max_file_bytes".to_string(),
                            });
                            continue;
                        }
                        if total_bytes.saturating_add(len) > limits.max_total_bytes {
                            errors.push(fret_core::ExternalDropReadError {
                                name,
                                message: "read exceeds max_total_bytes".to_string(),
                            });
                            continue;
                        }
                        total_bytes = total_bytes.saturating_add(len);
                        texts.push(text.clone());
                    }

                    self.pending_events.push(Event::IncomingOpenData(
                        fret_core::IncomingOpenDataEvent {
                            token,
                            files,
                            texts,
                            errors,
                            limits: include_limits.then_some(limits),
                        },
                    ));
                    window.request_redraw();
                }
                Effect::IncomingOpenRelease { token } => {
                    self.diag_incoming_open_payloads.remove(&token);
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
