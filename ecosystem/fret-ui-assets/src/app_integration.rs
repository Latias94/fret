use fret_core::UiServices;

use crate::{UiAssets, UiAssetsBudgets};

/// Install `fret-ui-assets` into a `fret-app` application.
///
/// This variant is useful when you want to run installation during early app initialization
/// (before GPU services are available).
pub fn install(app: &mut fret_app::App) {
    install_with_budgets(app, UiAssetsBudgets::default());
}

/// Install `fret-ui-assets` into a `fret-app` application with explicit cache budgets.
pub fn install_with_budgets(app: &mut fret_app::App, budgets: UiAssetsBudgets) {
    UiAssets::configure(app, budgets);
}

/// Install `fret-ui-assets` at the UI services boundary.
///
/// This is intentionally small:
/// - ensures the global caches exist,
/// - applies default budgets.
///
/// Driving the image cache state machine remains a responsibility of the app driver via
/// `UiAssets::handle_event(...)`.
pub fn install_with_ui_services(app: &mut fret_app::App, _services: &mut dyn UiServices) {
    install(app);
}

/// Install `fret-ui-assets` at the UI services boundary with explicit cache budgets.
pub fn install_with_ui_services_and_budgets(
    app: &mut fret_app::App,
    _services: &mut dyn UiServices,
    budgets: UiAssetsBudgets,
) {
    install_with_budgets(app, budgets);
}
