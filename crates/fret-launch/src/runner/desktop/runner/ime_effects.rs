use std::collections::HashSet;

use fret_core::{AppWindowId, Rect};

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_ime_allow(
        &mut self,
        window: AppWindowId,
        enabled: bool,
        window_state_dirty: &mut HashSet<AppWindowId>,
    ) {
        let changed = self
            .windows
            .get_mut(window)
            .is_some_and(|state| state.platform.set_ime_allowed(enabled));
        if !changed {
            return;
        }

        #[cfg(target_os = "android")]
        self.android_force_soft_input(enabled);
        window_state_dirty.insert(window);
    }

    pub(super) fn handle_ime_request_virtual_keyboard(
        &mut self,
        window: AppWindowId,
        visible: bool,
    ) {
        #[cfg(target_os = "android")]
        {
            let _ = window;
            self.android_force_soft_input(visible);
        }
        #[cfg(not(target_os = "android"))]
        {
            let _ = (window, visible);
        }
    }

    pub(super) fn handle_ime_set_cursor_area(
        &mut self,
        window: AppWindowId,
        rect: Rect,
        window_state_dirty: &mut HashSet<AppWindowId>,
    ) {
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        if std::env::var_os("FRET_IME_DEBUG").is_some_and(|v| !v.is_empty()) {
            tracing::info!(
                "IME_DEBUG effect: ImeSetCursorArea window={:?} rect=({:.1},{:.1} {:.1}x{:.1})",
                window,
                rect.origin.x.0,
                rect.origin.y.0,
                rect.size.width.0,
                rect.size.height.0
            );
        }
        if state.platform.set_ime_cursor_area(rect) {
            window_state_dirty.insert(window);
        }
    }
}
