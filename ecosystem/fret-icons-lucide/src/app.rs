use fret_icons::IconRegistry;

/// Install this icon pack into a `fret-app` application.
///
/// This registers the pack's icons into the global `IconRegistry`.
pub fn install(app: &mut fret_app::App) {
    app.with_global_mut(IconRegistry::default, |icons, _app| {
        crate::register_icons(icons);
    });
}
