//! Development-oriented reload controls for UI assets.
//!
//! This is intentionally small and portable: instead of using OS-specific file watchers, apps can
//! bump a global epoch to force re-decoding path-based image sources.

use fret_runtime::GlobalsHost;

pub use fret_runtime::{AssetReloadEpoch, asset_reload_epoch, asset_reload_support};

#[deprecated(
    note = "prefer fret_runtime::AssetReloadEpoch or app-facing asset helpers; the generic asset reload epoch is no longer UI-specific"
)]
pub type UiAssetsReloadEpoch = AssetReloadEpoch;

#[deprecated(
    note = "prefer fret_runtime::bump_asset_reload_epoch or app-facing asset helpers; the generic asset reload epoch is no longer UI-specific"
)]
pub fn bump_ui_assets_reload_epoch<H: GlobalsHost>(host: &mut H) {
    fret_runtime::bump_asset_reload_epoch(host);
}
