use std::{any::TypeId, collections::HashSet, sync::OnceLock};

use super::macos_cursor::dock_tearoff_log;
use super::streaming_images::{
    StreamingImageUpdateNv12, StreamingImageUpdateRgba8, UploadedImageEntry,
};
use super::window::bring_window_to_front;
use fret_app::{CreateWindowKind, Effect, WindowRequest};
use fret_core::time::Instant;
use fret_core::{Event, Point, Px};
use fret_platform::clipboard::Clipboard as _;
use fret_platform::external_drop::ExternalDropProvider as _;
use fret_platform::file_dialog::FileDialogProvider as _;
use fret_platform::open_url::OpenUrl as _;
use fret_platform_native::external_drop::NativeExternalDrop;
use fret_platform_native::file_dialog::NativeFileDialog;
use fret_runtime::{PlatformCapabilities, PlatformCompletion};
use tracing::error;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowLevel;

use super::{WinitCommandContext, WinitGlobalContext, WinitRunner, WinitWindowContext};

impl<D: super::WinitAppDriver> WinitRunner<D> {
    pub(super) fn system_font_rescan_async_enabled() -> bool {
        static FLAG: OnceLock<bool> = OnceLock::new();
        *FLAG.get_or_init(|| {
            if cfg!(any(target_os = "ios", target_os = "android")) {
                return false;
            }
            std::env::var("FRET_TEXT_SYSTEM_FONT_RESCAN_ASYNC")
                .ok()
                .is_some_and(|v| !v.trim().is_empty() && v.trim() != "0")
                || std::env::var_os("FRET_TEXT_SYSTEM_FONT_RESCAN_ASYNC").is_none()
        })
    }

    pub(super) fn system_font_catalog_startup_async_enabled() -> bool {
        static FLAG: OnceLock<bool> = OnceLock::new();
        *FLAG.get_or_init(|| {
            if cfg!(any(target_os = "ios", target_os = "android")) {
                return false;
            }
            std::env::var("FRET_TEXT_SYSTEM_FONT_CATALOG_STARTUP_ASYNC")
                .ok()
                .is_some_and(|v| !v.trim().is_empty() && v.trim() != "0")
                || std::env::var_os("FRET_TEXT_SYSTEM_FONT_CATALOG_STARTUP_ASYNC").is_none()
        })
    }

    pub(super) fn request_redraw_all_windows(&self) {
        for (_id, state) in self.windows.iter() {
            state.window.request_redraw();
        }
    }

    pub(super) fn publish_system_font_rescan_state(&mut self) {
        self.app.set_global::<fret_runtime::SystemFontRescanState>(
            fret_runtime::SystemFontRescanState {
                in_flight: self.system_font_rescan_in_flight,
                pending: self.system_font_rescan_pending,
            },
        );
    }

    pub(super) fn request_system_font_rescan(&mut self) {
        if !Self::system_font_rescan_async_enabled() {
            self.rescan_system_fonts_sync();
            return;
        }

        if self.system_font_rescan_in_flight {
            self.system_font_rescan_pending = true;
            self.publish_system_font_rescan_state();
            return;
        }

        let Some(seed) = self
            .renderer
            .as_mut()
            .and_then(|renderer| renderer.system_font_rescan_seed())
        else {
            return;
        };

        if let Ok(mut slot) = self.system_font_rescan_result.lock() {
            *slot = None;
        }
        self.system_font_rescan_in_flight = true;
        self.publish_system_font_rescan_state();

        let result_slot = self.system_font_rescan_result.clone();
        let dispatcher = self.dispatcher.handle();
        let dispatcher_for_wake = dispatcher.clone();
        dispatcher.dispatch_background(
            Box::new(move || {
                let result = seed.run();
                if let Ok(mut slot) = result_slot.lock() {
                    *slot = Some(result);
                }
                dispatcher_for_wake.wake(None);
            }),
            fret_runtime::DispatchPriority::Low,
        );
    }

    pub(super) fn rescan_system_fonts_sync(&mut self) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        if !renderer.rescan_system_fonts() {
            return;
        }

        // Font catalog refresh trigger (ADR 0258): explicit system font rescan.
        crate::runner::font_catalog::apply_renderer_font_catalog_update(
            &mut self.app,
            renderer,
            fret_runtime::FontFamilyDefaultsPolicy::None,
        );
        self.request_redraw_all_windows();
    }

    fn observe_window_surface_sizes(&mut self, now: Instant) {
        let mut any_changed = false;
        for (id, state) in self.windows.iter() {
            let size = state.window.surface_size();
            let entry = self
                .last_window_surface_sizes
                .entry(id)
                .or_insert((size.width, size.height));
            if *entry != (size.width, size.height) {
                *entry = (size.width, size.height);
                any_changed = true;
            }
        }
        if any_changed {
            self.last_window_surface_size_changed_at = Some(now);
        }
    }

    fn should_defer_system_font_rescan_apply(&self, now: Instant) -> bool {
        let Some(changed_at) = self.last_window_surface_size_changed_at else {
            return false;
        };
        // Give resize-driven layout a brief window to settle before applying the font update.
        // This is intentionally long enough to cover a few slow frames during interactive resize,
        // so a completed rescan is less likely to land inside a measured perf window.
        now < changed_at + std::time::Duration::from_millis(200)
    }

    pub(super) fn apply_pending_system_font_rescan_result(&mut self, now: Instant) -> bool {
        // Avoid applying a completed system font rescan while the user is actively resizing the
        // window. Applying the rescan bumps `TextFontStackKey` and can trigger large relayouts.
        self.observe_window_surface_sizes(now);
        if self.should_defer_system_font_rescan_apply(now) {
            return false;
        }

        let result = self
            .system_font_rescan_result
            .lock()
            .ok()
            .and_then(|mut slot| slot.take());
        let Some(result) = result else {
            return false;
        };

        self.system_font_rescan_in_flight = false;

        let Some(renderer) = self.renderer.as_mut() else {
            return true;
        };

        if !renderer.apply_system_font_rescan_result(result) {
            return true;
        }

        // Font catalog refresh trigger (ADR 0258): explicit system font rescan (async).
        crate::runner::font_catalog::apply_renderer_font_catalog_update(
            &mut self.app,
            renderer,
            fret_runtime::FontFamilyDefaultsPolicy::None,
        );
        self.request_redraw_all_windows();

        let should_restart = self.system_font_rescan_pending;
        self.system_font_rescan_pending = false;
        self.publish_system_font_rescan_state();
        if should_restart {
            self.request_system_font_rescan();
        }

        true
    }

    pub(super) fn drain_inboxes(&mut self, window: Option<fret_core::AppWindowId>) -> bool {
        let did_work = self.app.with_global_mut_untracked(
            fret_runtime::InboxDrainRegistry::default,
            |registry, app| registry.drain_all(app, window),
        );
        tracing::trace!(?window, did_work, "driver: drain_inboxes");
        did_work
    }

    fn incoming_open_limits_cap(&self) -> fret_core::ExternalDropReadLimits {
        fret_core::ExternalDropReadLimits {
            max_total_bytes: self.config.file_dialog_max_total_bytes,
            max_file_bytes: self.config.file_dialog_max_file_bytes,
            max_files: self.config.file_dialog_max_files,
        }
    }

    fn build_incoming_open_data_event(
        &self,
        token: fret_core::IncomingOpenToken,
        limits: fret_core::ExternalDropReadLimits,
        include_limits: bool,
    ) -> Option<fret_core::IncomingOpenDataEvent> {
        if let Some(payload) = self.diag_incoming_open_payloads.get(&token) {
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

            return Some(fret_core::IncomingOpenDataEvent {
                token,
                files,
                texts,
                errors,
                limits: include_limits.then_some(limits),
            });
        }

        let payload = self.incoming_open_path_payloads.get(&token)?;

        let mut files: Vec<fret_core::ExternalDropFileData> = Vec::new();
        let mut errors: Vec<fret_core::ExternalDropReadError> = Vec::new();
        let mut total: u64 = 0;

        for path in payload.paths.iter().take(limits.max_files) {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());

            let meta_len = match std::fs::metadata(path) {
                Ok(m) => Some(m.len()),
                Err(err) => {
                    errors.push(fret_core::ExternalDropReadError {
                        name,
                        message: format!("metadata failed: {err}"),
                    });
                    continue;
                }
            };

            if let Some(len) = meta_len
                && len > limits.max_file_bytes
            {
                errors.push(fret_core::ExternalDropReadError {
                    name,
                    message: format!(
                        "file too large ({} bytes > max_file_bytes {})",
                        len, limits.max_file_bytes
                    ),
                });
                continue;
            }

            if total >= limits.max_total_bytes {
                errors.push(fret_core::ExternalDropReadError {
                    name,
                    message: format!(
                        "read too large (total {} >= max_total_bytes {})",
                        total, limits.max_total_bytes
                    ),
                });
                break;
            }

            let bytes = match std::fs::read(path) {
                Ok(bytes) => bytes,
                Err(err) => {
                    errors.push(fret_core::ExternalDropReadError {
                        name,
                        message: format!("read failed: {err}"),
                    });
                    continue;
                }
            };

            if bytes.len() as u64 > limits.max_file_bytes {
                errors.push(fret_core::ExternalDropReadError {
                    name,
                    message: format!(
                        "file too large ({} bytes > max_file_bytes {})",
                        bytes.len(),
                        limits.max_file_bytes
                    ),
                });
                continue;
            }

            let next_total = total.saturating_add(bytes.len() as u64);
            if next_total > limits.max_total_bytes {
                errors.push(fret_core::ExternalDropReadError {
                    name,
                    message: format!(
                        "read too large (next_total {} > max_total_bytes {})",
                        next_total, limits.max_total_bytes
                    ),
                });
                break;
            }

            total = next_total;
            files.push(fret_core::ExternalDropFileData { name, bytes });
        }

        Some(fret_core::IncomingOpenDataEvent {
            token,
            files,
            texts: Vec::new(),
            errors,
            limits: include_limits.then_some(limits),
        })
    }

    pub(super) fn drain_effects(&mut self, event_loop: &dyn ActiveEventLoop) {
        const MAX_EFFECT_DRAIN_TURNS: usize = 8;

        for _ in 0..MAX_EFFECT_DRAIN_TURNS {
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
                        if let Some(state) = self.windows.get(window) {
                            state.window.request_redraw();
                            // Some platforms may not wake the event loop for `request_redraw()`
                            // alone; scheduling a one-shot RAF ensures the first frame presents
                            // without requiring any input events.
                            self.raf_windows.insert(window);
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
                        if safe_area_insets.is_some() || occlusion_insets.is_some() {
                            let entry =
                                self.diag_window_insets_overrides.entry(window).or_default();
                            if let Some(value) = safe_area_insets {
                                entry.safe_area_insets = Some(value);
                            }
                            if let Some(value) = occlusion_insets {
                                entry.occlusion_insets = Some(value);
                            }
                        }

                        let mut changed = false;
                        self.app.with_global_mut(
                            fret_core::WindowMetricsService::default,
                            |svc, _app| {
                                if let Some(value) = safe_area_insets {
                                    let current = svc.safe_area_insets(window);
                                    let current_known = svc.safe_area_insets_is_known(window);
                                    let needs_set = if value.is_none() {
                                        !current_known || current.is_some()
                                    } else {
                                        !current_known || current != value
                                    };
                                    if needs_set {
                                        svc.set_safe_area_insets(window, value);
                                        changed = true;
                                    }
                                }
                                if let Some(value) = occlusion_insets {
                                    let current = svc.occlusion_insets(window);
                                    let current_known = svc.occlusion_insets_is_known(window);
                                    let needs_set = if value.is_none() {
                                        !current_known || current.is_some()
                                    } else {
                                        !current_known || current != value
                                    };
                                    if needs_set {
                                        svc.set_occlusion_insets(window, value);
                                        changed = true;
                                    }
                                }
                            },
                        );
                        if changed && let Some(state) = self.windows.get(window) {
                            state.window.request_redraw();
                            self.raf_windows.insert(window);
                        }
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
                        self.raf_windows.insert(window);
                        if let Some(state) = self.windows.get(window) {
                            state.window.request_redraw();
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
                        return;
                    }
                    Effect::ShowAboutPanel => {
                        #[cfg(target_os = "macos")]
                        {
                            super::macos_menu::show_about_panel();
                        }
                    }
                    Effect::HideApp => {
                        #[cfg(target_os = "macos")]
                        {
                            super::macos_menu::hide_app();
                        }
                    }
                    Effect::HideOtherApps => {
                        #[cfg(target_os = "macos")]
                        {
                            super::macos_menu::hide_other_apps();
                        }
                    }
                    Effect::UnhideAllApps => {
                        #[cfg(target_os = "macos")]
                        {
                            super::macos_menu::unhide_all_apps();
                        }
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
                        if enabled {
                            self.diag_clipboard_force_unavailable_windows.insert(window);
                        } else {
                            self.diag_clipboard_force_unavailable_windows
                                .remove(&window);
                        }
                    }
                    Effect::DiagIncomingOpenInject { window, items } => {
                        let token = self.allocate_incoming_open_token();

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
                        self.deliver_window_event_now(
                            window,
                            &Event::IncomingOpenRequest {
                                token,
                                items: request_items,
                            },
                        );
                    }
                    Effect::ClipboardSetText { text } => {
                        if let Err(err) = self.clipboard.set_text(&text) {
                            tracing::debug!(?err, "failed to set clipboard text");
                        }
                    }
                    Effect::ClipboardGetText { window, token } => {
                        if self
                            .diag_clipboard_force_unavailable_windows
                            .contains(&window)
                        {
                            self.deliver_window_event_now(
                                window,
                                &Event::ClipboardTextUnavailable {
                                    token,
                                    message: None,
                                },
                            );
                            continue;
                        }

                        match self.clipboard.get_text() {
                            Ok(Some(text)) => self.deliver_window_event_now(
                                window,
                                &Event::ClipboardText { token, text },
                            ),
                            Ok(None) => self.deliver_window_event_now(
                                window,
                                &Event::ClipboardTextUnavailable {
                                    token,
                                    message: None,
                                },
                            ),
                            Err(err) => {
                                tracing::debug!(?err, "failed to read clipboard text");
                                self.deliver_window_event_now(
                                    window,
                                    &Event::ClipboardTextUnavailable {
                                        token,
                                        message: Some(format!("{err:?}")),
                                    },
                                );
                            }
                        }
                    }
                    Effect::PrimarySelectionSetText { text } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.clipboard.primary_text {
                            continue;
                        }
                        if let Err(err) = self.clipboard.set_primary_text(&text) {
                            tracing::debug!(?err, "failed to set primary selection text");
                        }
                    }
                    Effect::PrimarySelectionGetText { window, token } => {
                        if self
                            .diag_clipboard_force_unavailable_windows
                            .contains(&window)
                        {
                            self.deliver_window_event_now(
                                window,
                                &Event::PrimarySelectionTextUnavailable { token },
                            );
                            continue;
                        }

                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.clipboard.primary_text {
                            self.deliver_window_event_now(
                                window,
                                &Event::PrimarySelectionTextUnavailable { token },
                            );
                            continue;
                        }

                        match self.clipboard.get_primary_text() {
                            Ok(Some(text)) => self.deliver_window_event_now(
                                window,
                                &Event::PrimarySelectionText { token, text },
                            ),
                            Ok(None) | Err(_) => self.deliver_window_event_now(
                                window,
                                &Event::PrimarySelectionTextUnavailable { token },
                            ),
                        }
                    }
                    Effect::ExternalDropReadAll { window, token } => {
                        let limits = fret_platform::external_drop::ExternalDropReadLimits {
                            max_total_bytes: self.config.external_drop_max_total_bytes,
                            max_file_bytes: self.config.external_drop_max_file_bytes,
                            max_files: self.config.external_drop_max_files,
                        };

                        if let Some(paths) = self.external_drop.paths(token).map(|p| p.to_vec())
                            && self.spawn_platform_completion_task(window, move || {
                                let event = NativeExternalDrop::read_paths(token, paths, limits);
                                PlatformCompletion::ExternalDropData(event)
                            })
                        {
                            continue;
                        }

                        let Some(event) = self.external_drop.read_all(token, limits) else {
                            continue;
                        };
                        self.deliver_window_event_now(window, &Event::ExternalDropData(event));
                    }
                    Effect::ExternalDropReadAllWithLimits {
                        window,
                        token,
                        limits,
                    } => {
                        let cap = fret_platform::external_drop::ExternalDropReadLimits {
                            max_total_bytes: self.config.external_drop_max_total_bytes,
                            max_file_bytes: self.config.external_drop_max_file_bytes,
                            max_files: self.config.external_drop_max_files,
                        };
                        let limits = limits.capped_by(cap);

                        if let Some(paths) = self.external_drop.paths(token).map(|p| p.to_vec())
                            && self.spawn_platform_completion_task(window, move || {
                                let event = NativeExternalDrop::read_paths(token, paths, limits);
                                PlatformCompletion::ExternalDropData(event)
                            })
                        {
                            continue;
                        }

                        let Some(event) = self.external_drop.read_all(token, limits) else {
                            continue;
                        };
                        self.deliver_window_event_now(window, &Event::ExternalDropData(event));
                    }
                    Effect::ExternalDropRelease { token } => {
                        self.external_drop.release(token);
                    }
                    Effect::OpenUrl { url, .. } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.shell.open_url {
                            continue;
                        }
                        if let Err(err) = self.open_url.open_url(&url) {
                            tracing::debug!(?err, url = %url, "failed to open url");
                        }
                    }
                    Effect::ShareSheetShow {
                        window,
                        token,
                        items,
                    } => {
                        let _ = items;
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.shell.share_sheet {
                            self.deliver_window_event_now(
                                window,
                                &Event::ShareSheetCompleted {
                                    token,
                                    outcome: fret_core::ShareSheetOutcome::Unavailable,
                                },
                            );
                            continue;
                        }

                        // Desktop share sheets are not implemented yet; report unavailable.
                        self.deliver_window_event_now(
                            window,
                            &Event::ShareSheetCompleted {
                                token,
                                outcome: fret_core::ShareSheetOutcome::Unavailable,
                            },
                        );
                    }
                    Effect::FileDialogOpen { window, options } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.fs.file_dialogs {
                            continue;
                        }
                        match self.file_dialog.open_files(&options) {
                            Ok(Some(selection)) => {
                                self.deliver_platform_completion_now(
                                    window,
                                    PlatformCompletion::FileDialogSelection(selection),
                                );
                            }
                            Ok(None) => {
                                self.deliver_platform_completion_now(
                                    window,
                                    PlatformCompletion::FileDialogCanceled,
                                );
                            }
                            Err(err) => {
                                tracing::debug!(?err, "file dialog open failed");
                            }
                        }
                    }
                    Effect::FileDialogReadAll { window, token } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.fs.file_dialogs {
                            continue;
                        }
                        let limits = fret_platform::external_drop::ExternalDropReadLimits {
                            max_total_bytes: self.config.file_dialog_max_total_bytes,
                            max_file_bytes: self.config.file_dialog_max_file_bytes,
                            max_files: self.config.file_dialog_max_files,
                        };

                        if let Some(paths) = self.file_dialog.paths(token).map(|p| p.to_vec())
                            && self.spawn_platform_completion_task(window, move || {
                                let data = NativeFileDialog::read_paths(token, paths, limits);
                                PlatformCompletion::FileDialogData(data)
                            })
                        {
                            continue;
                        }

                        let Some(data) = self.file_dialog.read_all(token, limits) else {
                            continue;
                        };
                        self.deliver_platform_completion_now(
                            window,
                            PlatformCompletion::FileDialogData(data),
                        );
                    }
                    Effect::FileDialogReadAllWithLimits {
                        window,
                        token,
                        limits,
                    } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.fs.file_dialogs {
                            continue;
                        }
                        let cap = fret_platform::external_drop::ExternalDropReadLimits {
                            max_total_bytes: self.config.file_dialog_max_total_bytes,
                            max_file_bytes: self.config.file_dialog_max_file_bytes,
                            max_files: self.config.file_dialog_max_files,
                        };
                        let limits = limits.capped_by(cap);

                        if let Some(paths) = self.file_dialog.paths(token).map(|p| p.to_vec())
                            && self.spawn_platform_completion_task(window, move || {
                                let data = NativeFileDialog::read_paths(token, paths, limits);
                                PlatformCompletion::FileDialogData(data)
                            })
                        {
                            continue;
                        }

                        let Some(data) = self.file_dialog.read_all(token, limits) else {
                            continue;
                        };
                        self.deliver_platform_completion_now(
                            window,
                            PlatformCompletion::FileDialogData(data),
                        );
                    }
                    Effect::FileDialogRelease { token } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.fs.file_dialogs {
                            continue;
                        }
                        self.file_dialog.release(token);
                    }
                    Effect::IncomingOpenReadAll { window, token } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.shell.incoming_open {
                            self.deliver_window_event_now(
                                window,
                                &Event::IncomingOpenUnavailable { token },
                            );
                            continue;
                        }

                        let limits = self.incoming_open_limits_cap();
                        let include_limits = false;
                        match self.build_incoming_open_data_event(token, limits, include_limits) {
                            Some(data) => self
                                .deliver_window_event_now(window, &Event::IncomingOpenData(data)),
                            None => self.deliver_window_event_now(
                                window,
                                &Event::IncomingOpenUnavailable { token },
                            ),
                        }
                    }
                    Effect::IncomingOpenReadAllWithLimits {
                        window,
                        token,
                        limits,
                    } => {
                        let caps = self
                            .app
                            .global::<PlatformCapabilities>()
                            .cloned()
                            .unwrap_or_default();
                        if !caps.shell.incoming_open {
                            self.deliver_window_event_now(
                                window,
                                &Event::IncomingOpenUnavailable { token },
                            );
                            continue;
                        }

                        let cap = self.incoming_open_limits_cap();
                        let limits = limits.capped_by(cap);
                        let include_limits = true;
                        match self.build_incoming_open_data_event(token, limits, include_limits) {
                            Some(data) => self
                                .deliver_window_event_now(window, &Event::IncomingOpenData(data)),
                            None => self.deliver_window_event_now(
                                window,
                                &Event::IncomingOpenUnavailable { token },
                            ),
                        }
                    }
                    Effect::IncomingOpenRelease { token } => {
                        self.diag_incoming_open_payloads.remove(&token);
                        self.incoming_open_path_payloads.remove(&token);
                    }
                    Effect::TextAddFonts { fonts } => {
                        let Some(renderer) = self.renderer.as_mut() else {
                            continue;
                        };

                        let added = renderer.add_fonts(fonts);
                        if added == 0 {
                            continue;
                        }

                        // Font catalog refresh trigger (ADR 0258): `Effect::TextAddFonts`.
                        crate::runner::font_catalog::apply_renderer_font_catalog_update(
                            &mut self.app,
                            renderer,
                            fret_runtime::FontFamilyDefaultsPolicy::None,
                        );
                        self.request_redraw_all_windows();
                    }
                    Effect::TextRescanSystemFonts => {
                        self.request_system_font_rescan();
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
                        let Some(context) = self.context.as_ref() else {
                            self.deliver_window_event_now(
                                window,
                                &Event::ImageRegisterFailed {
                                    token,
                                    message: "wgpu not initialized".to_string(),
                                },
                            );
                            continue;
                        };
                        let Some(renderer) = self.renderer.as_mut() else {
                            self.deliver_window_event_now(
                                window,
                                &Event::ImageRegisterFailed {
                                    token,
                                    message: "renderer not initialized".to_string(),
                                },
                            );
                            continue;
                        };

                        if width == 0 || height == 0 {
                            self.deliver_window_event_now(
                                window,
                                &Event::ImageRegisterFailed {
                                    token,
                                    message: format!("invalid image size: {width}x{height}"),
                                },
                            );
                            continue;
                        }

                        let expected_len = (width as usize)
                            .saturating_mul(height as usize)
                            .saturating_mul(4);
                        if bytes.len() != expected_len {
                            self.deliver_window_event_now(
                                window,
                                &Event::ImageRegisterFailed {
                                    token,
                                    message: format!(
                                        "invalid rgba8 byte length: got {} expected {}",
                                        bytes.len(),
                                        expected_len
                                    ),
                                },
                            );
                            continue;
                        }

                        let color_space = match color_info.encoding {
                            fret_core::ImageEncoding::Srgb => fret_render::ImageColorSpace::Srgb,
                            fret_core::ImageEncoding::Linear => {
                                fret_render::ImageColorSpace::Linear
                            }
                        };

                        let uploaded = fret_render::upload_rgba8_image(
                            &context.device,
                            &context.queue,
                            (width, height),
                            &bytes,
                            color_space,
                        );

                        let view = uploaded
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        let image = renderer.register_image(fret_render::ImageDescriptor {
                            view,
                            size: uploaded.size,
                            format: uploaded.format,
                            color_space: uploaded.color_space,
                            alpha_mode,
                        });
                        self.uploaded_images.insert(
                            image,
                            UploadedImageEntry {
                                uploaded,
                                stream_generation: 0,
                                alpha_mode,
                                nv12_planes: None,
                            },
                        );

                        self.deliver_window_event_now(
                            window,
                            &Event::ImageRegistered {
                                token,
                                image,
                                width,
                                height,
                            },
                        );
                        if let Some(state) = self.windows.get(window) {
                            state.window.request_redraw();
                        }
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
                        self.apply_streaming_image_update_rgba8(
                            &mut stats,
                            StreamingImageUpdateRgba8 {
                                window,
                                token,
                                image,
                                stream_generation,
                                width,
                                height,
                                update_rect_px,
                                bytes_per_row,
                                bytes: &bytes,
                                color_info,
                                alpha_mode,
                            },
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
                        stats.yuv_conversions_attempted =
                            stats.yuv_conversions_attempted.saturating_add(1);
                        if self.try_apply_streaming_image_update_nv12_gpu(
                            &mut stats,
                            StreamingImageUpdateNv12 {
                                window,
                                token,
                                image,
                                stream_generation,
                                width,
                                height,
                                update_rect_px,
                                y_bytes_per_row,
                                y_plane: &y_plane,
                                uv_bytes_per_row,
                                uv_plane: &uv_plane,
                                color_info,
                            },
                        ) {
                            continue;
                        }

                        let t0 = Instant::now();
                        match crate::runner::yuv::nv12_to_rgba8_rect(
                            crate::runner::yuv::Nv12ToRgba8RectInput {
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
                                    &mut stats,
                                    StreamingImageUpdateRgba8 {
                                        window,
                                        token,
                                        image,
                                        stream_generation,
                                        width,
                                        height,
                                        update_rect_px: Some(rect),
                                        bytes_per_row: rect.w.saturating_mul(4),
                                        bytes: &rgba,
                                        color_info: fret_core::ImageColorInfo::srgb_rgba(),
                                        alpha_mode: fret_core::AlphaMode::Opaque,
                                    },
                                );
                            }
                            Err(_message) => {
                                if self.config.streaming_update_ack_enabled {
                                    let target = window
                                        .or(self.main_window)
                                        .or_else(|| self.windows.keys().next());
                                    if let Some(target) = target {
                                        self.deliver_window_event_now(
                                            target,
                                            &Event::ImageUpdateDropped {
                                                token,
                                                image,
                                                reason:
                                                    fret_core::ImageUpdateDropReason::InvalidPayload,
                                            },
                                        );
                                    }
                                }
                            }
                        }
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
                        stats.yuv_conversions_attempted =
                            stats.yuv_conversions_attempted.saturating_add(1);
                        let t0 = Instant::now();
                        match crate::runner::yuv::i420_to_rgba8_rect(
                            crate::runner::yuv::I420ToRgba8RectInput {
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
                                    &mut stats,
                                    StreamingImageUpdateRgba8 {
                                        window,
                                        token,
                                        image,
                                        stream_generation,
                                        width,
                                        height,
                                        update_rect_px: Some(rect),
                                        bytes_per_row: rect.w.saturating_mul(4),
                                        bytes: &rgba,
                                        color_info: fret_core::ImageColorInfo::srgb_rgba(),
                                        alpha_mode: fret_core::AlphaMode::Opaque,
                                    },
                                );
                            }
                            Err(_message) => {
                                if self.config.streaming_update_ack_enabled {
                                    let target = window
                                        .or(self.main_window)
                                        .or_else(|| self.windows.keys().next());
                                    if let Some(target) = target {
                                        self.deliver_window_event_now(
                                            target,
                                            &Event::ImageUpdateDropped {
                                                token,
                                                image,
                                                reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Effect::ImageUnregister { image } => {
                        let Some(renderer) = self.renderer.as_mut() else {
                            continue;
                        };

                        self.uploaded_images.remove(&image);

                        if !renderer.unregister_image(image) {
                            continue;
                        }

                        for (_id, state) in self.windows.iter() {
                            state.window.request_redraw();
                        }
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
                    Effect::Window(req) => match req {
                        WindowRequest::Close(window) => {
                            let is_main = Some(window) == self.main_window;
                            let closed = self.close_window(window);
                            if !closed {
                                continue;
                            }

                            if is_main && self.config.exit_on_main_window_close {
                                let windows: Vec<fret_core::AppWindowId> =
                                    self.windows.keys().collect();
                                for window in windows {
                                    let _ = self.force_close_window(window);
                                }
                                self.dispatcher.shutdown();
                                event_loop.exit();
                                return;
                            }

                            if self.windows.is_empty() {
                                self.dispatcher.shutdown();
                                event_loop.exit();
                                return;
                            }
                        }
                        WindowRequest::Create(create) => {
                            if matches!(create.kind, CreateWindowKind::DockFloating { .. }) {
                                dock_tearoff_log(format_args!(
                                    "[effect-window-create] kind={:?} anchor={:?}",
                                    create.kind, create.anchor
                                ));
                            }
                            let new_window =
                                match self.create_window_from_request(event_loop, &create) {
                                    Ok(id) => id,
                                    Err(e) => {
                                        error!(error = ?e, "failed to create window from request");
                                        continue;
                                    }
                                };
                            if matches!(
                                create.kind,
                                CreateWindowKind::DockFloating { .. }
                                    | CreateWindowKind::DockRestore { .. }
                            ) {
                                self.dock_floating_windows.insert(new_window);
                            }

                            if let CreateWindowKind::DockFloating { source_window, .. } =
                                &create.kind
                            {
                                #[cfg(target_os = "macos")]
                                {
                                    // When tearing off during an active drag, macOS may create the
                                    // new window behind the source window. Bring it to front
                                    // immediately so the subsequent `drag_window()` (if used)
                                    // behaves like ImGui's multi-viewport UX.
                                    let sender =
                                        self.windows.get(*source_window).map(|w| w.window.as_ref());
                                    if let Some(state) = self.windows.get(new_window) {
                                        let _ =
                                            bring_window_to_front(state.window.as_ref(), sender);
                                    }
                                }

                                if let Some(anchor) = create.anchor
                                    && let Some(state) = self.windows.get(new_window)
                                    && let Some(pos) = self
                                        .compute_window_outer_position_from_cursor_grab(
                                            new_window,
                                            anchor.position,
                                        )
                                {
                                    state.window.set_outer_position(pos);
                                }

                                if self.is_left_mouse_down_for_window(*source_window) {
                                    let grab_offset = create
                                        .anchor
                                        .map(|a| a.position)
                                        .unwrap_or(Point::new(Px(40.0), Px(20.0)));
                                    let caps = self
                                        .app
                                        .global::<PlatformCapabilities>()
                                        .cloned()
                                        .unwrap_or_default();
                                    let allow_follow = caps.ui.window_set_outer_position
                                        == fret_runtime::WindowSetOuterPositionQuality::Reliable;
                                    if allow_follow {
                                        if caps.ui.window_z_level
                                            != fret_runtime::WindowZLevelQuality::None
                                            && let Some(state) = self.windows.get(new_window)
                                        {
                                            state.window.set_window_level(WindowLevel::AlwaysOnTop);
                                        }

                                        self.dock_tearoff_follow = Some(super::DockTearoffFollow {
                                            window: new_window,
                                            source_window: *source_window,
                                            grab_offset,
                                            manual_follow: true,
                                            last_outer_pos: None,
                                        });
                                        // Do not call `drag_window()` here. ImGui drives multi-viewport
                                        // window movement by updating the platform window position in
                                        // response to mouse motion; native OS dragging tends to
                                        // introduce a fixed cursor offset and prevents reliable
                                        // hit-testing of other windows under the moving viewport.
                                    }
                                }
                                let panel = match &create.kind {
                                    CreateWindowKind::DockFloating { panel, .. } => Some(panel),
                                    _ => None,
                                };
                                self.enqueue_window_front(
                                    new_window,
                                    Some(*source_window),
                                    panel.cloned(),
                                    now,
                                );
                            }

                            self.driver
                                .window_created(&mut self.app, &create, new_window);

                            self.app.request_redraw(new_window);
                        }
                        WindowRequest::SetInnerSize { window, size } => {
                            if let Some(state) = self.windows.get(window) {
                                let _ = state.window.request_surface_size(
                                    winit::dpi::LogicalSize::new(
                                        size.width.0 as f64,
                                        size.height.0 as f64,
                                    )
                                    .into(),
                                );
                                state.window.request_redraw();
                            }
                        }
                        WindowRequest::SetOuterPosition { window, position } => {
                            if let Some(state) = self.windows.get(window) {
                                state
                                    .window
                                    .set_outer_position(winit::dpi::Position::Logical(
                                        winit::dpi::LogicalPosition::new(
                                            position.x as f64,
                                            position.y as f64,
                                        ),
                                    ));
                                state.window.request_redraw();
                            }
                        }
                        WindowRequest::Raise {
                            window,
                            sender: sender_id,
                        } => {
                            let sender_window = sender_id
                                .and_then(|id| self.windows.get(id))
                                .map(|w| w.window.as_ref());
                            if let Some(state) = self.windows.get(window) {
                                let _ = bring_window_to_front(state.window.as_ref(), sender_window);
                                state.window.request_redraw();
                            }
                            #[cfg(target_os = "macos")]
                            {
                                if self.windows.contains_key(window) {
                                    self.enqueue_window_front(window, sender_id, None, now);
                                }
                            }
                        }
                    },
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
                match self.streaming_uploads.pending_redraw_hint() {
                    Some(windows) if windows.is_empty() => {
                        for (_id, state) in self.windows.iter() {
                            state.window.request_redraw();
                        }
                    }
                    Some(windows) => {
                        for window in windows {
                            if let Some(state) = self.windows.get(window) {
                                state.window.request_redraw();
                            }
                        }
                    }
                    None => {}
                }
            }

            if !did_work {
                break;
            }
        }
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
            && let (Some(renderer), Some(config)) = (
                self.renderer.as_mut(),
                self.app.global::<fret_core::TextFontFamilyConfig>(),
            )
            && renderer.set_text_font_families(config)
        {
            let new_key = renderer.text_font_stack_key();
            let old_key = self
                .app
                .global::<fret_runtime::TextFontStackKey>()
                .map(|k| k.0);
            if old_key != Some(new_key) {
                self.app.set_global::<fret_runtime::TextFontStackKey>(
                    fret_runtime::TextFontStackKey(new_key),
                );
            }

            for (_id, state) in self.windows.iter() {
                state.window.request_redraw();
            }
        }

        if changed.contains(&TypeId::of::<fret_runtime::fret_i18n::I18nService>())
            && let Some(renderer) = self.renderer.as_mut()
        {
            let locale = self
                .app
                .global::<fret_runtime::fret_i18n::I18nService>()
                .and_then(|service| service.preferred_locales().first())
                .map(|locale| locale.to_string());
            if renderer.set_text_locale(locale.as_deref()) {
                let new_key = renderer.text_font_stack_key();
                let old_key = self
                    .app
                    .global::<fret_runtime::TextFontStackKey>()
                    .map(|k| k.0);
                if old_key != Some(new_key) {
                    self.app.set_global::<fret_runtime::TextFontStackKey>(
                        fret_runtime::TextFontStackKey(new_key),
                    );
                }

                for (_id, state) in self.windows.iter() {
                    state.window.request_redraw();
                }
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
