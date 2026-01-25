mod bound;
mod cx;
mod input;
mod widget;

use fret_core::{Px, Rect, SemanticsRole, TextMetrics, TextStyle};

use crate::TextInputStyle;

pub use bound::BoundTextInput;

#[derive(Debug)]
pub struct TextInput {
    a11y_role: SemanticsRole,
    enabled: bool,
    focusable: bool,
    text: String,
    caret: usize,
    selection_anchor: usize,
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
    prepared_scale_factor_bits: Option<u32>,
    last_font_stack_key: Option<u64>,
    last_bounds: Rect,
    last_sent_cursor: Option<Rect>,
    pending_clipboard_token: Option<fret_runtime::ClipboardToken>,

    chrome_style: TextInputStyle,
    chrome_override: bool,
    last_theme_revision: Option<u64>,

    text_style_override: bool,
    last_text_style_theme_revision: Option<u64>,
}

#[cfg(test)]
mod tests;
