//! UI render asset conveniences (Image/SVG caches and upload helpers).
//!
//! This crate provides small caching layers for common UI render assets (images, SVGs) so
//! components can avoid repeated decode/raster/upload work across frames.
//!
//! This is an ecosystem crate: it composes higher-level policies on top of the core runtime
//! services. See ADR 0106.

pub mod asset_resolver;
pub mod image_asset_cache;
pub mod image_asset_state;
pub mod image_source;
pub mod image_upload;
pub mod reload;
pub mod svg_asset_cache;
pub mod svg_asset_state;
pub mod svg_file;
pub mod ui_assets;

#[cfg(feature = "ui")]
pub mod ui;

pub use asset_resolver::*;
pub use image_asset_cache::*;
pub use image_source::*;
pub use image_upload::*;
pub use reload::*;
pub use svg_asset_cache::*;
pub use svg_file::*;
pub use ui_assets::*;

#[cfg(feature = "app-integration")]
pub mod advanced;
#[cfg(feature = "app-integration")]
pub mod app;

#[cfg(test)]
mod surface_policy_tests {
    const LIB_RS: &str = include_str!("lib.rs");
    const APP_RS: &str = include_str!("app.rs");
    const ADVANCED_RS: &str = include_str!("advanced.rs");
    const IMAGE_SOURCE_RS: &str = include_str!("image_source.rs");
    const RELOAD_RS: &str = include_str!("reload.rs");
    const SVG_FILE_RS: &str = include_str!("svg_file.rs");

    fn public_surface() -> &'static str {
        LIB_RS.split("#[cfg(test)]").next().unwrap_or(LIB_RS)
    }

    #[test]
    fn app_integration_stays_under_explicit_app_module() {
        let public_surface = public_surface();
        assert!(public_surface.contains("pub mod app;"));
        assert!(public_surface.contains("pub mod advanced;"));
        assert!(!public_surface.contains("pub use app::"));
        assert!(!public_surface.contains("pub use advanced::"));
        assert!(!public_surface.contains("pub fn configure_caches("));
        assert!(!public_surface.contains("pub fn configure_caches_with_budgets("));
        assert!(!public_surface.contains("pub fn configure_caches_with_ui_services("));
        assert!(!public_surface.contains("pub fn configure_caches_with_ui_services_and_budgets("));
        assert!(APP_RS.contains("pub fn configure_caches(app: &mut fret_app::App)"));
        assert!(APP_RS.contains("pub fn configure_caches_with_budgets("));
        assert!(!APP_RS.contains("configure_caches_with_ui_services"));
        assert!(ADVANCED_RS.contains("pub fn configure_caches_with_ui_services("));
        assert!(ADVANCED_RS.contains("pub fn configure_caches_with_ui_services_and_budgets("));
    }

    #[test]
    fn legacy_public_path_helpers_are_deleted_while_internal_native_bridges_remain() {
        for source in [IMAGE_SOURCE_RS, SVG_FILE_RS] {
            assert!(source.contains("pub(crate) fn from_native_file_path("));
            assert!(!source.contains("pub fn from_file_path("));
            assert!(!source.contains("pub fn from_path("));
        }
    }

    #[test]
    fn legacy_install_aliases_stay_deprecated_and_point_to_explicit_cache_setup() {
        assert!(APP_RS.contains("pub fn configure_caches(app: &mut fret_app::App)"));
        assert!(APP_RS.contains("pub fn configure_caches_with_budgets("));
        assert!(APP_RS.contains("pub fn install(app: &mut fret_app::App)"));
        assert!(APP_RS.contains("pub fn install_with_budgets("));
        assert!(APP_RS.contains(
            "use configure_caches; this only configures caches and does not wire event handling"
        ));
        assert!(
            APP_RS.contains(
                "use configure_caches_with_budgets; this only configures caches and does not wire event handling"
            )
        );

        assert!(ADVANCED_RS.contains("pub fn configure_caches_with_ui_services("));
        assert!(ADVANCED_RS.contains("pub fn configure_caches_with_ui_services_and_budgets("));
        assert!(ADVANCED_RS.contains("pub fn install_with_ui_services("));
        assert!(ADVANCED_RS.contains("pub fn install_with_ui_services_and_budgets("));
        assert!(
            ADVANCED_RS.contains(
                "use configure_caches_with_ui_services; this only configures caches and does not wire event handling"
            )
        );
        assert!(
            ADVANCED_RS.contains(
                "use configure_caches_with_ui_services_and_budgets; this only configures caches and does not wire event handling"
            )
        );
    }

    #[test]
    fn reload_surface_keeps_only_generic_asset_reload_names() {
        assert!(RELOAD_RS.contains("AssetReloadEpoch"));
        assert!(RELOAD_RS.contains("bump_asset_reload_epoch"));
        assert!(!RELOAD_RS.contains("UiAssetsReloadEpoch"));
        assert!(!RELOAD_RS.contains("bump_ui_assets_reload_epoch"));
    }
}
