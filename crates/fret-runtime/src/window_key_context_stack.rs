use std::collections::HashMap;
use std::sync::Arc;

use fret_core::AppWindowId;

/// Window-scoped key-context stack snapshots published by the UI runtime.
///
/// This is a data-only integration seam that allows keymap `when` expressions (`keyctx.*`) and
/// cross-surface command gating (menus, palette, shortcut help) to observe the active key contexts
/// without depending on `fret-ui` internals.
#[derive(Debug, Default)]
pub struct WindowKeyContextStackService {
    by_window: HashMap<AppWindowId, Vec<Arc<str>>>,
}

impl WindowKeyContextStackService {
    pub fn snapshot(&self, window: AppWindowId) -> Option<&[Arc<str>]> {
        self.by_window.get(&window).map(|v| v.as_slice())
    }

    pub fn set_snapshot(&mut self, window: AppWindowId, key_contexts: Vec<Arc<str>>) {
        self.by_window.insert(window, key_contexts);
    }

    pub fn remove_window(&mut self, window: AppWindowId) {
        self.by_window.remove(&window);
    }
}
