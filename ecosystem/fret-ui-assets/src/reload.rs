//! Development-oriented reload controls for UI assets.
//!
//! UI-specific compatibility shims now sit on top of the shared runtime asset reload epoch.
//! Desktop hosts may drive that epoch from native file watchers or metadata polling depending on
//! startup policy and platform support.

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
