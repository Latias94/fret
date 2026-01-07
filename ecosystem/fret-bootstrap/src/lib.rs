//! Opinionated bootstrap utilities for Fret applications.
//!
//! This crate is intentionally *ecosystem-level* (not part of the portable kernel). It composes
//! existing primitives from `fret-launch` and friends to provide a convenient “golden path”
//! startup experience.
//!
//! Minimal example (native):
//!
//! ```no_run
//! use fret_app::App;
//! use fret_bootstrap::BootstrapBuilder;
//! use fret_launch::FnDriver;
//!
//! # fn event(_d: &mut (), _cx: fret_launch::WinitEventContext<'_, ()>, _e: &fret_core::Event) {}
//! # fn render(_d: &mut (), _cx: fret_launch::WinitRenderContext<'_, ()>) {}
//! #
//! let driver = FnDriver::new((), |_d, _app, _w| (), event, render);
//! let builder = BootstrapBuilder::new(App::new(), driver)
//!     .with_default_settings_json()?
//!     .register_icon_pack(|_icons| {});
//! builder.run()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::path::Path;

use fret_app::{App, SettingsError, SettingsFileV1};
use fret_icons::IconRegistry;

#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error(transparent)]
    Settings(#[from] SettingsError),
}

/// Apply `SettingsFileV1` to an `App` and runner config.
///
/// This is a convenience helper for the common pattern:
/// - apply docking interaction settings to `App` globals
/// - apply font family overrides to runner config
#[cfg(not(target_arch = "wasm32"))]
pub fn apply_settings(
    app: &mut App,
    config: &mut fret_launch::WinitRunnerConfig,
    settings: &SettingsFileV1,
) {
    app.set_global(settings.clone());
    app.set_global(settings.docking_interaction_settings());
    config.text_font_families = settings.fonts.clone();
}

/// Builder wrapper around `fret_launch::WinitAppBuilder` with common bootstrapping conveniences.
#[cfg(not(target_arch = "wasm32"))]
pub struct BootstrapBuilder<D: fret_launch::WinitAppDriver> {
    inner: fret_launch::WinitAppBuilder<D>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<D: fret_launch::WinitAppDriver + 'static> BootstrapBuilder<D> {
    pub fn new(app: App, driver: D) -> Self {
        Self {
            inner: fret_launch::WinitAppBuilder::new(app, driver),
        }
    }

    pub fn with_settings_json(mut self, path: impl AsRef<Path>) -> Result<Self, BootstrapError> {
        let path = path.as_ref();
        let Some(settings) = SettingsFileV1::load_json_if_exists(path)? else {
            return Ok(self);
        };

        let settings_for_config = settings.clone();
        self.inner = self.inner.configure(move |config| {
            config.text_font_families = settings_for_config.fonts.clone();
        });

        self.inner = self.inner.init_app(move |app| {
            app.set_global(settings.clone());
            app.set_global(settings.docking_interaction_settings());
        });

        Ok(self)
    }

    pub fn with_default_settings_json(self) -> Result<Self, BootstrapError> {
        self.with_settings_json(".fret/settings.json")
    }

    /// Register an icon pack (e.g. `fret_icons_lucide::register_icons`) into the global `IconRegistry`.
    pub fn register_icon_pack(mut self, register: fn(&mut IconRegistry)) -> Self {
        self.inner = self.inner.init_app(move |app| {
            app.with_global_mut(IconRegistry::default, |icons, _app| {
                register(icons);
            });
        });
        self
    }

    /// Pre-register all SVG icons from the global `IconRegistry` during `on_gpu_ready`.
    #[cfg(feature = "preload-icon-svgs")]
    pub fn preload_icon_svgs_on_gpu_ready(mut self) -> Self {
        self.inner = self.inner.on_gpu_ready(|app, _context, renderer| {
            let services = renderer as &mut dyn fret_core::UiServices;
            fret_ui_kit::declarative::icon::preload_icon_svgs(app, services);
        });
        self
    }

    pub fn configure(mut self, f: impl FnOnce(&mut fret_launch::WinitRunnerConfig)) -> Self {
        self.inner = self.inner.configure(f);
        self
    }

    pub fn init_app(mut self, f: impl FnOnce(&mut App)) -> Self {
        self.inner = self.inner.init_app(f);
        self
    }

    pub fn on_main_window_created(
        mut self,
        f: impl FnOnce(&mut App, fret_core::AppWindowId) + 'static,
    ) -> Self {
        self.inner = self.inner.on_main_window_created(f);
        self
    }

    pub fn on_gpu_ready(
        mut self,
        f: impl FnOnce(&mut App, &fret_render::WgpuContext, &mut fret_render::Renderer) + 'static,
    ) -> Self {
        self.inner = self.inner.on_gpu_ready(f);
        self
    }

    pub fn run(self) -> Result<(), fret_launch::RunnerError> {
        self.inner.run()
    }

    pub fn into_inner(self) -> fret_launch::WinitAppBuilder<D> {
        self.inner
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<D: fret_launch::WinitAppDriver + 'static> From<fret_launch::WinitAppBuilder<D>>
    for BootstrapBuilder<D>
{
    fn from(inner: fret_launch::WinitAppBuilder<D>) -> Self {
        Self { inner }
    }
}
