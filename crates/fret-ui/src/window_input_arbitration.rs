use std::collections::HashMap;

use fret_core::AppWindowId;

use crate::tree::UiInputArbitrationSnapshot;

/// Window-scoped input arbitration snapshots published by the UI runtime.
///
/// This is an integration seam for policy-heavy ecosystem crates: it exposes the current
/// window-level arbitration state (modal barrier, pointer occlusion, pointer capture) without
/// requiring direct access to the `UiTree`.
#[derive(Debug, Default)]
pub struct WindowInputArbitrationService {
    by_window: HashMap<AppWindowId, UiInputArbitrationSnapshot>,
}

impl WindowInputArbitrationService {
    pub fn snapshot(&self, window: AppWindowId) -> Option<UiInputArbitrationSnapshot> {
        self.by_window.get(&window).copied()
    }

    pub fn set_snapshot(&mut self, window: AppWindowId, snapshot: UiInputArbitrationSnapshot) {
        self.by_window.insert(window, snapshot);
    }

    pub fn remove_window(&mut self, window: AppWindowId) {
        self.by_window.remove(&window);
    }
}
