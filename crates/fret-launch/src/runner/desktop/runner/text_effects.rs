use std::sync::OnceLock;

use fret_assets::AssetRequest;
use fret_core::time::Instant;
use fret_runtime::{FontFamilyDefaultsPolicy, RendererFontSourceLane};

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
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
        let _ = crate::runner::font_catalog::publish_system_font_rescan_state(
            &mut self.app,
            self.system_font_rescan_in_flight,
            self.system_font_rescan_pending,
        );
    }

    pub(super) fn handle_text_add_font_assets(&mut self, requests: Vec<AssetRequest>) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        let added = crate::runner::font_catalog::inject_font_asset_requests_and_refresh_catalog(
            &mut self.app,
            renderer,
            requests,
            RendererFontSourceLane::AssetRequest,
            FontFamilyDefaultsPolicy::None,
        );
        if added == 0 {
            return;
        }

        self.request_redraw_all_windows();
    }

    pub(super) fn handle_text_rescan_system_fonts(&mut self) {
        self.request_system_font_rescan();
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
            if let Some(renderer) = self.renderer.as_mut() {
                // If system fonts are disabled, desktop async startup may still have seeded an
                // empty runtime catalog while the renderer already contains the bundled baseline.
                // Reconcile the catalog with the current renderer environment instead of leaving
                // diagnostics waiting on an impossible system-font rescan.
                let _ = crate::runner::font_catalog::apply_renderer_font_catalog_update(
                    &mut self.app,
                    renderer,
                    FontFamilyDefaultsPolicy::None,
                );
                self.request_redraw_all_windows();
            }
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
            FontFamilyDefaultsPolicy::None,
        );
        self.request_redraw_all_windows();
    }

    pub(super) fn finish_system_font_rescan_result_state(&mut self) -> bool {
        self.system_font_rescan_in_flight = false;
        let should_restart = self.system_font_rescan_pending;
        self.system_font_rescan_pending = false;
        self.publish_system_font_rescan_state();
        should_restart
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

        let should_restart = self.finish_system_font_rescan_result_state();

        let Some(renderer) = self.renderer.as_mut() else {
            if should_restart {
                self.request_system_font_rescan();
            }
            return true;
        };

        let rescan_entries = result.all_font_catalog_entries().to_vec();
        if !renderer.apply_system_font_rescan_result(result) {
            // Desktop async startup seeds an empty runtime catalog before the background scan.
            // A no-op renderer apply can still carry the first completed catalog snapshot, so
            // publish the scan entries without requiring a second main-thread enumeration.
            let _ =
                crate::runner::font_catalog::publish_renderer_font_environment_from_catalog_entries(
                    &mut self.app,
                    renderer,
                    rescan_entries,
                    FontFamilyDefaultsPolicy::None,
                );
            self.request_redraw_all_windows();
            if should_restart {
                self.request_system_font_rescan();
            }
            return true;
        }

        // Font catalog refresh trigger (ADR 0258): explicit system font rescan (async).
        crate::runner::font_catalog::apply_renderer_font_catalog_update(
            &mut self.app,
            renderer,
            FontFamilyDefaultsPolicy::None,
        );
        self.request_redraw_all_windows();

        if should_restart {
            self.request_system_font_rescan();
        }

        true
    }
}
