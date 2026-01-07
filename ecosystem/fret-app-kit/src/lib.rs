pub mod tree;

pub mod avatar_asset_cache;
#[deprecated(
    note = "Use `fret_ui_assets::image_asset_cache` directly; this module will be removed once downstream code migrates."
)]
pub mod image_asset_cache;
pub mod image_asset_state;
#[deprecated(
    note = "Use `fret_ui_assets::image_upload` directly; this module will be removed once downstream code migrates."
)]
pub mod image_upload;
#[deprecated(
    note = "Use `fret_ui_assets::svg_asset_cache` directly; this module will be removed once downstream code migrates."
)]
pub mod svg_asset_cache;
pub mod svg_asset_state;
