use fret_icons::{
    IconRegistry, InstalledIconPacks, panic_on_icon_pack_metadata_conflict,
    panic_on_icon_registry_freeze_failure,
};

/// Install this icon pack into a `fret-app` application.
///
/// This registers the pack's icons into the global `IconRegistry`.
pub fn install(app: &mut fret_app::App) {
    app.with_global_mut(IconRegistry::default, |icons, app| {
        crate::PACK.register_into_registry(icons);
        let frozen = icons.freeze().unwrap_or_else(|errors| {
            panic_on_icon_registry_freeze_failure(
                "fret_icons_lucide.app.install",
                Some(crate::PACK_METADATA.pack_id),
                errors,
            )
        });
        app.set_global(frozen);
    });
    app.with_global_mut(InstalledIconPacks::default, |installed, _app| {
        installed
            .record(crate::PACK_METADATA)
            .unwrap_or_else(|err| {
                panic_on_icon_pack_metadata_conflict("fret_icons_lucide.app.install", err)
            });
    });
}
