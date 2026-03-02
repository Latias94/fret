use std::collections::HashMap;

use fret_core::AppWindowId;

use crate::InputContext;

/// Window-scoped `InputContext` snapshots published by the UI runtime.
///
/// This is a data-only integration seam that allows runner/platform layers (e.g. OS menu bars) to
/// access focus/modal state without depending on `fret-ui` internals.
#[derive(Debug, Default)]
pub struct WindowInputContextService {
    by_window: HashMap<AppWindowId, InputContext>,
}

impl WindowInputContextService {
    pub fn window_count(&self) -> usize {
        self.by_window.len()
    }

    pub fn snapshot(&self, window: AppWindowId) -> Option<&InputContext> {
        self.by_window.get(&window)
    }

    pub fn set_snapshot(&mut self, window: AppWindowId, input_ctx: InputContext) {
        self.by_window.insert(window, input_ctx);
    }

    pub fn remove_window(&mut self, window: AppWindowId) {
        self.by_window.remove(&window);
    }
}
