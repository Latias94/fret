use fret::app::prelude::*;

fn install_marker(_app: &mut App) {}

#[test]
fn app_profile_exports_backend_free_fret_app_authoring_spec() {
    let _app = FretApp::new("backend-free-authoring")
        .minimal_defaults()
        .config_files(false)
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
