//! Development-oriented reload controls for UI assets.
//!
//! `fret-ui-assets` intentionally re-exports the shared runtime asset-reload vocabulary rather than
//! inventing a UI-specific reload contract.
//! Desktop hosts may drive that epoch from native file watchers or metadata polling depending on
//! startup policy and platform support.

pub use fret_runtime::{
    AssetReloadBackendKind, AssetReloadEpoch, AssetReloadFallbackReason, AssetReloadStatus,
    asset_reload_epoch, asset_reload_status, asset_reload_support, bump_asset_reload_epoch,
};
