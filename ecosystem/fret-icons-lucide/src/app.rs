use fret_icons::{IconRegistry, InstalledIconPacks};

/// Install this icon pack into a `fret-app` application.
///
/// This registers the pack's icons into the global `IconRegistry`.
pub fn install(app: &mut fret_app::App) {
    app.with_global_mut(IconRegistry::default, |icons, app| {
        crate::PACK.register_into_registry(icons);
        let frozen = icons.freeze().unwrap_or_else(|errors| {
            panic!("failed to freeze icon registry in fret_icons_lucide.app.install: {errors:?}")
        });
        app.set_global(frozen);
    });
    app.with_global_mut(InstalledIconPacks::default, |installed, _app| {
        installed.record(crate::PACK_METADATA).unwrap_or_else(|err| {
            panic!("failed to record installed icon pack metadata in fret_icons_lucide.app.install: {err}")
        });
    });
}
