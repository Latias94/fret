use fret_core::UiServices;

/// Install this icon pack at the UI services boundary.
///
/// Icon packs are data-only; UI services are not required. This signature exists to fit the
/// ecosystem installer shape used by `fret-bootstrap`.
pub fn install_with_ui_services(app: &mut fret_app::App, _services: &mut dyn UiServices) {
    let _ = _services;
    crate::app::install(app);
}
