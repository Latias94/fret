use fret_core::UiServices;

use crate::UiAssetsBudgets;

/// Ensure the UI image/SVG caches exist at the UI services boundary.
///
/// This is intentionally small:
/// - ensures the global caches exist,
/// - applies default budgets.
///
/// Driving the image cache state machine remains a responsibility of the app driver via
/// `UiAssets::handle_event(...)`.
pub fn configure_caches_with_ui_services(app: &mut fret_app::App, _services: &mut dyn UiServices) {
    crate::app::configure_caches(app);
}

/// Ensure the UI image/SVG caches exist at the UI services boundary with explicit budgets.
pub fn configure_caches_with_ui_services_and_budgets(
    app: &mut fret_app::App,
    _services: &mut dyn UiServices,
    budgets: UiAssetsBudgets,
) {
    crate::app::configure_caches_with_budgets(app, budgets);
}
