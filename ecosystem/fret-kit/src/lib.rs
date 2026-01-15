//! Batteries-included desktop-first entry points for Fret.
//!
//! This crate is intentionally **ecosystem-level**:
//! - it composes `fret-bootstrap` (golden-path wiring) with a default component surface,
//! - it enables a practical desktop-first default stack,
//! - it remains optional: advanced users can depend on `fret` + `fret-bootstrap` directly.

#[cfg(all(feature = "icons-lucide", feature = "icons-radix"))]
compile_error!("`fret-kit` features `icons-lucide` and `icons-radix` are mutually exclusive.");

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
use fret_app::App;

/// Re-export the default shadcn/ui surface as `shadcn`.
pub use fret_ui_shadcn as shadcn;

/// Re-export the underlying `fret` facade (desktop builds).
#[cfg(feature = "desktop")]
pub use fret;

/// Common imports for application code using `fret-kit`.
///
/// Recommended: `use fret_kit::prelude::*;`
pub mod prelude {
    pub use crate::shadcn;
    pub use crate::shadcn::prelude::*;

    pub use fret_app::App;
    pub use fret_runtime::CommandId;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Bootstrap(#[from] fret_bootstrap::BootstrapError),
    #[error(transparent)]
    Runner(#[from] fret_launch::RunnerError),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Create a desktop-first UI app builder with conservative defaults applied.
///
/// Defaults (when the corresponding features are enabled):
/// - diagnostics (`diagnostics`)
/// - layered config files (`.fret/settings.json`, `.fret/keymap.json`)
/// - shadcn app integration
/// - icon pack installation + optional SVG preloading
/// - UI assets caches with default budgets (`ui-assets`)
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn app_with_hooks<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(
        &mut fret_ui::ElementContext<'a, App>,
        &mut S,
    ) -> Vec<fret_ui::element::AnyElement>,
    configure: fn(
        fret_bootstrap::ui_app_driver::UiAppDriver<S>,
    ) -> fret_bootstrap::ui_app_driver::UiAppDriver<S>,
) -> Result<fret_bootstrap::UiAppBootstrapBuilder<S>> {
    let builder = fret_bootstrap::ui_app_with_hooks(root_name, init_window, view, configure);

    #[cfg(feature = "diagnostics")]
    let builder = builder.with_default_diagnostics();

    let builder = builder.with_default_config_files()?;

    let builder = builder.install_app(fret_ui_shadcn::install_app);

    #[cfg(feature = "ui-assets")]
    let builder = builder.with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096);

    #[cfg(feature = "icons-lucide")]
    let builder = builder.with_lucide_icons();

    #[cfg(feature = "icons-radix")]
    let builder = builder.with_radix_icons();

    #[cfg(feature = "preload-icon-svgs")]
    let builder = builder.preload_icon_svgs_on_gpu_ready();

    Ok(builder)
}

/// Same as [`app_with_hooks`], but without a driver configuration hook.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn app<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(
        &mut fret_ui::ElementContext<'a, App>,
        &mut S,
    ) -> Vec<fret_ui::element::AnyElement>,
) -> Result<fret_bootstrap::UiAppBootstrapBuilder<S>> {
    app_with_hooks(root_name, init_window, view, |d| d)
}

/// Run a desktop-first UI app using default window settings.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn run_with_hooks<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(
        &mut fret_ui::ElementContext<'a, App>,
        &mut S,
    ) -> Vec<fret_ui::element::AnyElement>,
    configure: fn(
        fret_bootstrap::ui_app_driver::UiAppDriver<S>,
    ) -> fret_bootstrap::ui_app_driver::UiAppDriver<S>,
) -> Result<()> {
    app_with_hooks(root_name, init_window, view, configure)?
        .with_main_window(root_name, (960.0, 720.0))
        .run()?;
    Ok(())
}

/// Run a desktop-first UI app using default window settings.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn run<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(
        &mut fret_ui::ElementContext<'a, App>,
        &mut S,
    ) -> Vec<fret_ui::element::AnyElement>,
) -> Result<()> {
    run_with_hooks(root_name, init_window, view, |d| d)
}
