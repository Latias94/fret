//! UI render asset conveniences (Image/SVG caches and upload helpers).
//!
//! This crate intentionally re-exports `ecosystem/fret-asset-cache` under a name that is less
//! likely to be misread as an editor/project asset pipeline. See ADR 0004 and ADR 0106.

pub mod image_asset_state;
pub mod image_source;
pub mod svg_asset_state;
pub mod ui_assets;

pub use fret_asset_cache::*;
pub use image_source::*;
pub use ui_assets::*;

#[cfg(feature = "app-integration")]
mod app_integration;
#[cfg(feature = "app-integration")]
pub use app_integration::{install, install_app, install_app_with_budgets, install_with_budgets};
