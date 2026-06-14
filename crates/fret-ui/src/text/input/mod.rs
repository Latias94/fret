mod bound;
mod cx;
#[allow(clippy::module_inception)]
mod input;
mod widget;

use fret_core::{Point, Px, Rect, SemanticsRole, TextMetrics, TextStyle};

use crate::{Invalidation, TextInputStyle};

pub use bound::BoundTextInput;

pub type TextInputInsertFilter = std::sync::Arc<dyn Fn(&str) -> String + 'static>;

const OBSCURE_MASK: &str = "•";
const OBSCURE_MASK_BYTES: usize = OBSCURE_MASK.len();

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ImeSurroundingTextCacheKey {
    text_revision: u64,
    caret: usize,
    selection_anchor: usize,
}

#[derive(Debug, Default, Clone)]
struct ImeSurroundingTextCache {
    key: Option<ImeSurroundingTextCacheKey>,
    value: Option<fret_runtime::WindowImeSurroundingText>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ObscureTextCacheKey {
    text_revision: u64,
}

#[derive(Debug, Default, Clone)]
struct ObscureTextCache {
    key: Option<ObscureTextCacheKey>,
    masked: String,
    base_grapheme_boundaries: Vec<usize>,
}

pub struct TextInput {
    a11y_role: SemanticsRole,
    enabled: bool,
    focusable: bool,
    read_only: bool,
    focus_ring_always_paint: bool,
    obscure_text: bool,
    insert_filter: Option<TextInputInsertFilter>,
    obscure_text_cache: ObscureTextCache,
    text: String,
    base_text_revision: u64,
    ime_surrounding_text_cache: std::cell::RefCell<ImeSurroundingTextCache>,
    caret_blink_timer: Option<fret_runtime::TimerToken>,
    caret_blink_visible: bool,
    caret: usize,
    selection_anchor: usize,
    offset_x: Px,
    selection_dragging: bool,
    last_pointer_pos: Option<Point>,
    selection_autoscroll_timer: Option<fret_runtime::TimerToken>,
    preedit: String,
    preedit_cursor: Option<(usize, usize)>,
    ime_replace_range: Option<(usize, usize)>,
    ime_deduper: crate::text_edit::ime::Deduper,
    style: TextStyle,
    placeholder: Option<std::sync::Arc<str>>,
    text_blob: Option<fret_core::TextBlobId>,
    text_metrics: Option<TextMetrics>,
    placeholder_blob: Option<fret_core::TextBlobId>,
    placeholder_metrics: Option<TextMetrics>,
    prefix_blob: Option<fret_core::TextBlobId>,
    prefix_metrics: Option<TextMetrics>,
    suffix_blob: Option<fret_core::TextBlobId>,
    suffix_metrics: Option<TextMetrics>,
    preedit_blob: Option<fret_core::TextBlobId>,
    preedit_metrics: Option<TextMetrics>,
    caret_stops: Vec<(usize, Px)>,
    pending_release: Vec<fret_core::TextBlobId>,
    text_change_invalidation: Invalidation,
    prepared_scale_factor_bits: Option<u32>,
    last_font_stack_key: Option<u64>,
    last_bounds: Rect,
    last_sent_cursor: Option<Rect>,
    last_visual_snapshot: Option<fret_runtime::WindowTextInputVisualSnapshot>,
    pending_clipboard_token: Option<fret_runtime::ClipboardToken>,
    pending_primary_selection_token: Option<fret_runtime::ClipboardToken>,

    chrome_style: TextInputStyle,
    chrome_override: bool,
    last_theme_revision: Option<u64>,

    text_style_override: bool,
    last_text_style_theme_revision: Option<u64>,
}

impl std::fmt::Debug for TextInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextInput")
            .field("a11y_role", &self.a11y_role)
            .field("enabled", &self.enabled)
            .field("focusable", &self.focusable)
            .field("read_only", &self.read_only)
            .field("focus_ring_always_paint", &self.focus_ring_always_paint)
            .field("obscure_text", &self.obscure_text)
            .field("insert_filter", &self.insert_filter.is_some())
            .field("text", &self.text)
            .field("caret", &self.caret)
            .field("selection_anchor", &self.selection_anchor)
            .field("preedit", &self.preedit)
            .field("preedit_cursor", &self.preedit_cursor)
            .field("ime_replace_range", &self.ime_replace_range)
            .field("placeholder", &self.placeholder)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
