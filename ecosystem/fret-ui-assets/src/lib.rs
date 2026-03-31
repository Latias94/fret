//! UI render asset conveniences (Image/SVG caches and upload helpers).
//!
//! This crate provides small caching layers for common UI render assets (images, SVGs) so
//! components can avoid repeated decode/raster/upload work across frames.
//!
//! This is an ecosystem crate: it composes higher-level policies on top of the core runtime
//! services. See ADR 0106.
//!
//! URL image note:
//! - `ImageSource::from_url(...)` is a direct helper for URL-backed image loads on every platform.
//! - For logical asset requests (`resolve_image_source*`), the shared image bridge can now consume
//!   resolver-provided `AssetExternalReference::Url` on every platform.
//! - The shipped desktop host still does not install a first-party default URL resolver; desktop
//!   apps must opt in with a custom resolver if they want URL assets.
//! - Web/WASM only gets a browser-native URL lane when the winning resolver layer returns
//!   `AssetExternalReference::Url`.
//! - Otherwise the current first-party web path falls back to resolving bytes and decoding from
//!   `ResolvedAssetBytes`, which can cost more CPU and memory than a browser-native decode path on
//!   image-heavy surfaces.

pub mod asset_resolver;
pub mod image_asset_cache;
pub mod image_asset_state;
pub mod image_source;
pub mod image_upload;
pub mod svg_asset_cache;
pub mod svg_asset_state;
pub mod ui_assets;

#[cfg(feature = "ui")]
pub mod ui;

pub use asset_resolver::*;
pub use image_asset_cache::*;
pub use image_source::*;
pub use image_upload::*;
pub use svg_asset_cache::*;
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
    const UI_RS: &str = include_str!("ui.rs");
    const ASSET_RESOLVER_RS: &str = include_str!("asset_resolver.rs");

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
    fn legacy_public_path_helpers_and_svg_file_shims_are_deleted() {
        assert!(IMAGE_SOURCE_RS.contains("pub(crate) fn from_native_file_path("));
        assert!(!IMAGE_SOURCE_RS.contains("pub fn from_file_path("));
        assert!(!IMAGE_SOURCE_RS.contains("pub fn from_path("));
        assert!(!public_surface().contains("pub mod svg_file;"));
        assert!(!public_surface().contains("pub use svg_file::*;"));
        assert!(!ASSET_RESOLVER_RS.contains("pub fn resolve_svg_file_source("));
        assert!(!ASSET_RESOLVER_RS.contains("pub fn resolve_svg_file_source_from_host("));
        assert!(!UI_RS.contains("pub trait SvgFileElementContextExt"));
    }

    #[test]
    fn cache_setup_surface_keeps_only_explicit_configuration_names() {
        assert!(APP_RS.contains("pub fn configure_caches(app: &mut fret_app::App)"));
        assert!(APP_RS.contains("pub fn configure_caches_with_budgets("));
        assert!(!APP_RS.contains("pub fn install(app: &mut fret_app::App)"));
        assert!(!APP_RS.contains("pub fn install_with_budgets("));

        assert!(ADVANCED_RS.contains("pub fn configure_caches_with_ui_services("));
        assert!(ADVANCED_RS.contains("pub fn configure_caches_with_ui_services_and_budgets("));
        assert!(!ADVANCED_RS.contains("pub fn install_with_ui_services("));
        assert!(!ADVANCED_RS.contains("pub fn install_with_ui_services_and_budgets("));
    }

    #[test]
    fn reload_surface_is_not_reexported_from_ui_assets() {
        let public_surface = public_surface();
        assert!(!public_surface.contains("pub mod reload;"));
        assert!(!public_surface.contains("pub use reload::*;"));
        assert!(!public_surface.contains("AssetReloadEpoch"));
        assert!(!public_surface.contains("bump_asset_reload_epoch"));
    }
}
