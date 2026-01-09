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
//!
//! UI app “golden path” example (native, requires the `ui-app-driver` feature):
//!
//! ```no_run
//! use fret_bootstrap::BootstrapBuilder;
//!
//! # #[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
//! # fn demo() -> Result<(), fret_launch::RunnerError> {
//! let builder = fret_bootstrap::ui_app("todo", |_app, _window| (), |_cx, _state| vec![])
//!     .with_default_settings_json()?
//!     .register_icon_pack(|_icons| {});
//! builder.run()?;
//! # Ok(())
//! # }
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

    /// Configure budgets for UI render asset caches (`ImageAssetCache` / `SvgAssetCache`).
    ///
    /// This is an ecosystem-level convenience; it does not change the core "resource handles" boundary (ADR 0004).
    #[cfg(feature = "ui-assets")]
    pub fn with_ui_assets_budgets(
        mut self,
        image_budget_bytes: u64,
        image_max_ready_entries: usize,
        svg_budget_bytes: u64,
        svg_max_ready_entries: usize,
    ) -> Self {
        self.inner = self.inner.init_app(move |app| {
            use fret_ui_assets::image_asset_cache::ImageAssetCache;
            use fret_ui_assets::svg_asset_cache::SvgAssetCache;

            app.with_global_mut(ImageAssetCache::default, |cache, _app| {
                cache.set_budget_bytes(image_budget_bytes);
                cache.set_max_ready_entries(image_max_ready_entries);
            });

            app.with_global_mut(SvgAssetCache::default, |cache, _app| {
                cache.set_budget_bytes(svg_budget_bytes);
                cache.set_max_ready_entries(svg_max_ready_entries);
            });
        });

        self
    }

    /// Enable the `fret-launch` dev hotpatch trigger by setting environment variables.
    ///
    /// This is intended for local developer workflows; production apps should not rely on it.
    ///
    /// # Safety
    ///
    /// `std::env::set_var` is unsafe on Rust 2024 because mutating the process environment while
    /// other threads may read it can cause undefined behavior on some platforms.
    /// Call this early during startup, before any other threads are spawned.
    #[cfg(feature = "hotpatch-subsecond")]
    pub unsafe fn enable_hotpatch_env(
        self,
        trigger_path: impl AsRef<Path>,
        poll_interval_ms: u64,
    ) -> Self {
        let trigger_path = trigger_path.as_ref();

        // Safety: the caller must ensure no other threads concurrently read/write the process
        // environment while these variables are being set.
        unsafe {
            std::env::set_var("FRET_HOTPATCH", "1");
            std::env::set_var("FRET_HOTPATCH_TRIGGER_PATH", trigger_path.as_os_str());
            std::env::set_var("FRET_HOTPATCH_POLL_MS", poll_interval_ms.to_string());
        }

        self
    }

    /// Enable the `fret-launch` dev hotpatch trigger using a file-based polling marker.
    ///
    /// This is a clearer name for `enable_hotpatch_env`.
    ///
    /// # Safety
    ///
    /// See `enable_hotpatch_env`.
    #[cfg(feature = "hotpatch-subsecond")]
    pub unsafe fn enable_hotpatch_file_trigger_env(
        self,
        trigger_path: impl AsRef<Path>,
        poll_interval_ms: u64,
    ) -> Self {
        unsafe { self.enable_hotpatch_env(trigger_path, poll_interval_ms) }
    }

    /// Enable Subsecond hotpatch by connecting to a devserver websocket.
    ///
    /// The runner will listen for Dioxus-style devserver messages and apply incoming Subsecond
    /// jump tables. Once a patch is applied, the runner schedules a safe hot-reload reset on the
    /// next event-loop turn.
    ///
    /// # Safety
    ///
    /// `std::env::set_var` is unsafe on Rust 2024 because mutating the process environment while
    /// other threads may read it can cause undefined behavior on some platforms.
    /// Call this early during startup, before any other threads are spawned.
    #[cfg(feature = "hotpatch-subsecond")]
    pub unsafe fn enable_hotpatch_subsecond_devserver_env(
        self,
        devserver_ws_endpoint: impl AsRef<str>,
    ) -> Self {
        let endpoint = devserver_ws_endpoint.as_ref();

        unsafe {
            std::env::set_var("FRET_HOTPATCH", "1");
            std::env::set_var("FRET_HOTPATCH_DEVSERVER_WS", endpoint);
        }

        self
    }

    /// Same as `enable_hotpatch_subsecond_devserver_env`, but additionally sets a build-id filter.
    ///
    /// When `build_id` is set, the runner will ignore devserver patches whose `for_build_id` does
    /// not match, which helps avoid cross-process confusion in multi-app workflows.
    ///
    /// # Safety
    ///
    /// See `enable_hotpatch_subsecond_devserver_env`.
    #[cfg(feature = "hotpatch-subsecond")]
    pub unsafe fn enable_hotpatch_subsecond_devserver_env_with_build_id(
        self,
        devserver_ws_endpoint: impl AsRef<str>,
        build_id: u64,
    ) -> Self {
        let builder =
            unsafe { self.enable_hotpatch_subsecond_devserver_env(devserver_ws_endpoint) };

        unsafe {
            std::env::set_var("FRET_HOTPATCH_BUILD_ID", build_id.to_string());
        }

        builder
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

    /// Configure the main window title and size (logical pixels).
    pub fn with_main_window(mut self, title: impl Into<String>, size: (f64, f64)) -> Self {
        let title = title.into();
        let (width, height) = size;

        self.inner = self.inner.configure(move |config| {
            config.main_window_title = title.clone();
            config.main_window_size.width = width;
            config.main_window_size.height = height;
        });

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

#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
pub mod ui_app_driver;

/// Concrete `BootstrapBuilder` type returned by `ui_app` / `ui_app_with_app`.
#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
pub type UiAppBootstrapBuilder<S> = BootstrapBuilder<
    fret_launch::FnDriver<ui_app_driver::UiAppDriver<S>, ui_app_driver::UiAppWindowState<S>>,
>;

/// Create a “golden path” native UI app builder, using `App::new()` by default and allowing a
/// hook to configure the driver before it is wrapped into `FnDriver`.
///
/// Prefer passing a non-capturing closure so it can coerce to a `fn` pointer (hotpatch-friendly).
#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
pub fn ui_app_with_hooks<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(
        &mut fret_ui::ElementContext<'a, App>,
        &mut S,
    ) -> Vec<fret_ui::element::AnyElement>,
    configure: fn(ui_app_driver::UiAppDriver<S>) -> ui_app_driver::UiAppDriver<S>,
) -> UiAppBootstrapBuilder<S> {
    ui_app_with_app_and_hooks(App::new(), root_name, init_window, view, configure)
}

/// Create a “golden path” native UI app builder, using `App::new()` by default.
///
/// This hides the `FnDriver` boilerplate and keeps example code short.
#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
pub fn ui_app<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(
        &mut fret_ui::ElementContext<'a, App>,
        &mut S,
    ) -> Vec<fret_ui::element::AnyElement>,
) -> UiAppBootstrapBuilder<S> {
    ui_app_with_app(App::new(), root_name, init_window, view)
}

/// Same as `ui_app`, but allows providing a pre-configured `App`.
#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
pub fn ui_app_with_app<S: 'static>(
    app: App,
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(
        &mut fret_ui::ElementContext<'a, App>,
        &mut S,
    ) -> Vec<fret_ui::element::AnyElement>,
) -> UiAppBootstrapBuilder<S> {
    ui_app_with_app_and_hooks(app, root_name, init_window, view, |d| d)
}

/// Same as `ui_app_with_app`, but allows a hook to configure the driver before it is wrapped into
/// `FnDriver`.
#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
pub fn ui_app_with_app_and_hooks<S: 'static>(
    app: App,
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(
        &mut fret_ui::ElementContext<'a, App>,
        &mut S,
    ) -> Vec<fret_ui::element::AnyElement>,
    configure: fn(ui_app_driver::UiAppDriver<S>) -> ui_app_driver::UiAppDriver<S>,
) -> UiAppBootstrapBuilder<S> {
    let driver = configure(ui_app_driver::UiAppDriver::new(
        root_name,
        init_window,
        view,
    ))
    .into_fn_driver();
    BootstrapBuilder::new(app, driver)
}
