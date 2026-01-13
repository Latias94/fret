use fret_core::UiServices;

use crate::{UiAssets, UiAssetsBudgets};

/// Install `fret-ui-assets` into a `fret-app` application (app-only).
///
/// This variant is useful when you want to run installation during early app initialization
/// (before GPU services are available).
pub fn install_app(app: &mut fret_app::App) {
    install_app_with_budgets(app, UiAssetsBudgets::default());
}

pub fn install_app_with_budgets(app: &mut fret_app::App, budgets: UiAssetsBudgets) {
    UiAssets::configure(app, budgets);
}

/// Install `fret-ui-assets` into a `fret-app` application.
///
/// This is intentionally small:
/// - ensures the global caches exist,
/// - applies default budgets.
///
/// Driving the image cache state machine remains a responsibility of the app driver via
/// `UiAssets::handle_event(...)`.
pub fn install(app: &mut fret_app::App, _services: &mut dyn UiServices) {
    install_app(app);
}

pub fn install_with_budgets(
    app: &mut fret_app::App,
    _services: &mut dyn UiServices,
    budgets: UiAssetsBudgets,
) {
    install_app_with_budgets(app, budgets);
}
