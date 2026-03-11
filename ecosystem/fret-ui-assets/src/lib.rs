//! UI render asset conveniences (Image/SVG caches and upload helpers).
//!
//! This crate provides small caching layers for common UI render assets (images, SVGs) so
//! components can avoid repeated decode/raster/upload work across frames.
//!
//! This is an ecosystem crate: it composes higher-level policies on top of the core runtime
//! services. See ADR 0106.

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

    fn public_surface() -> &'static str {
        LIB_RS.split("#[cfg(test)]").next().unwrap_or(LIB_RS)
    }

    #[test]
    fn app_integration_stays_under_explicit_app_module() {
        let public_surface = public_surface();
        assert!(public_surface.contains("pub mod app;"));
        assert!(public_surface.contains("pub mod advanced;"));
        assert!(APP_RS.contains("pub fn install(app: &mut fret_app::App)"));
        assert!(APP_RS.contains("pub fn install_with_budgets("));
        assert!(!APP_RS.contains("install_with_ui_services"));
        assert!(ADVANCED_RS.contains("pub fn install_with_ui_services("));
        assert!(ADVANCED_RS.contains("pub fn install_with_ui_services_and_budgets("));
    }
}
