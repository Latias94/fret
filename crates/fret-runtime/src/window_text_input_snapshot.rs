use std::collections::HashMap;

use fret_core::{AppWindowId, Rect};

/// Window-scoped platform text-input snapshots published by the UI runtime.
///
/// This is a data-only integration seam for platform/runner layers that need to interop with
/// editor-grade text input (IME, accessibility).
///
/// Indices are expressed in UTF-16 code units over the widget's **composed view**:
/// base buffer text with the active IME preedit spliced at the caret.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WindowTextInputSnapshot {
    pub focus_is_text_input: bool,
    pub is_composing: bool,
    /// Total length (UTF-16 code units) of the composed view.
    pub text_len_utf16: u32,
    /// Anchor/focus selection offsets in UTF-16 code units (composed view).
    pub selection_utf16: Option<(u32, u32)>,
    /// Marked (preedit) range in UTF-16 code units (composed view).
    pub marked_utf16: Option<(u32, u32)>,
    /// Best-effort IME cursor area in window logical coordinates.
    ///
    /// On Windows/winit this is the primary hook for positioning the candidate window.
    pub ime_cursor_area: Option<Rect>,
}

#[derive(Debug, Default)]
pub struct WindowTextInputSnapshotService {
    by_window: HashMap<AppWindowId, WindowTextInputSnapshot>,
}

impl WindowTextInputSnapshotService {
    pub fn snapshot(&self, window: AppWindowId) -> Option<&WindowTextInputSnapshot> {
        self.by_window.get(&window)
    }

    pub fn set_snapshot(&mut self, window: AppWindowId, snapshot: WindowTextInputSnapshot) {
        self.by_window.insert(window, snapshot);
    }

    pub fn remove_window(&mut self, window: AppWindowId) {
        self.by_window.remove(&window);
    }
}
