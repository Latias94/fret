use std::collections::HashSet;

use fret_core::{AppWindowId, CursorIcon};

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_cursor_set_icon(
        &mut self,
        window: AppWindowId,
        icon: CursorIcon,
        window_state_dirty: &mut HashSet<AppWindowId>,
    ) {
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        if state.platform.set_cursor_icon(icon) {
            window_state_dirty.insert(window);
        }
    }
}
