use fret_core::UiServices;
use fret_icons::IconRegistry;

/// Install this icon pack into a `fret-app` application.
///
/// This registers the pack's icons into the global `IconRegistry`.
pub fn install(app: &mut fret_app::App) {
    app.with_global_mut(IconRegistry::default, |icons, _app| {
        crate::register_icons(icons);
    });
}

/// Install this icon pack at the UI services boundary.
///
/// Icon packs are data-only; UI services are not required. This signature exists to fit the
/// ecosystem installer shape used by `fret-bootstrap`.
pub fn install_with_ui_services(app: &mut fret_app::App, _services: &mut dyn UiServices) {
    let _ = _services;
    install(app);
}
