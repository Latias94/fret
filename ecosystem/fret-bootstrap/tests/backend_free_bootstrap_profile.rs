use std::time::Duration;

use fret_bootstrap::assets::{
    AssetBundleId, AssetReloadPolicy, AssetStartupMode, AssetStartupPlan, StaticAssetEntry,
};

#[test]
fn backend_free_profile_exposes_asset_planning_surface() {
    let app_bundle = AssetBundleId::app("profile-test");
    let package_bundle = AssetBundleId::package("profile-test-package");

    let _plan = AssetStartupPlan::new()
        .development_manifest("assets.manifest.json")
        .development_dir("assets")
        .development_bundle_dir(package_bundle.clone(), "package-assets")
        .development_bundle_dir_if_native(app_bundle.clone(), "native-assets")
        .packaged_entries(Vec::<StaticAssetEntry>::new())
        .packaged_bundle_entries(package_bundle.clone(), Vec::<StaticAssetEntry>::new())
        .packaged_embedded_entries(package_bundle, Vec::<StaticAssetEntry>::new());

    let _mode = AssetStartupMode::preferred();
    let _poll = AssetReloadPolicy::poll_metadata(Duration::from_millis(250));
    let _watcher = AssetReloadPolicy::development_default();

    assert_eq!(app_bundle.as_str(), "app:profile-test");
}
