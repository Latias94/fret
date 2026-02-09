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
//!     .with_default_config_files()?
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
//! # fn demo() -> Result<(), Box<dyn std::error::Error>> {
//! let builder = fret_bootstrap::ui_app(
//!     "todo",
//!     |_app, _window| (),
//!     |_cx, _state| fret_bootstrap::ui_app_driver::ViewElements::default(),
//! )
//!     .with_default_config_files()?
//!     .register_icon_pack(|_icons| {});
//! builder.run()?;
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use fret_app::SettingsFileV1;
#[cfg(not(target_arch = "wasm32"))]
use fret_app::config_files::LayeredConfigPaths;
use fret_app::{App, KeymapFileError, MenuBarFileError, SettingsError};
use fret_i18n::{I18nLookup, I18nService, LocaleId};
use fret_i18n_fluent::{FluentCatalog, FluentLookup};
#[cfg(not(target_arch = "wasm32"))]
use fret_icons::IconRegistry;

#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error(transparent)]
    Keymap(#[from] KeymapFileError),
    #[error(transparent)]
    MenuBar(#[from] MenuBarFileError),
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
    install_default_i18n_backend(app);
    fret_app::settings::apply_settings_globals(app, settings);
    config.text_font_families = settings.fonts.clone();
}

/// Installs a default i18n backend if the app hasn't provided one yet.
///
/// Notes:
/// - This is an ecosystem-level “golden path” convenience, not a portable kernel contract.
/// - Callers can override this by installing their own backend before bootstrapping.
pub fn install_default_i18n_backend(app: &mut App) {
    let mut service = app
        .global::<I18nService>()
        .cloned()
        .unwrap_or_else(I18nService::default);
    if service.lookup().is_some() {
        return;
    }

    service.set_lookup(Some(default_i18n_lookup()));
    app.set_global(service);
}

fn default_i18n_lookup() -> Arc<dyn I18nLookup + 'static> {
    let mut catalog = FluentCatalog::new();
    catalog
        .add_locale_ftl(
            LocaleId::parse("en-US").expect("hardcoded locale en-US must parse"),
            DEFAULT_I18N_FTL_EN_US,
        )
        .expect("en-US i18n resource must load");
    catalog
        .add_locale_ftl(
            LocaleId::parse("zh-CN").expect("hardcoded locale zh-CN must parse"),
            DEFAULT_I18N_FTL_ZH_CN,
        )
        .expect("zh-CN i18n resource must load");

    let lookup = FluentLookup::new(Arc::new(catalog));
    Arc::new(lookup)
}

const DEFAULT_I18N_FTL_EN_US: &str = r#"
core-command-category-app = App
workspace-menu-file = File
workspace-menu-edit = Edit
workspace-menu-view = View
workspace-menu-window = Window

core-command-title-app-command-palette = Command Palette
core-command-title-app-about = About
core-command-title-app-preferences = Preferences...
core-command-title-app-locale-switch-next = Switch Language
core-command-title-app-hide = Hide
core-command-title-app-hide-others = Hide Others
core-command-title-app-show-all = Show All
core-command-title-app-quit = Quit
"#;

const DEFAULT_I18N_FTL_ZH_CN: &str = r#"
core-command-category-app = 应用
workspace-menu-file = 文件
workspace-menu-edit = 编辑
workspace-menu-view = 视图
workspace-menu-window = 窗口

core-command-title-app-command-palette = 命令面板
core-command-title-app-about = 关于
core-command-title-app-preferences = 偏好设置...
core-command-title-app-locale-switch-next = 切换语言
core-command-title-app-hide = 隐藏
core-command-title-app-hide-others = 隐藏其他应用
core-command-title-app-show-all = 显示全部
core-command-title-app-quit = 退出
"#;

/// Builder wrapper around `fret_launch::WinitAppBuilder` with common bootstrapping conveniences.
#[cfg(not(target_arch = "wasm32"))]
pub struct BootstrapBuilder<D: fret_launch::WinitAppDriver> {
    inner: fret_launch::WinitAppBuilder<D>,
    on_gpu_ready_hooks: Vec<
        Box<dyn FnOnce(&mut App, &fret_render::WgpuContext, &mut fret_render::Renderer) + 'static>,
    >,
}

#[cfg(not(target_arch = "wasm32"))]
impl<D: fret_launch::WinitAppDriver + 'static> BootstrapBuilder<D> {
    pub fn new(app: App, driver: D) -> Self {
        Self {
            inner: fret_launch::WinitAppBuilder::new(app, driver),
            on_gpu_ready_hooks: Vec::new(),
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
            install_default_i18n_backend(app);
            fret_app::settings::apply_settings_globals(app, &settings);
            fret_app::sync_os_menu_bar(app);
        });

        Ok(self)
    }

    pub fn with_default_settings_json(self) -> Result<Self, BootstrapError> {
        self.with_settings_json(".fret/settings.json")
    }

    pub fn with_layered_settings(
        mut self,
        project_root: impl AsRef<Path>,
    ) -> Result<Self, BootstrapError> {
        let paths = LayeredConfigPaths::for_project_root(project_root);
        let (settings, _report) = fret_app::load_layered_settings(&paths)?;

        let settings_for_config = settings.clone();
        self.inner = self.inner.configure(move |config| {
            config.text_font_families = settings_for_config.fonts.clone();
        });

        self.inner = self.inner.init_app(move |app| {
            install_default_i18n_backend(app);
            fret_app::settings::apply_settings_globals(app, &settings);
            fret_app::sync_os_menu_bar(app);
        });

        Ok(self)
    }

    pub fn with_layered_keymap(
        mut self,
        project_root: impl AsRef<Path>,
    ) -> Result<Self, BootstrapError> {
        let paths = LayeredConfigPaths::for_project_root(project_root);
        let (layered, _report) = fret_app::load_layered_keymap(&paths)?;

        self.inner = self.inner.init_app(move |app| {
            fret_app::apply_layered_keymap(app, layered.clone());
        });

        Ok(self)
    }

    pub fn with_layered_menu_bar(
        mut self,
        project_root: impl AsRef<Path>,
    ) -> Result<Self, BootstrapError> {
        let paths = LayeredConfigPaths::for_project_root(project_root);
        let (layered, _report) = fret_app::load_layered_menu_bar(&paths)?;

        self.inner = self.inner.init_app(move |app| {
            if let Err(e) = fret_app::apply_layered_menu_bar(app, None, layered.clone()) {
                app.with_global_mut(
                    fret_app::ConfigFilesWatcherStatus::default,
                    |status, _app| {
                        status.note(fret_app::ConfigFilesWatcherTick {
                            reloaded_settings: false,
                            reloaded_keymap: false,
                            reloaded_menu_bar: false,
                            settings_error: None,
                            keymap_error: None,
                            menu_bar_error: Some(e.to_string()),
                            actionable_keymap_conflicts: 0,
                            keymap_conflict_samples: Vec::new(),
                        });
                    },
                );
            }
        });

        Ok(self)
    }

    /// Installs command-provided default keybindings into the app keymap.
    ///
    /// Ordering note: call this before `with_layered_keymap(...)` so user/project keymap files can
    /// override defaults via last-wins resolution.
    pub fn with_command_default_keybindings(mut self) -> Self {
        self.inner = self.inner.init_app(move |app| {
            fret_app::install_command_default_keybindings_into_keymap(app);
        });
        self
    }

    /// Installs a set of plugins into the app-owned registry (ADR 0016).
    ///
    /// Ordering note: for correct keymap layering (ADR 0021), prefer calling this before
    /// `with_layered_keymap(...)` / `with_default_config_files()` so user/project overrides remain
    /// last-wins.
    pub fn with_plugins(mut self, plugins: &[&dyn fret_app::Plugin]) -> Self {
        let plugins: Vec<&dyn fret_app::Plugin> = plugins.iter().copied().collect();
        self.inner = self.inner.init_app(move |app| {
            fret_app::install_plugins(app, plugins.iter().copied());
        });
        self
    }

    pub fn with_default_config_files(self) -> Result<Self, BootstrapError> {
        Ok(self
            .with_layered_settings(".")?
            .with_command_default_keybindings()
            .with_layered_keymap(".")?
            .with_layered_menu_bar(".")?)
    }

    /// Enables polling-based hot reload for layered `settings.json` / `keymap.json` / `menubar.json` files.
    ///
    /// This uses a repeating `Effect::SetTimer` and checks file metadata (mtime/len) on each tick.
    /// It is intended for local dev workflows and stays portable (no platform-specific watcher deps).
    pub fn with_config_files_watcher(mut self, poll_interval: Duration) -> Self {
        self.inner = self.inner.init_app(move |app| {
            fret_app::ConfigFilesWatcher::install(app, poll_interval, ".");
        });
        self
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
        let budgets = fret_ui_assets::UiAssetsBudgets {
            image_budget_bytes,
            image_max_ready_entries,
            svg_budget_bytes,
            svg_max_ready_entries,
        };
        self.inner = self.inner.init_app(move |app| {
            fret_ui_assets::install_app_with_budgets(app, budgets);
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
            app.with_global_mut(IconRegistry::default, |icons, app| {
                register(icons);
                let frozen =
                    icons.freeze_or_default_with_context("fret_bootstrap.register_icon_pack");
                app.set_global(frozen);
            });
        });
        self
    }

    /// Install the Lucide icon pack into the global `IconRegistry`.
    ///
    /// Requires enabling the `fret-bootstrap/icons-lucide` feature.
    #[cfg(feature = "icons-lucide")]
    pub fn with_lucide_icons(self) -> Self {
        self.install_app(fret_icons_lucide::install_app)
    }

    /// Install the Radix icon pack into the global `IconRegistry`.
    ///
    /// Requires enabling the `fret-bootstrap/icons-radix` feature.
    #[cfg(feature = "icons-radix")]
    pub fn with_radix_icons(self) -> Self {
        self.install_app(fret_icons_radix::install_app)
    }

    /// Pre-register all SVG icons from the global `IconRegistry` during `on_gpu_ready`.
    #[cfg(feature = "preload-icon-svgs")]
    pub fn preload_icon_svgs_on_gpu_ready(mut self) -> Self {
        self.on_gpu_ready_hooks
            .push(Box::new(|app, _context, renderer| {
                let services = renderer as &mut dyn fret_core::UiServices;
                fret_ui_kit::declarative::icon::preload_icon_svgs(app, services);
            }));
        self
    }

    pub fn configure(mut self, f: impl FnOnce(&mut fret_launch::WinitRunnerConfig)) -> Self {
        self.inner = self.inner.configure(f);
        self
    }

    /// Initialize a default tracing subscriber (if one is not already installed).
    ///
    /// Controlled by `RUST_LOG` when set; otherwise uses a conservative default filter suitable for
    /// app development.
    #[cfg(feature = "tracing")]
    pub fn with_default_tracing(self) -> Self {
        init_tracing();
        self
    }

    /// Initialize default diagnostics (tracing + panic logging) for application development.
    #[cfg(feature = "diagnostics")]
    pub fn with_default_diagnostics(self) -> Self {
        init_diagnostics();
        self
    }

    /// Configure the main window title and size (logical pixels).
    pub fn with_main_window(mut self, title: impl Into<String>, size: (f64, f64)) -> Self {
        let title = title.into();
        let (width, height) = size;

        let title_for_global = title.clone();
        self.inner = self.inner.init_app(move |app| {
            app.set_global(fret_app::AppDisplayName(Arc::from(
                title_for_global.clone(),
            )));
        });

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

    /// Install an ecosystem crate that only needs access to the app state.
    ///
    /// This runs during early initialization (before GPU services exist), which is important for
    /// correct keymap layering semantics (user/project keymaps should remain last-wins).
    pub fn install_app(mut self, install: fn(&mut App)) -> Self {
        self.inner = self.inner.init_app(install);
        self
    }

    /// Install an ecosystem crate at the UI services boundary.
    ///
    /// This runs during `on_gpu_ready`, with `services` backed by the renderer.
    pub fn install(mut self, install: fn(&mut App, &mut dyn fret_core::UiServices)) -> Self {
        self.on_gpu_ready_hooks
            .push(Box::new(move |app, _context, renderer| {
                let services = renderer as &mut dyn fret_core::UiServices;
                install(app, services);
            }));
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
        self.on_gpu_ready_hooks.push(Box::new(f));
        self
    }

    pub fn run(self) -> Result<(), fret_launch::RunnerError> {
        self.into_inner().run()
    }

    pub fn into_inner(self) -> fret_launch::WinitAppBuilder<D> {
        let BootstrapBuilder {
            mut inner,
            on_gpu_ready_hooks,
        } = self;

        if on_gpu_ready_hooks.is_empty() {
            return inner;
        }

        inner = inner.on_gpu_ready(move |app, context, renderer| {
            for hook in on_gpu_ready_hooks {
                hook(app, context, renderer);
            }
        });

        inner
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<D: fret_launch::WinitAppDriver + 'static> From<fret_launch::WinitAppBuilder<D>>
    for BootstrapBuilder<D>
{
    fn from(inner: fret_launch::WinitAppBuilder<D>) -> Self {
        Self {
            inner,
            on_gpu_ready_hooks: Vec::new(),
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
pub mod ui_app_driver;

#[cfg(all(feature = "ui-app-driver", feature = "diagnostics"))]
pub mod ui_diagnostics;

#[cfg(all(
    feature = "ui-app-driver",
    feature = "diagnostics",
    feature = "diagnostics-ws"
))]
mod ui_diagnostics_ws_bridge;

#[cfg(all(not(target_arch = "wasm32"), feature = "diagnostics"))]
pub fn init_diagnostics() {
    init_tracing();
    init_panic_hook();
}

#[cfg(all(not(target_arch = "wasm32"), feature = "tracing"))]
pub fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::prelude::*;

    const DEFAULT: &str = "info,fret=info,fret_launch=info,fret_render=info";

    let filter = std::env::var("RUST_LOG")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .and_then(|v| EnvFilter::try_new(v).ok())
        .unwrap_or_else(|| EnvFilter::try_new(DEFAULT).expect("default tracing filter is valid"));

    #[cfg(feature = "tracy")]
    {
        let tracy_enabled = std::env::var_os("FRET_TRACY").is_some_and(|v| !v.is_empty());
        if tracy_enabled {
            use tracing_subscriber::fmt::format::DefaultFields;

            #[derive(Default)]
            struct FretTracyConfig {
                fmt: DefaultFields,
                callstack_depth: u16,
            }

            impl tracing_tracy::Config for FretTracyConfig {
                type Formatter = DefaultFields;

                fn formatter(&self) -> &Self::Formatter {
                    &self.fmt
                }

                fn stack_depth(&self, metadata: &tracing::Metadata<'_>) -> u16 {
                    if self.callstack_depth == 0 {
                        return 0;
                    }

                    match metadata.name() {
                        "fret.ui.layout"
                        | "fret.ui.paint"
                        | "fret_ui.layout_all"
                        | "fret_ui.paint_all"
                        | "fret.ui.layout_engine.solve"
                        | "fret.ui.paint_cache.replay"
                        | "fret.runner.redraw"
                        | "fret.runner.prepare"
                        | "fret.runner.render"
                        | "fret.runner.record"
                        | "fret.runner.present"
                        | "fret.runner.render_scene"
                        | "ui.cache_root.mount"
                        | "ui.cache_root.reuse"
                        | "ui.cache_root.layout"
                        | "ui.cache_root.paint" => self.callstack_depth,
                        _ => 0,
                    }
                }
            }

            let callstack_depth = std::env::var("FRET_TRACY_CALLSTACK_DEPTH")
                .ok()
                .and_then(|v| v.parse::<u16>().ok())
                .unwrap_or(16);
            let callstack_enabled =
                std::env::var_os("FRET_TRACY_CALLSTACK").is_some_and(|v| !v.is_empty());

            let config = FretTracyConfig {
                fmt: DefaultFields::default(),
                callstack_depth: if callstack_enabled {
                    callstack_depth
                } else {
                    0
                },
            };

            let _ = tracing_subscriber::registry()
                .with(filter.clone())
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_target(false)
                        .compact(),
                )
                .with(tracing_tracy::TracyLayer::new(config))
                .try_init();
            return;
        }
    }

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .compact(),
        )
        .try_init();
}

#[cfg(all(not(target_arch = "wasm32"), feature = "diagnostics"))]
pub fn init_panic_hook() {
    use std::backtrace::Backtrace;
    use std::sync::Once;

    static INSTALLED: Once = Once::new();
    INSTALLED.call_once(|| {
        let default_hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |info| {
            let thread = std::thread::current();
            let thread_name = thread.name().unwrap_or("<unnamed>");

            let message = info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| info.payload().downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<non-string panic payload>".to_string());

            let location = info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                .unwrap_or_else(|| "<unknown>".to_string());

            let backtrace = Backtrace::capture();
            match backtrace.status() {
                std::backtrace::BacktraceStatus::Captured => {
                    tracing::error!(
                        thread = thread_name,
                        location = location,
                        message = message,
                        backtrace = %backtrace,
                        "panic"
                    );
                }
                std::backtrace::BacktraceStatus::Disabled
                | std::backtrace::BacktraceStatus::Unsupported => {
                    tracing::error!(
                        thread = thread_name,
                        location = location,
                        message = message,
                        "panic (set RUST_BACKTRACE=1 to capture a backtrace)"
                    );
                }
                _ => {
                    tracing::error!(
                        thread = thread_name,
                        location = location,
                        message = message,
                        "panic"
                    );
                }
            }

            default_hook(info);
        }));
    });
}

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
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, App>, &mut S) -> ui_app_driver::ViewElements,
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
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, App>, &mut S) -> ui_app_driver::ViewElements,
) -> UiAppBootstrapBuilder<S> {
    ui_app_with_app(App::new(), root_name, init_window, view)
}

/// Same as `ui_app`, but allows providing a pre-configured `App`.
#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
pub fn ui_app_with_app<S: 'static>(
    app: App,
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, App>, &mut S) -> ui_app_driver::ViewElements,
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
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, App>, &mut S) -> ui_app_driver::ViewElements,
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
