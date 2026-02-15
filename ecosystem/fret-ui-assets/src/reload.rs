//! Development-oriented reload controls for UI assets.
//!
//! This is intentionally small and portable: instead of using OS-specific file watchers, apps can
//! bump a global epoch to force re-decoding path-based image sources.

use fret_runtime::GlobalsHost;

/// Global epoch that can be observed by UI code (via `ElementContext::observe_global`) to safely
/// invalidate view-cached subtrees when assets should be reloaded.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiAssetsReloadEpoch(pub u64);

impl UiAssetsReloadEpoch {
    pub fn bump(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

pub fn bump_ui_assets_reload_epoch<H: GlobalsHost>(host: &mut H) {
    host.with_global_mut(UiAssetsReloadEpoch::default, |epoch, _host| {
        epoch.bump();
    });
}
