use crate::{UiAssets, UiAssetsBudgets};

/// Ensure the UI image/SVG caches exist and apply default budgets.
///
/// This does **not** wire the event pipeline. Apps/bootstrap code must still drive
/// `UiAssets::handle_event(...)` when image readiness events need to update UI state.
pub fn configure_caches(app: &mut fret_app::App) {
    configure_caches_with_budgets(app, UiAssetsBudgets::default());
}

/// Ensure the UI image/SVG caches exist and apply explicit budgets.
pub fn configure_caches_with_budgets(app: &mut fret_app::App, budgets: UiAssetsBudgets) {
    UiAssets::configure(app, budgets);
}
