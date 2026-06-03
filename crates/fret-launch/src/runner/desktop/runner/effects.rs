use std::{any::TypeId, collections::HashSet};

use super::macos_cursor::dock_tearoff_log;
use fret_app::Effect;
use fret_core::Event;
use fret_core::time::Instant;
use winit::event_loop::ActiveEventLoop;

use super::{WinitCommandContext, WinitGlobalContext, WinitRunner, WinitWindowContext};

impl<D: super::WinitAppDriver> WinitRunner<D> {
    pub(super) fn drain_inboxes(&mut self, window: Option<fret_core::AppWindowId>) -> bool {
        let did_work = self.app.with_global_mut_untracked(
            fret_runtime::InboxDrainRegistry::default,
            |registry, app| registry.drain_all(app, window),
        );
        tracing::trace!(?window, did_work, "driver: drain_inboxes");
        did_work
    }

    pub(super) fn drain_effects(&mut self, event_loop: &dyn ActiveEventLoop) {
        let mut should_exit = false;
        crate::runner::common::fixed_point::drain_bounded(|| {
            let now = Instant::now();
            let mut did_work = self.dispatcher.drain_turn(now);
            did_work |= self.drain_inboxes(None);
            did_work |= self.apply_pending_system_font_rescan_result(now);
            let effects = self.app.flush_effects();
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
                    let window = ack
                        .window_hint
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    let Some(window) = window else {
                        continue;
                    };
                    match ack.kind {
                        crate::runner::streaming_upload::StreamingUploadAckKind::Dropped(
                            reason,
                        ) => {
                            self.deliver_window_event_now(
                                window,
                                &Event::ImageUpdateDropped {
                                    token: ack.token,
                                    image: ack.image,
                                    reason,
                                },
                            );
                        }
                    }
                }
            }

            did_work |= self.poll_watch_restart_trigger(now);
            did_work |= self.poll_hotpatch_trigger(now);
            did_work |= !effects.is_empty();
            let mut window_state_dirty: HashSet<fret_core::AppWindowId> = HashSet::new();

            for effect in effects {
                match effect {
                    Effect::Redraw(window) => {
                        if self.request_window_redraw_with_reason(
                            window,
                            fret_runtime::RunnerFrameDriveReason::EffectRedraw,
                        ) {
                            // Some platforms may not wake the event loop for `request_redraw()`
                            // alone; scheduling a one-shot RAF ensures the first frame presents
                            // without requiring any input events.
                            self.raf_windows.request(window);
                        }
                    }
                    Effect::ImeAllow { window, enabled } => {
                        if let Some(state) = self.windows.get_mut(window)
                            && state.platform.set_ime_allowed(enabled)
                        {
                            #[cfg(target_os = "android")]
                            self.android_force_soft_input(enabled);
                            window_state_dirty.insert(window);
                        }
                    }
                    Effect::ImeRequestVirtualKeyboard { window, visible } => {
                        #[cfg(target_os = "android")]
                        {
                            let _ = window;
                            self.android_force_soft_input(visible);
                        }
                        #[cfg(not(target_os = "android"))]
                        {
                            let _ = (window, visible);
                        }
                    }
                    Effect::ImeSetCursorArea { window, rect } => {
                        if let Some(state) = self.windows.get_mut(window) {
                            if std::env::var_os("FRET_IME_DEBUG").is_some_and(|v| !v.is_empty()) {
                                tracing::info!(
                                    "IME_DEBUG effect: ImeSetCursorArea window={:?} rect=({:.1},{:.1} {:.1}x{:.1})",
                                    window,
                                    rect.origin.x.0,
                                    rect.origin.y.0,
                                    rect.size.width.0,
                                    rect.size.height.0
                                );
                            }
                            if state.platform.set_ime_cursor_area(rect) {
                                window_state_dirty.insert(window);
                            }
                        }
                    }
                    Effect::WindowMetricsSetInsets {
                        window,
                        safe_area_insets,
                        occlusion_insets,
                    } => {
                        self.apply_window_metrics_insets_request(
                            window,
                            safe_area_insets,
                            occlusion_insets,
                        );
                    }
                    Effect::WindowMetricsSetPreferences {
                        window,
                        color_scheme,
                        prefers_reduced_motion,
                        text_scale_factor,
                    } => {
                        self.apply_window_metrics_preferences_request(
                            window,
                            color_scheme,
                            prefers_reduced_motion,
                            text_scale_factor,
                        );
                    }
                    Effect::CursorSetIcon { window, icon } => {
                        let Some(state) = self.windows.get_mut(window) else {
                            continue;
                        };
                        if state.platform.set_cursor_icon(icon) {
                            window_state_dirty.insert(window);
                        }
                    }
                    Effect::RequestAnimationFrame(window) => {
                        self.raf_windows.request(window);
                        if self.windows.contains_key(window) {
                            self.record_frame_drive_reason(
                                window,
                                fret_runtime::RunnerFrameDriveReason::EffectRequestAnimationFrame,
                            );
                        }
                    }
                    Effect::DiagInjectEvent { window, event } => {
                        fret_runtime::with_injected_event_scope(|| {
                            self.deliver_window_event_now(window, &event);
                        });
                        if self.windows.contains_key(window) {
                            let _ = self.request_window_redraw_with_reason(
                                window,
                                fret_runtime::RunnerFrameDriveReason::EffectRedraw,
                            );
                            self.raf_windows.request(window);
                        }
                    }
                    Effect::SetTimer { .. } => {
                        self.schedule_timer(now, &effect);
                    }
                    Effect::CancelTimer { token } => {
                        self.timers.remove(&token);
                    }
                    Effect::QuitApp => {
                        let prompt_window = self.main_window.or_else(|| self.windows.keys().next());
                        if let Some(window) = prompt_window
                            && !self.driver.before_close_window(&mut self.app, window)
                        {
                            continue;
                        }

                        #[cfg(feature = "dev-state")]
                        if self.dev_state.enabled() {
                            let alive: std::collections::HashSet<fret_core::AppWindowId> =
                                self.windows.keys().collect();
                            self.dev_state
                                .sync_window_keys_from_app(&self.app, |window| {
                                    alive.contains(&window)
                                });

                            let keys = self.dev_state.window_keys_snapshot();
                            for (window, key) in keys {
                                let Some(state) = self.windows.get(window) else {
                                    continue;
                                };
                                let physical = state.window.surface_size();
                                let logical: winit::dpi::LogicalSize<f64> =
                                    physical.to_logical(state.window.scale_factor());
                                let position = state.window.outer_position().ok();
                                self.dev_state
                                    .observe_window_geometry_now(&key, logical, position);
                            }
                            self.dev_state.export_and_flush_now(&mut self.app);
                        }

                        let windows: Vec<fret_core::AppWindowId> = self.windows.keys().collect();
                        for window in windows {
                            let _ = self.force_close_window(window);
                        }

                        self.dispatcher.shutdown();
                        event_loop.exit();
                        should_exit = true;
                        return false;
                    }
                    Effect::ShowAboutPanel => {
                        self.handle_show_about_panel();
                    }
                    Effect::HideApp => {
                        self.handle_hide_app();
                    }
                    Effect::HideOtherApps => {
                        self.handle_hide_other_apps();
                    }
                    Effect::UnhideAllApps => {
                        self.handle_unhide_all_apps();
                    }
                    Effect::Command { window, command } => match window {
                        Some(window) => {
                            if let Some(state) = self.windows.get_mut(window) {
                                let services = Self::ui_services_mut(
                                    &mut self.renderer,
                                    &mut self.no_services,
                                );
                                self.driver.handle_command(
                                    WinitCommandContext {
                                        app: &mut self.app,
                                        services,
                                        window,
                                        state: &mut state.user,
                                    },
                                    command,
                                );
                            }
                        }
                        None => {
                            let services =
                                Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                            self.driver.handle_global_command(
                                WinitGlobalContext {
                                    app: &mut self.app,
                                    services,
                                },
                                command,
                            );
                        }
                    },
                    Effect::SetMenuBar { window, menu_bar } => {
                        if window.is_none() {
                            self.menu_bar = Some(menu_bar.clone());
                        }
                        #[cfg(windows)]
                        {
                            let targets: Vec<fret_core::AppWindowId> = match window {
                                Some(window) => vec![window],
                                None => self.windows.keys().collect(),
                            };
                            for window in targets {
                                let Some(state) = self.windows.get_mut(window) else {
                                    continue;
                                };
                                let Some(menu) = super::windows_menu::set_window_menu_bar(
                                    &self.app,
                                    state.window.as_ref(),
                                    window,
                                    &menu_bar,
                                ) else {
                                    continue;
                                };
                                state.os_menu = Some(menu);
                            }
                        }
                        #[cfg(target_os = "macos")]
                        {
                            let _ = window;
                            super::macos_menu::set_app_menu_bar(&self.app, &menu_bar);
                        }
                        #[cfg(all(not(windows), not(target_os = "macos")))]
                        {
                            let _ = (window, menu_bar);
                        }
                    }
                    Effect::DiagClipboardForceUnavailable { window, enabled } => {
                        self.apply_diag_clipboard_force_unavailable(window, enabled);
                    }
                    Effect::DiagIncomingOpenInject { window, items } => {
                        self.handle_diag_incoming_open_inject(window, items);
                    }
                    Effect::ClipboardWriteText {
                        window,
                        token,
                        text,
                    } => {
                        self.handle_clipboard_write_text(window, token, text);
                    }
                    Effect::ClipboardReadText { window, token } => {
                        self.handle_clipboard_read_text(window, token);
                    }
                    Effect::PrimarySelectionSetText { text } => {
                        self.handle_primary_selection_set_text(text);
                    }
                    Effect::PrimarySelectionGetText { window, token } => {
                        self.handle_primary_selection_get_text(window, token);
                    }
                    Effect::ExternalDropReadAll { window, token } => {
                        self.handle_external_drop_read_all(window, token);
                    }
                    Effect::ExternalDropReadAllWithLimits {
                        window,
                        token,
                        limits,
                    } => {
                        self.handle_external_drop_read_all_with_limits(window, token, limits);
                    }
                    Effect::ExternalDropRelease { token } => {
                        self.handle_external_drop_release(token);
                    }
                    Effect::OpenUrl { url, .. } => {
                        self.handle_open_url(url);
                    }
                    Effect::ShareSheetShow {
                        window,
                        token,
                        items,
                    } => {
                        self.handle_share_sheet_show(window, token, items);
                    }
                    Effect::FileDialogOpen { window, options } => {
                        self.handle_file_dialog_open(window, options);
                    }
                    Effect::FileDialogReadAll { window, token } => {
                        self.handle_file_dialog_read_all(window, token);
                    }
                    Effect::FileDialogReadAllWithLimits {
                        window,
                        token,
                        limits,
                    } => {
                        self.handle_file_dialog_read_all_with_limits(window, token, limits);
                    }
                    Effect::FileDialogRelease { token } => {
                        self.handle_file_dialog_release(token);
                    }
                    Effect::IncomingOpenReadAll { window, token } => {
                        self.handle_incoming_open_read_all(window, token);
                    }
                    Effect::IncomingOpenReadAllWithLimits {
                        window,
                        token,
                        limits,
                    } => {
                        self.handle_incoming_open_read_all_with_limits(window, token, limits);
                    }
                    Effect::IncomingOpenRelease { token } => {
                        self.handle_incoming_open_release(token);
                    }
                    Effect::TextAddFontAssets { requests } => {
                        self.handle_text_add_font_assets(requests);
                    }
                    Effect::TextRescanSystemFonts => {
                        self.handle_text_rescan_system_fonts();
                    }
                    Effect::ImageRegisterRgba8 {
                        window,
                        token,
                        width,
                        height,
                        bytes,
                        color_info,
                        alpha_mode,
                    } => {
                        self.handle_image_register_rgba8(
                            window, token, width, height, bytes, color_info, alpha_mode,
                        );
                    }
                    Effect::ImageUpdateRgba8 {
                        window,
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
                        self.handle_image_update_rgba8(
                            &mut stats,
                            window,
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
                        );
                    }
                    Effect::ImageUpdateNv12 {
                        window,
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
                        self.handle_image_update_nv12(
                            &mut stats,
                            window,
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
                        );
                    }
                    Effect::ImageUpdateI420 {
                        window,
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
                        self.handle_image_update_i420(
                            &mut stats,
                            window,
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
                        );
                    }
                    Effect::ImageUnregister { image } => {
                        self.handle_image_unregister(image);
                    }
                    Effect::ViewportInput(event) => {
                        self.driver.viewport_input(&mut self.app, event);
                    }
                    Effect::Dock(op) => {
                        if matches!(op, fret_core::DockOp::RequestFloatPanelToNewWindow { .. }) {
                            dock_tearoff_log(format_args!("[effect-dock] {:?}", op));
                        }
                        self.driver.dock_op(&mut self.app, op);
                    }
                    Effect::Window(req) => {
                        if self.handle_window_request_effect(event_loop, req, now) {
                            should_exit = true;
                            return false;
                        }
                    }
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

            for window in window_state_dirty {
                if let Some(state) = self.windows.get_mut(window) {
                    state.platform.prepare_frame(state.window.as_ref());
                }
            }

            did_work |= self.fire_due_timers(now);
            did_work |= self.clear_internal_drag_hover_if_needed();
            did_work |= self.propagate_model_changes();
            did_work |= self.propagate_global_changes();

            if self.streaming_uploads.has_pending() {
                self.request_streaming_pending_redraws();
            }

            if !did_work {
                return false;
            }
            true
        });
        if should_exit {}
    }

    pub(super) fn propagate_model_changes(&mut self) -> bool {
        let changed = self.app.take_changed_models();
        if changed.is_empty() {
            return false;
        }

        for (window, runtime) in self.windows.iter_mut() {
            self.driver.handle_model_changes(
                WinitWindowContext {
                    app: &mut self.app,
                    window,
                    state: &mut runtime.user,
                },
                &changed,
            );
        }
        true
    }

    pub(super) fn propagate_global_changes(&mut self) -> bool {
        let changed = self.app.take_changed_globals();
        if changed.is_empty() {
            return false;
        }

        #[cfg(windows)]
        {
            if changed.contains(&TypeId::of::<fret_runtime::KeymapService>()) {
                super::windows_menu::sync_keymap_from_app(&self.app);
            }
            if changed.contains(&TypeId::of::<fret_runtime::WindowInputContextService>())
                || changed.contains(&TypeId::of::<fret_runtime::WindowCommandEnabledService>())
                || changed.contains(&TypeId::of::<
                    fret_runtime::WindowCommandActionAvailabilityService,
                >())
                || changed.contains(&TypeId::of::<fret_runtime::WindowCommandGatingService>())
            {
                super::windows_menu::sync_command_gating_from_app(&self.app);
            }
        }

        #[cfg(target_os = "macos")]
        {
            let keymap_changed = changed.contains(&TypeId::of::<fret_runtime::KeymapService>());
            if keymap_changed {
                super::macos_menu::sync_keymap_from_app(&self.app);
            }
            if changed.contains(&TypeId::of::<fret_runtime::WindowInputContextService>())
                || changed.contains(&TypeId::of::<fret_runtime::WindowCommandEnabledService>())
                || changed.contains(&TypeId::of::<
                    fret_runtime::WindowCommandActionAvailabilityService,
                >())
                || changed.contains(&TypeId::of::<fret_runtime::WindowCommandGatingService>())
            {
                super::macos_menu::sync_command_gating_from_app(&self.app);
            }
            if keymap_changed && let Some(menu_bar) = self.menu_bar.clone() {
                super::macos_menu::set_app_menu_bar(&self.app, &menu_bar);
            }
        }

        if changed.contains(&TypeId::of::<fret_core::TextFontFamilyConfig>())
            && let Some(renderer) = self.renderer.as_mut()
            && crate::runner::font_catalog::sync_renderer_font_families_from_globals(
                &mut self.app,
                renderer,
            )
        {
            for (_id, state) in self.windows.iter() {
                state.window.request_redraw();
            }
        }

        if changed.contains(&TypeId::of::<fret_runtime::fret_i18n::I18nService>())
            && let Some(renderer) = self.renderer.as_mut()
            && crate::runner::font_catalog::sync_renderer_locale_from_globals(
                &mut self.app,
                renderer,
            )
        {
            for (_id, state) in self.windows.iter() {
                state.window.request_redraw();
            }
        }

        for (window, runtime) in self.windows.iter_mut() {
            self.driver.handle_global_changes(
                WinitWindowContext {
                    app: &mut self.app,
                    window,
                    state: &mut runtime.user,
                },
                &changed,
            );
        }
        true
    }
}
