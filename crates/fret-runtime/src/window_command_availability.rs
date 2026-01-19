use std::collections::HashMap;

use fret_core::AppWindowId;

/// Window-scoped command availability snapshots published by the app layer.
///
/// This is a data-only integration seam used by cross-surface command gating (menus, command
/// palette, shortcuts) without depending on UI-kit or app-specific model types.
#[derive(Debug, Default)]
pub struct WindowCommandAvailabilityService {
    by_window: HashMap<AppWindowId, WindowCommandAvailability>,
}

impl WindowCommandAvailabilityService {
    pub fn snapshot(&self, window: AppWindowId) -> Option<&WindowCommandAvailability> {
        self.by_window.get(&window)
    }

    pub fn set_snapshot(&mut self, window: AppWindowId, availability: WindowCommandAvailability) {
        self.by_window.insert(window, availability);
    }

    pub fn remove_window(&mut self, window: AppWindowId) {
        self.by_window.remove(&window);
    }
}

/// Minimal command availability surface (v1).
///
/// This is intentionally conservative: it only captures state that is hard to infer at the runner
/// boundary and is needed for native OS menus to present correct enable/disable states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowCommandAvailability {
    pub edit_can_undo: bool,
    pub edit_can_redo: bool,
}

impl Default for WindowCommandAvailability {
    fn default() -> Self {
        Self {
            edit_can_undo: true,
            edit_can_redo: true,
        }
    }
}
