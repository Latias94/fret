use fret_ui::UiHost;

use super::AppUi;

/// Grouped render-time effect helpers for the default app authoring surface.
#[doc(hidden)]
pub struct AppUiEffects<'view, 'cx, 'a, H: UiHost> {
    pub(super) cx: &'view mut AppUi<'cx, 'a, H>,
}

impl<'view, 'cx, 'a, H: UiHost> AppUiEffects<'view, 'cx, 'a, H> {
    pub fn take_transient(self, key: u64) -> bool {
        self.cx.cx.take_transient_for(self.cx.action_root, key)
    }
}
