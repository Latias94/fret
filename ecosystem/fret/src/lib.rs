//! Batteries-included desktop-first entry points for Fret.
//!
//! This crate is intentionally **ecosystem-level**:
//! - it composes `fret-bootstrap` (golden-path wiring) with a default component surface,
//! - it enables a practical desktop-first default stack,
//! - it remains optional: advanced users can depend on `fret-framework` + `fret-bootstrap` directly.
//!
//! ## Choosing a native entry path
//!
//! - `fret::App::new(...).window(...).ui(...)?` is the recommended app-author path.
//! - `fret::run_native_with_fn_driver(...)` is the recommended advanced escape hatch when you need
//!   runner-level customization but still want the `fret` defaults/bootstrap story.
//! - `fret::run_native_with_compat_driver(...)` is a compatibility path for existing low-level
//!   integrations that still implement `fret_launch::WinitAppDriver` directly.
//!
//! ## Getting started (desktop)
//!
//! ```no_run
//! use fret::prelude::*;
//!
//! fn init_window(_app: &mut App, _window: AppWindowId) -> () {
//!     ()
//! }
//!
//! fn view<'a>(cx: &mut ElementContext<'a, App>, _st: &mut ()) -> ViewElements {
//!     ui::text("Fret!").into_element(cx).into()
//! }
//!
//! fn main() -> fret::Result<()> {
//!     fret::App::new("hello")
//!         .window("Hello", (560.0, 360.0))
//!         .ui(init_window, view)?
//!         .run()
//! }
//! ```

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
use fret_app::App as KernelApp;

/// Re-export the default shadcn/ui surface as `shadcn`.
#[cfg(feature = "shadcn")]
pub use fret_ui_shadcn as shadcn;

/// Re-export the `IconRegistry` type for app code that wants to install a custom icon pack.
pub use fret_icons::IconRegistry;

/// Re-export `ViewElements` so app code can stay on the `fret` surface.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub use fret_bootstrap::ui_app_driver::ViewElements;

/// Re-export portable action/command identity types for app code and macros.
pub use fret_runtime::{ActionId, ActionMeta, ActionRegistry, CommandId, TypedAction};

pub mod actions;
pub mod view;
pub mod workspace_menu;
pub mod workspace_shell;

pub use workspace_shell::{workspace_shell_model, workspace_shell_model_default_menu};

mod pending_shortcut_overlay;

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
mod app_entry;
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub use app_entry::App;
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub use app_entry::App as AppBuilder;

/// Runtime defaults applied by the `fret` facade (within the enabled crate features).
///
/// This is an ecosystem-level convenience (not a kernel contract).
#[derive(Debug, Clone, Copy)]
pub struct Defaults {
    /// Enable default diagnostics wiring (tracing + panic hook).
    pub diagnostics: bool,
    /// Enable layered `.fret/*` config file loading (settings/keymap/menubar).
    pub config_files: bool,
    /// Install the default shadcn integration into the app.
    pub shadcn: bool,
    /// Install UI asset caches (images/SVG) with budgets.
    pub ui_assets: bool,
    /// Optional override budgets for UI assets.
    pub ui_assets_budgets: Option<(u64, usize, u64, usize)>,
    /// Install built-in icon packs (controlled by crate features).
    pub icons: bool,
    /// Preload icon SVGs on GPU ready (controlled by crate features).
    pub preload_icon_svgs: bool,
}

impl Defaults {
    /// Recommended desktop-first “batteries included” defaults.
    pub const fn desktop_batteries() -> Self {
        Self {
            diagnostics: true,
            config_files: true,
            shadcn: true,
            ui_assets: true,
            ui_assets_budgets: None,
            icons: true,
            preload_icon_svgs: true,
        }
    }

    /// Recommended desktop-first defaults for app authors.
    ///
    /// These defaults are intended to be smooth and practical without pulling in every optional
    /// integration. In particular, they avoid UI assets caches and GPU-time SVG preloading unless
    /// explicitly enabled.
    pub const fn desktop_app() -> Self {
        Self {
            diagnostics: true,
            config_files: false,
            shadcn: true,
            ui_assets: false,
            ui_assets_budgets: None,
            icons: false,
            preload_icon_svgs: false,
        }
    }

    /// Minimal defaults that avoid filesystem config loading and other batteries.
    pub const fn minimal() -> Self {
        Self {
            diagnostics: false,
            config_files: false,
            shadcn: false,
            ui_assets: false,
            ui_assets_budgets: None,
            icons: false,
            preload_icon_svgs: false,
        }
    }

    pub const fn with_ui_assets_budgets(
        mut self,
        image_budget_bytes: u64,
        image_max_ready_entries: usize,
        svg_budget_bytes: u64,
        svg_max_ready_entries: usize,
    ) -> Self {
        self.ui_assets = true;
        self.ui_assets_budgets = Some((
            image_budget_bytes,
            image_max_ready_entries,
            svg_budget_bytes,
            svg_max_ready_entries,
        ));
        self
    }
}

impl Default for Defaults {
    fn default() -> Self {
        Self::desktop_app()
    }
}

/// Interop helpers for embedding foreign UI as isolated surfaces (desktop builds).
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub mod interop;

/// Dev-only helpers (hotpatch/dev-state) for iteration workflows.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop", feature = "devloop"))]
pub mod dev {
    pub use fret_launch::dev_state::{
        DevStateExport, DevStateHook, DevStateHooks, DevStateSnapshot, DevStateWindowKeyRegistry,
    };
}

/// Re-export the kernel facade (desktop builds).
#[cfg(feature = "desktop")]
pub use fret_framework as kernel;

/// Common imports for application code using `fret`.
///
/// Recommended: `use fret::prelude::*;`
pub mod prelude {
    pub use fret_ui_kit::prelude::*;

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub use crate::AppBuilder;

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub use crate::App as FretApp;

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub use crate::interop::embedded_viewport::{
        EmbeddedViewportForeignUiAppDriverExt, EmbeddedViewportUiAppDriverExt,
    };

    pub use crate::workspace_menu::{
        InWindowMenubarFocusHandle, MenubarFromRuntimeOptions, menubar_from_runtime,
        menubar_from_runtime_with_focus_handle,
    };

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub use crate::ViewElements;
    pub use crate::view::{View, ViewCx};
    pub use fret_app::{App, Effect};
    pub use fret_core::{Event, SemanticsRole};
    pub use fret_ui::ThemeSnapshot;
    pub use fret_ui::element::{Elements, HoverRegionProps, Length, SemanticsProps};

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub use crate::interop;

    #[cfg(feature = "shadcn")]
    pub use crate::shadcn;

    #[cfg(feature = "state-query")]
    pub use fret_query::{CancellationToken, QueryError, QueryHandle, QueryKey, QueryPolicy};
}

#[derive(Debug, thiserror::Error)]
/// Public error type for the `fret` facade.
pub enum Error {
    #[error(transparent)]
    Bootstrap(#[from] BootstrapError),
    #[error(transparent)]
    Runner(#[from] RunnerError),
}

/// Result type used by the `fret` facade.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BootstrapError(#[from] fret_bootstrap::BootstrapError);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct RunnerError(#[from] fret_launch::RunnerError);

/// A `UiAppDriver` wrapper used by `fret` to avoid exposing `fret-bootstrap` types in signatures.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub struct UiAppDriver<S> {
    inner: fret_bootstrap::ui_app_driver::UiAppDriver<S>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl<S> UiAppDriver<S> {
    pub(crate) fn new(inner: fret_bootstrap::ui_app_driver::UiAppDriver<S>) -> Self {
        Self { inner }
    }

    pub(crate) fn into_inner(self) -> fret_bootstrap::ui_app_driver::UiAppDriver<S> {
        self.inner
    }

    pub fn close_on_window_close_requested(mut self, enabled: bool) -> Self {
        self.inner = self.inner.close_on_window_close_requested(enabled);
        self
    }

    #[cfg(feature = "ui-assets")]
    pub fn drive_ui_assets(mut self, enabled: bool) -> Self {
        self.inner = self.inner.drive_ui_assets(enabled);
        self
    }

    pub fn on_event(
        mut self,
        f: fn(
            &mut KernelApp,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &fret_core::Event,
        ),
    ) -> Self {
        self.inner = self.inner.on_event(f);
        self
    }

    pub fn on_command(
        mut self,
        f: fn(
            &mut KernelApp,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &fret_runtime::CommandId,
        ),
    ) -> Self {
        self.inner = self.inner.on_command(f);
        self
    }

    pub fn on_preferences(
        mut self,
        f: fn(
            &mut KernelApp,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
        ),
    ) -> Self {
        self.inner = self.inner.on_preferences(f);
        self
    }

    pub fn on_hot_reload_window(
        mut self,
        f: fn(
            &mut KernelApp,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
        ),
    ) -> Self {
        self.inner = self.inner.on_hot_reload_window(f);
        self
    }

    pub fn on_model_changes(
        mut self,
        f: fn(
            &mut KernelApp,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &[fret_app::ModelId],
        ),
    ) -> Self {
        self.inner = self.inner.on_model_changes(f);
        self
    }

    pub fn on_global_changes(
        mut self,
        f: fn(
            &mut KernelApp,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &[std::any::TypeId],
        ),
    ) -> Self {
        self.inner = self.inner.on_global_changes(f);
        self
    }

    pub fn window_create_spec(
        mut self,
        f: fn(
            &mut KernelApp,
            &fret_app::CreateWindowRequest,
        ) -> Option<fret_launch::WindowCreateSpec>,
    ) -> Self {
        self.inner = self.inner.window_create_spec(f);
        self
    }

    pub fn window_created(
        mut self,
        f: fn(&mut KernelApp, &fret_app::CreateWindowRequest, fret_core::AppWindowId),
    ) -> Self {
        self.inner = self.inner.window_created(f);
        self
    }

    pub fn before_close_window(
        mut self,
        f: fn(&mut KernelApp, fret_core::AppWindowId) -> bool,
    ) -> Self {
        self.inner = self.inner.before_close_window(f);
        self
    }

    pub fn handle_global_command(
        mut self,
        f: fn(&mut KernelApp, &mut dyn fret_core::UiServices, fret_runtime::CommandId),
    ) -> Self {
        self.inner = self.inner.handle_global_command(f);
        self
    }

    pub fn viewport_input(mut self, f: fn(&mut KernelApp, fret_core::ViewportInputEvent)) -> Self {
        self.inner = self.inner.viewport_input(f);
        self
    }

    pub fn record_engine_frame(
        mut self,
        f: fn(
            &mut KernelApp,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &crate::kernel::render::WgpuContext,
            &mut crate::kernel::render::Renderer,
            f32,
            fret_runtime::TickId,
            fret_runtime::FrameId,
        ) -> fret_launch::EngineFrameUpdate,
    ) -> Self {
        self.inner = self.inner.record_engine_frame(f);
        self
    }

    pub fn dock_op(mut self, f: fn(&mut KernelApp, fret_core::DockOp)) -> Self {
        self.inner = self.inner.dock_op(f);
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn command_palette(mut self, enabled: bool) -> Self {
        self.inner = self.inner.command_palette(enabled);
        self
    }
}

/// A `UiAppBuilder` wrapper used by `fret` to avoid exposing `fret-bootstrap` types in signatures.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub struct UiAppBuilder<S> {
    inner: fret_bootstrap::UiAppBootstrapBuilder<S>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl<S: 'static> UiAppBuilder<S> {
    pub(crate) fn from_bootstrap(inner: fret_bootstrap::UiAppBootstrapBuilder<S>) -> Self {
        Self { inner }
    }

    pub fn with_main_window(self, title: impl Into<String>, size: (f64, f64)) -> Self {
        Self {
            inner: self.inner.with_main_window(title, size),
        }
    }

    pub fn configure(self, f: impl FnOnce(&mut fret_launch::WinitRunnerConfig)) -> Self {
        Self {
            inner: self.inner.configure(f),
        }
    }

    pub fn init_app(self, f: impl FnOnce(&mut KernelApp)) -> Self {
        Self {
            inner: self.inner.init_app(f),
        }
    }

    pub fn install_app(self, install: fn(&mut KernelApp)) -> Self {
        Self {
            inner: self.inner.install_app(install),
        }
    }

    pub fn install(self, install: fn(&mut KernelApp, &mut dyn fret_core::UiServices)) -> Self {
        Self {
            inner: self.inner.install(install),
        }
    }

    /// Install custom GPU effects at the renderer boundary (ADR 0299).
    ///
    /// Note: the callback receives the **kernel** app type (`fret_app::App`, re-exported here as
    /// `KernelApp`), not the `fret::App` builder-chain facade.
    pub fn install_custom_effects(
        self,
        install: fn(&mut KernelApp, &mut dyn fret_core::CustomEffectService),
    ) -> Self {
        Self {
            inner: self.inner.install_custom_effects(install),
        }
    }

    pub fn register_icon_pack(self, register: fn(&mut IconRegistry)) -> Self {
        Self {
            inner: self.inner.register_icon_pack(register),
        }
    }

    #[cfg(feature = "ui-assets")]
    pub fn with_ui_assets_budgets(
        self,
        image_budget_bytes: u64,
        image_max_ready_entries: usize,
        svg_budget_bytes: u64,
        svg_max_ready_entries: usize,
    ) -> Self {
        Self {
            inner: self.inner.with_ui_assets_budgets(
                image_budget_bytes,
                image_max_ready_entries,
                svg_budget_bytes,
                svg_max_ready_entries,
            ),
        }
    }

    pub fn on_gpu_ready(
        self,
        f: impl FnOnce(
            &mut KernelApp,
            &crate::kernel::render::WgpuContext,
            &mut crate::kernel::render::Renderer,
        ) + 'static,
    ) -> Self {
        Self {
            inner: self.inner.on_gpu_ready(f),
        }
    }

    pub fn run(self) -> Result<()> {
        self.inner.run().map_err(RunnerError::from)?;
        Ok(())
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub(crate) fn apply_desktop_defaults_with<D: fret_launch::WinitAppDriver + 'static>(
    builder: fret_bootstrap::BootstrapBuilder<D>,
    defaults: Defaults,
) -> std::result::Result<fret_bootstrap::BootstrapBuilder<D>, fret_bootstrap::BootstrapError> {
    // Always ensure an i18n backend exists unless the app provides one.
    let builder = builder.init_app(fret_bootstrap::install_default_i18n_backend);
    let _ = defaults;

    #[cfg(feature = "diagnostics")]
    let builder = if defaults.diagnostics {
        builder.with_default_diagnostics()
    } else {
        builder
    };

    #[cfg(feature = "config-files")]
    let builder = if defaults.config_files {
        builder.with_default_config_files()?
    } else {
        builder.with_command_default_keybindings()
    };

    #[cfg(not(feature = "config-files"))]
    let builder = builder.with_command_default_keybindings();

    #[cfg(feature = "shadcn")]
    let builder = if defaults.shadcn {
        builder.install_app(fret_ui_shadcn::install_app)
    } else {
        builder
    };

    #[cfg(feature = "ui-assets")]
    let builder = if defaults.ui_assets {
        let (image_budget_bytes, image_max_ready_entries, svg_budget_bytes, svg_max_ready_entries) =
            defaults
                .ui_assets_budgets
                .unwrap_or((64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096));
        builder.with_ui_assets_budgets(
            image_budget_bytes,
            image_max_ready_entries,
            svg_budget_bytes,
            svg_max_ready_entries,
        )
    } else {
        builder
    };

    #[cfg(feature = "icons")]
    let builder = if defaults.icons {
        builder.with_lucide_icons()
    } else {
        builder
    };

    #[cfg(feature = "preload-icon-svgs")]
    let builder = if defaults.preload_icon_svgs {
        builder.preload_icon_svgs_on_gpu_ready()
    } else {
        builder
    };

    Ok(builder)
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub(crate) fn apply_desktop_defaults<D: fret_launch::WinitAppDriver + 'static>(
    builder: fret_bootstrap::BootstrapBuilder<D>,
) -> std::result::Result<fret_bootstrap::BootstrapBuilder<D>, fret_bootstrap::BootstrapError> {
    apply_desktop_defaults_with(builder, Defaults::default())
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub(crate) fn ui_bootstrap_builder_with_hooks<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut KernelApp, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, KernelApp>, &mut S) -> ViewElements,
    configure: fn(UiAppDriver<S>) -> UiAppDriver<S>,
) -> fret_bootstrap::UiAppBootstrapBuilder<S> {
    let driver = fret_bootstrap::ui_app_driver::UiAppDriver::new(root_name, init_window, view)
        .on_preferences(fret_bootstrap::ui_app_driver::default_on_preferences::<S>);
    #[cfg(feature = "shadcn")]
    let driver = driver
        .on_global_changes_middleware(shadcn_sync_theme_from_environment_on_global_changes::<S>);
    let driver = configure(UiAppDriver::new(driver)).into_inner();
    let builder = fret_bootstrap::BootstrapBuilder::new(KernelApp::new(), driver.into_fn_driver());

    builder
}

/// Run a native desktop app using the compatibility driver path.
///
/// Prefer `fret::App` / `UiAppBuilder` for general applications and
/// `run_native_with_fn_driver(...)` for new advanced integrations. This helper exists for
/// low-level integrations that still implement `fret_launch::WinitAppDriver` directly while
/// wanting the higher-level defaults/bootstrap story from `fret`.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn run_native_with_compat_driver<D: fret_launch::WinitAppDriver + 'static>(
    config: fret_launch::WinitRunnerConfig,
    app: KernelApp,
    driver: D,
) -> Result<()> {
    let builder = fret_bootstrap::BootstrapBuilder::new(app, driver).configure(move |c| {
        *c = config;
    });

    let builder = apply_desktop_defaults(builder).map_err(BootstrapError::from)?;

    builder.run().map_err(RunnerError::from)?;
    Ok(())
}

/// Run a native desktop app using the advanced `FnDriver` escape hatch.
///
/// This is the recommended low-level path when the app wants the `fret` bootstrap/defaults story
/// but needs runner-level customization without teaching `WinitAppDriver` as the primary model.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn run_native_with_fn_driver<D: 'static, S: 'static>(
    config: fret_launch::WinitRunnerConfig,
    app: KernelApp,
    driver_state: D,
    create_window_state: fn(&mut D, &mut KernelApp, fret_core::AppWindowId) -> S,
    handle_event: for<'d, 'cx, 'e> fn(
        &'d mut D,
        fret_launch::WinitEventContext<'cx, S>,
        &'e fret_core::Event,
    ),
    render: for<'d, 'cx> fn(&'d mut D, fret_launch::WinitRenderContext<'cx, S>),
) -> Result<()> {
    let builder = fret_bootstrap::BootstrapBuilder::new_fn(
        app,
        driver_state,
        create_window_state,
        handle_event,
        render,
    )
    .configure(move |c| {
        *c = config;
    });

    let builder = apply_desktop_defaults(builder).map_err(BootstrapError::from)?;

    builder.run().map_err(RunnerError::from)?;
    Ok(())
}

/// Compatibility shorthand for [`App::new(...).ui_with_hooks(...)`](crate::App::ui_with_hooks).
///
/// Prefer the builder chain for new code so window/defaults/app-install configuration stays in one
/// place.
///
/// Create a desktop-first UI app builder with conservative defaults applied.
///
/// Defaults (when the corresponding features are enabled):
/// - diagnostics (`diagnostics`)
/// - layered config files (`.fret/settings.json`, `.fret/keymap.json`, `.fret/menubar.json`)
/// - shadcn app integration (`shadcn`)
/// - icon pack installation + optional SVG preloading
/// - UI assets caches with default budgets (`ui-assets`)
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn app_with_hooks<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut KernelApp, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, KernelApp>, &mut S) -> ViewElements,
    configure: fn(UiAppDriver<S>) -> UiAppDriver<S>,
) -> Result<UiAppBuilder<S>> {
    let builder = ui_bootstrap_builder_with_hooks(root_name, init_window, view, configure);
    let builder =
        apply_desktop_defaults_with(builder, Defaults::default()).map_err(BootstrapError::from)?;
    Ok(UiAppBuilder::from_bootstrap(builder))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop", feature = "shadcn"))]
fn shadcn_sync_theme_from_environment_on_global_changes<S>(
    app: &mut KernelApp,
    window: fret_core::AppWindowId,
    _ui: &mut fret_ui::UiTree<KernelApp>,
    _st: &mut S,
    changed: &[std::any::TypeId],
) {
    if !changed.contains(&std::any::TypeId::of::<fret_core::WindowMetricsService>()) {
        return;
    }
    let config = app
        .global::<fret_ui_shadcn::ShadcnInstallConfig>()
        .copied()
        .unwrap_or_default();
    let _ =
        fret_ui_shadcn::sync_theme_from_environment(app, window, config.base_color, config.scheme);
}

/// Compatibility shorthand for [`App::new(...).ui(...)`](crate::App::ui).
///
/// Prefer the builder chain for new code.
///
/// Same as [`app_with_hooks`], but without a driver configuration hook.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn app<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut KernelApp, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, KernelApp>, &mut S) -> ViewElements,
) -> Result<UiAppBuilder<S>> {
    app_with_hooks(root_name, init_window, view, |d| d)
}

/// Compatibility shorthand for [`App::run_ui_with_hooks`](crate::App::run_ui_with_hooks).
///
/// Prefer the builder chain for new code.
///
/// Run a desktop-first UI app using default window settings.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn run_with_hooks<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut KernelApp, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, KernelApp>, &mut S) -> ViewElements,
    configure: fn(UiAppDriver<S>) -> UiAppDriver<S>,
) -> Result<()> {
    app_with_hooks(root_name, init_window, view, configure)?
        .with_main_window(root_name, (960.0, 720.0))
        .run()
}

/// Compatibility shorthand for [`App::run_ui`](crate::App::run_ui).
///
/// Prefer the builder chain for new code.
///
/// Run a desktop-first UI app using default window settings.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn run<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut KernelApp, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, KernelApp>, &mut S) -> ViewElements,
) -> Result<()> {
    run_with_hooks(root_name, init_window, view, |d| d)
}

#[cfg(all(
    test,
    not(target_arch = "wasm32"),
    feature = "desktop",
    feature = "shadcn"
))]
mod tests {
    use std::any::TypeId;

    use fret_core::{AppWindowId, ColorScheme, WindowMetricsService};
    use fret_ui::{Theme, UiTree};

    use super::KernelApp;

    #[test]
    fn shadcn_auto_theme_middleware_reacts_to_window_metrics() {
        let mut app = KernelApp::new();
        fret_ui_shadcn::install_app(&mut app);

        let window = AppWindowId::from(slotmap::KeyData::from_ffi(1));
        app.with_global_mut(WindowMetricsService::default, |svc, _app| {
            svc.set_color_scheme(window, Some(ColorScheme::Dark));
        });

        let mut ui = UiTree::<KernelApp>::default();
        let mut state = ();

        let before_bg = Theme::global(&app).colors.surface_background;
        let before_rev = Theme::global(&app).revision();

        super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
            &mut app,
            window,
            &mut ui,
            &mut state,
            &[],
        );

        assert_eq!(Theme::global(&app).revision(), before_rev);
        assert_eq!(Theme::global(&app).colors.surface_background, before_bg);

        super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
            &mut app,
            window,
            &mut ui,
            &mut state,
            &[TypeId::of::<WindowMetricsService>()],
        );

        assert_ne!(Theme::global(&app).colors.surface_background, before_bg);
        let rev_after = Theme::global(&app).revision();

        super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
            &mut app,
            window,
            &mut ui,
            &mut state,
            &[TypeId::of::<WindowMetricsService>()],
        );

        assert_eq!(Theme::global(&app).revision(), rev_after);
    }
}
