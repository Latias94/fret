use fret::app::prelude::*;

fn install_marker(_app: &mut App) {}

#[test]
fn app_profile_exports_backend_free_fret_app_authoring_spec() {
    let _app = FretApp::new("backend-free-authoring")
        .minimal_defaults()
        .config_files(false)
        .asset_startup(
            fret::assets::AssetStartupMode::preferred(),
            fret::assets::AssetStartupPlan::new()
                .development_bundle_dir_if_native(
                    fret::assets::AssetBundleId::app("backend-free-authoring"),
                    "assets",
                )
                .packaged_entries(std::iter::empty::<fret::assets::StaticAssetEntry>()),
        )
        .asset_reload_policy(fret::assets::AssetReloadPolicy::development_default())
        .asset_entries([fret::assets::StaticAssetEntry::new(
            "docs/readme.txt",
            fret::assets::AssetRevision(1),
            b"backend-free",
        )])
        .bundle_asset_entries(
            fret::assets::AssetBundleId::package("demo-kit"),
            std::iter::empty::<fret::assets::StaticAssetEntry>(),
        )
        .embedded_asset_entries(
            fret::assets::AssetBundleId::package("demo-kit"),
            std::iter::empty::<fret::assets::StaticAssetEntry>(),
        )
        .setup(install_marker);
}
