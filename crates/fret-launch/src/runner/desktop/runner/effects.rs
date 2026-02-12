use std::{any::TypeId, sync::OnceLock};

use super::{WinitRunner, WinitWindowContext};

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

    pub(super) fn request_system_font_rescan(&mut self) {
        if !Self::system_font_rescan_async_enabled() {
            self.rescan_system_fonts_sync();
            return;
        }

        if self.system_font_rescan_in_flight {
            self.system_font_rescan_pending = true;
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

    pub(super) fn apply_pending_system_font_rescan_result(&mut self) -> bool {
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
