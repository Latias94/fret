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
