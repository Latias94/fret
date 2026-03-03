use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{AppWindowId, Rect};

/// Best-effort IME surrounding text excerpt.
///
/// This is intended for platform bridges that need "text around caret" semantics (e.g. winit's
/// `ImeSurroundingText`).
///
/// - `text` MUST exclude any active preedit/composing text.
/// - `cursor`/`anchor` are UTF-8 byte offsets within `text` (must be on char boundaries).
/// - `text` SHOULD be limited to at most 4000 bytes (winit backend constraint).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindowImeSurroundingText {
    pub text: Arc<str>,
    pub cursor: u32,
    pub anchor: u32,
}

impl WindowImeSurroundingText {
    pub const MAX_TEXT_BYTES: usize = 4000;

    /// Create a best-effort surrounding text excerpt from a full base-buffer string.
    ///
    /// The returned excerpt is bounded to [`Self::MAX_TEXT_BYTES`] and the cursor/anchor offsets
    /// are relative to the excerpt (UTF-8 bytes).
    pub fn best_effort_for_str(text: &str, cursor: usize, anchor: usize) -> Self {
        fn clamp_down_to_char_boundary(text: &str, idx: usize) -> usize {
            let mut idx = idx.min(text.len());
            while idx > 0 && !text.is_char_boundary(idx) {
                idx = idx.saturating_sub(1);
            }
            idx
        }

        let cursor = clamp_down_to_char_boundary(text, cursor);
        let mut anchor = clamp_down_to_char_boundary(text, anchor);
        let len = text.len();

        if len <= Self::MAX_TEXT_BYTES {
            return Self {
                text: Arc::<str>::from(text),
                cursor: u32::try_from(cursor).unwrap_or(u32::MAX),
                anchor: u32::try_from(anchor).unwrap_or(u32::MAX),
            };
        }

        let mut low = cursor.min(anchor);
        let mut high = cursor.max(anchor);
        if high.saturating_sub(low) > Self::MAX_TEXT_BYTES {
            anchor = cursor;
            low = cursor;
            high = cursor;
        }

        let needed = high.saturating_sub(low);
        let slack = Self::MAX_TEXT_BYTES.saturating_sub(needed);
        let before = slack / 2;

        let mut start = low
            .saturating_sub(before)
            .min(len.saturating_sub(Self::MAX_TEXT_BYTES));
        let mut end = (start + Self::MAX_TEXT_BYTES).min(len);

        start = clamp_down_to_char_boundary(text, start);
        end = clamp_down_to_char_boundary(text, end);
        if end < start {
            end = start;
        }

        let cursor_rel = cursor.saturating_sub(start).min(end.saturating_sub(start));
        let anchor_rel = anchor.saturating_sub(start).min(end.saturating_sub(start));
        let excerpt = &text[start..end];

        Self {
            text: Arc::<str>::from(excerpt),
            cursor: u32::try_from(cursor_rel).unwrap_or(u32::MAX),
            anchor: u32::try_from(anchor_rel).unwrap_or(u32::MAX),
        }
    }
}

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
    /// Best-effort surrounding text excerpt for IME backends that support it.
    pub surrounding_text: Option<WindowImeSurroundingText>,
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
