use fret_icons::{IconRegistry, InstalledIconPacks};

/// Install this icon pack into a `fret-app` application.
///
/// This registers the pack's icons into the global `IconRegistry`.
pub fn install(app: &mut fret_app::App) {
    app.with_global_mut(IconRegistry::default, |icons, app| {
        crate::PACK.register_into_registry(icons);
        let frozen = icons.freeze_or_default_with_context("fret_icons_radix.app.install");
        app.set_global(frozen);
    });
    app.with_global_mut(InstalledIconPacks::default, |installed, _app| {
        installed.record(crate::PACK_METADATA);
    });
}
