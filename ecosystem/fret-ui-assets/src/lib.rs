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
mod app_integration;
#[cfg(feature = "app-integration")]
pub use app_integration::{install, install_app, install_app_with_budgets, install_with_budgets};
