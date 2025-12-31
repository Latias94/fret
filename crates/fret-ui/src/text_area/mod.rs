//! Multiline text area widget (retained) providing IME/caret/selection engine behavior.
//!
//! This lives in the runtime crate because it needs platform hooks and hard-to-change editing
//! semantics (ADR 0012 / ADR 0071).
use fret_core::{
    CaretAffinity, Color, Corners, Edges, Px, Rect, Size, TextConstraints, TextMetrics, TextStyle,
    TextWrap,
};
use fret_runtime::Effect;

use crate::widget::{CommandCx, EventCx};
use crate::{Theme, UiHost};

mod bound;
mod widget;

pub use bound::BoundTextArea;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PreparedKey {
    max_width_bits: u32,
    wrap: TextWrap,
    scale_bits: u32,
    show_scrollbar: bool,
}

#[derive(Debug, Clone)]
pub struct TextAreaStyle {
    pub padding_x: Px,
    pub padding_y: Px,
    pub background: Color,
    pub border: Edges,
    pub border_color: Color,
    pub focus_ring: Option<crate::element::RingStyle>,
    pub corner_radii: Corners,
    pub text_color: Color,
    pub selection_color: Color,
    pub caret_color: Color,
    pub preedit_bg_color: Color,
    pub preedit_underline_color: Color,
}

impl Default for TextAreaStyle {
    fn default() -> Self {
        Self {
            padding_x: Px(10.0),
            padding_y: Px(10.0),
            background: Color {
                r: 0.12,
                g: 0.12,
                b: 0.16,
                a: 1.0,
            },
            border: Edges::all(Px(1.0)),
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.35,
            },
            focus_ring: None,
            corner_radii: Corners::all(Px(8.0)),
            text_color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 1.0,
            },
            selection_color: Color {
                r: 0.24,
                g: 0.34,
                b: 0.52,
                a: 0.65,
            },
            caret_color: Color {
                r: 0.90,
                g: 0.90,
                b: 0.92,
                a: 1.0,
            },
            preedit_bg_color: Color {
                r: 0.24,
                g: 0.34,
                b: 0.52,
                a: 0.22,
            },
            preedit_underline_color: Color {
                r: 0.65,
                g: 0.82,
                b: 1.0,
                a: 0.95,
            },
        }
    }
}

#[derive(Debug)]
pub struct TextArea {
    text: String,
    text_style: TextStyle,
    wrap: TextWrap,
    min_height: Px,
    style: TextAreaStyle,
    style_override: bool,
    last_theme_revision: Option<u64>,
    text_style_override: bool,
    last_text_style_theme_revision: Option<u64>,

    blob: Option<fret_core::TextBlobId>,
    metrics: Option<TextMetrics>,
    pending_release: Vec<fret_core::TextBlobId>,
    prepared_key: Option<PreparedKey>,
    text_dirty: bool,
    show_scrollbar: bool,

    offset_y: Px,
    scrollbar_width: Px,
    dragging_thumb: bool,
    drag_pointer_start_y: Px,
    drag_offset_start_y: Px,
    last_content_height: Px,
    last_viewport_height: Px,

    preedit: String,
    preedit_cursor: Option<(usize, usize)>,
    preedit_rects: Vec<Rect>,
    ime_replace_range: Option<(usize, usize)>,

    caret: usize,
    selection_anchor: usize,
    affinity: CaretAffinity,
    preferred_x: Option<Px>,
    ensure_caret_visible: bool,
    selection_rects: Vec<Rect>,
    last_bounds: Rect,
    last_sent_cursor: Option<Rect>,
    last_text_input_tick: Option<fret_core::TickId>,
    last_text_input_text: Option<String>,
    last_ime_commit_tick: Option<fret_core::TickId>,
    last_ime_commit_text: Option<String>,
}

impl Default for TextArea {
    fn default() -> Self {
        Self {
            text: String::new(),
            text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
                ..Default::default()
            },
            wrap: TextWrap::Word,
            min_height: Px(0.0),
            style: TextAreaStyle::default(),
            style_override: false,
            last_theme_revision: None,
            text_style_override: false,
            last_text_style_theme_revision: None,
            blob: None,
            metrics: None,
            pending_release: Vec::new(),
            prepared_key: None,
            text_dirty: true,
            show_scrollbar: false,
            offset_y: Px(0.0),
            scrollbar_width: Px(10.0),
            dragging_thumb: false,
            drag_pointer_start_y: Px(0.0),
            drag_offset_start_y: Px(0.0),
            last_content_height: Px(0.0),
            last_viewport_height: Px(0.0),
            preedit: String::new(),
            preedit_cursor: None,
            preedit_rects: Vec::new(),
            ime_replace_range: None,
            caret: 0,
            selection_anchor: 0,
            affinity: CaretAffinity::Downstream,
            preferred_x: None,
            ensure_caret_visible: true,
            selection_rects: Vec::new(),
            last_bounds: Rect::default(),
            last_sent_cursor: None,
            last_text_input_tick: None,
            last_text_input_text: None,
            last_ime_commit_tick: None,
            last_ime_commit_text: None,
        }
    }
}

impl TextArea {
    pub fn new(text: impl Into<String>) -> Self {
        Self::default().with_text(text)
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.caret = self.text.len();
        self.selection_anchor = self.caret;
        self.ensure_caret_visible = true;
        self.preedit.clear();
        self.preedit_cursor = None;
        self.ime_replace_range = None;
        self.text_dirty = true;
        self.preferred_x = None;
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.set_text(text);
        self
    }

    pub fn with_text_style(mut self, style: TextStyle) -> Self {
        self.text_style = style;
        self.text_style_override = true;
        self.last_text_style_theme_revision = None;
        self
    }

    pub fn with_wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn with_min_height(mut self, min_height: Px) -> Self {
        self.min_height = min_height;
        self
    }

    pub fn with_style(mut self, style: TextAreaStyle) -> Self {
        self.style = style;
        self.style_override = true;
        self
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        self.scrollbar_width = theme.metrics.scrollbar_width;

        let rev = theme.revision();

        if !self.style_override && self.last_theme_revision != Some(rev) {
            self.last_theme_revision = Some(rev);
            self.style.padding_x = theme.metrics.padding_md;
            self.style.padding_y = theme.metrics.padding_md;
            self.style.background = theme.colors.panel_background;
            self.style.border_color = theme.colors.panel_border;
            // Focus ring styling is intentionally component-owned (recipes) rather than
            // runtime-owned to keep `fret-ui` mechanism-only (ADR 0066). Component libraries can
            // set `TextAreaStyle.focus_ring` explicitly when desired.
            self.style.focus_ring = None;
            self.style.corner_radii = Corners::all(theme.metrics.radius_md);
            self.style.text_color = theme.colors.text_primary;
            self.style.selection_color = theme.colors.selection_background;
            self.style.caret_color = theme.colors.text_primary;
            self.style.preedit_bg_color = Color {
                a: 0.22,
                ..theme.colors.selection_background
            };
            self.style.preedit_underline_color = theme.colors.accent;
        }

        if !self.text_style_override && self.last_text_style_theme_revision != Some(rev) {
            self.last_text_style_theme_revision = Some(rev);
            let next_size = theme
                .metric_by_key("font.size")
                .unwrap_or(theme.metrics.font_size);
            if self.text_style.size != next_size {
                self.text_style.size = next_size;
                self.text_dirty = true;
                self.prepared_key = None;
                if let Some(blob) = self.blob.take() {
                    self.pending_release.push(blob);
                }
                self.metrics = None;
            }
        }
    }

    pub fn offset_y(&self) -> Px {
        self.offset_y
    }

    fn clear_preedit(&mut self) {
        if self.preedit.is_empty() && self.preedit_cursor.is_none() {
            return;
        }
        self.preedit.clear();
        self.preedit_cursor = None;
        self.ime_replace_range = None;
        self.affinity = CaretAffinity::Downstream;
        self.text_dirty = true;
    }

    fn is_ime_composing(&self) -> bool {
        !self.preedit.is_empty() || self.preedit_cursor.is_some()
    }

    fn preedit_cursor_end(&self) -> usize {
        self.preedit_cursor
            .map(|(_, end)| end.min(self.preedit.len()))
            .unwrap_or(self.preedit.len())
    }

    fn layout_text(&self) -> Option<String> {
        if self.preedit.is_empty() {
            return None;
        }
        let prefix = self.text.get(..self.caret)?;
        let suffix = self.text.get(self.caret..)?;
        Some(format!("{prefix}{}{suffix}", self.preedit))
    }

    fn caret_display_index(&self) -> usize {
        if self.preedit.is_empty() {
            self.caret
        } else {
            self.caret + self.preedit_cursor_end()
        }
    }

    fn map_display_index_to_base(&self, display_index: usize) -> usize {
        if self.preedit.is_empty() {
            return display_index;
        }

        let anchor = self.caret;
        let preedit_len = self.preedit.len();
        if display_index <= anchor {
            display_index
        } else if display_index >= anchor + preedit_len {
            display_index - preedit_len
        } else {
            anchor
        }
    }

    fn content_bounds(&self) -> Rect {
        let scrollbar_w = self.scrollbar_width;
        let inner = self.inner_bounds();
        if self.last_content_height.0 > self.last_viewport_height.0 {
            Rect::new(
                inner.origin,
                Size::new(
                    Px((inner.size.width.0 - scrollbar_w.0).max(0.0)),
                    inner.size.height,
                ),
            )
        } else {
            inner
        }
    }

    fn selection_range(&self) -> (usize, usize) {
        let a = self.selection_anchor.min(self.caret);
        let b = self.selection_anchor.max(self.caret);
        (a, b)
    }

    fn clamp_to_boundary(text: &str, idx: usize) -> usize {
        if idx >= text.len() {
            return text.len();
        }
        if text.is_char_boundary(idx) {
            return idx;
        }
        let mut i = idx;
        while i > 0 && !text.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    fn prev_boundary(text: &str, idx: usize) -> usize {
        let idx = Self::clamp_to_boundary(text, idx);
        if idx == 0 {
            return 0;
        }
        let slice = &text[..idx];
        slice.char_indices().last().map(|(i, _)| i).unwrap_or(0)
    }

    fn next_boundary(text: &str, idx: usize) -> usize {
        let idx = Self::clamp_to_boundary(text, idx);
        if idx >= text.len() {
            return text.len();
        }
        let ch = text[idx..].chars().next().unwrap();
        idx + ch.len_utf8()
    }

    fn is_word_char(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    fn move_word_left(text: &str, idx: usize) -> usize {
        let mut i = Self::prev_boundary(text, idx);
        while i > 0 {
            let prev = Self::prev_boundary(text, i);
            let ch = text[prev..i].chars().next().unwrap_or(' ');
            if !ch.is_whitespace() {
                break;
            }
            i = prev;
        }
        while i > 0 {
            let prev = Self::prev_boundary(text, i);
            let ch = text[prev..i].chars().next().unwrap_or(' ');
            if !Self::is_word_char(ch) {
                break;
            }
            i = prev;
        }
        i
    }

    fn move_word_right(text: &str, idx: usize) -> usize {
        let mut i = Self::next_boundary(text, idx);
        while i < text.len() {
            let next = Self::next_boundary(text, i);
            let ch = text[i..next].chars().next().unwrap_or(' ');
            if !ch.is_whitespace() {
                break;
            }
            i = next;
        }
        while i < text.len() {
            let next = Self::next_boundary(text, i);
            let ch = text[i..next].chars().next().unwrap_or(' ');
            if !Self::is_word_char(ch) {
                break;
            }
            i = next;
        }
        i
    }

    fn delete_selection_if_any(&mut self) -> bool {
        let (a, b) = self.selection_range();
        if a == b {
            return false;
        }
        self.text.replace_range(a..b, "");
        self.caret = a;
        self.selection_anchor = self.caret;
        self.clear_preedit();
        self.affinity = CaretAffinity::Downstream;
        self.text_dirty = true;
        true
    }

    fn replace_selection(&mut self, insert: &str) {
        let (a, b) = self.selection_range();
        if a != b {
            self.text.replace_range(a..b, insert);
            self.caret = a + insert.len();
            self.selection_anchor = self.caret;
        } else {
            self.text.insert_str(self.caret, insert);
            self.caret += insert.len();
            self.selection_anchor = self.caret;
        }
        self.clear_preedit();
        self.affinity = CaretAffinity::Downstream;
        self.text_dirty = true;
    }

    fn queue_release_blob(&mut self) {
        if let Some(blob) = self.blob.take() {
            self.pending_release.push(blob);
        }
        self.prepared_key = None;
    }

    fn flush_pending_releases(&mut self, services: &mut dyn fret_core::UiServices) {
        for blob in self.pending_release.drain(..) {
            services.text().release(blob);
        }
    }

    fn request_clipboard_paste<H: UiHost>(&mut self, cx: &mut CommandCx<'_, H>) -> bool {
        let Some(window) = cx.window else {
            return true;
        };
        cx.app.push_effect(Effect::ClipboardGetText { window });
        true
    }

    fn max_offset(&self) -> Px {
        Px((self.last_content_height.0 - self.last_viewport_height.0).max(0.0))
    }

    fn clamp_offset(&mut self, content_height: Px, viewport_height: Px) {
        let max = Px((content_height.0 - viewport_height.0).max(0.0));
        self.offset_y = Px(self.offset_y.0.clamp(0.0, max.0));
    }

    fn scrollbar_geometry(&self, bounds: Rect) -> Option<(Rect, Rect)> {
        let viewport_h = self.last_viewport_height;
        if viewport_h.0 <= 0.0 {
            return None;
        }

        let content_h = self.last_content_height;
        if content_h.0 <= viewport_h.0 {
            return None;
        }

        let w = self.scrollbar_width;
        let track = Rect::new(
            fret_core::Point::new(
                Px(bounds.origin.x.0 + bounds.size.width.0 - w.0),
                bounds.origin.y,
            ),
            Size::new(w, bounds.size.height),
        );

        let ratio = (viewport_h.0 / content_h.0).clamp(0.0, 1.0);
        let min_thumb = 24.0;
        let thumb_h = Px((viewport_h.0 * ratio).max(min_thumb).min(viewport_h.0));

        let max_offset = self.max_offset().0;
        let t = if max_offset <= 0.0 {
            0.0
        } else {
            (self.offset_y.0 / max_offset).clamp(0.0, 1.0)
        };
        let travel = (viewport_h.0 - thumb_h.0).max(0.0);
        let thumb_y = Px(track.origin.y.0 + travel * t);

        let thumb = Rect::new(
            fret_core::Point::new(track.origin.x, thumb_y),
            Size::new(w, thumb_h),
        );

        Some((track, thumb))
    }

    fn set_offset_from_thumb_y(&mut self, bounds: Rect, thumb_top_y: Px) {
        let Some((track, thumb)) = self.scrollbar_geometry(bounds) else {
            return;
        };

        let viewport_h = self.last_viewport_height.0;
        let travel = (viewport_h - thumb.size.height.0).max(0.0);
        if travel <= 0.0 {
            self.offset_y = Px(0.0);
            return;
        }

        let t = ((thumb_top_y.0 - track.origin.y.0) / travel).clamp(0.0, 1.0);
        let max = self.max_offset().0;
        self.offset_y = Px(max * t);
    }

    fn inner_bounds(&self) -> Rect {
        let px = self.style.padding_x;
        let py = self.style.padding_y;
        Rect::new(
            fret_core::Point::new(
                self.last_bounds.origin.x + px,
                self.last_bounds.origin.y + py,
            ),
            Size::new(
                Px((self.last_bounds.size.width.0 - px.0 * 2.0).max(0.0)),
                Px((self.last_bounds.size.height.0 - py.0 * 2.0).max(0.0)),
            ),
        )
    }

    fn set_caret_from_point<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        point: fret_core::Point,
    ) {
        let Some(blob) = self.blob else {
            return;
        };
        let hit = cx.services.hit_test_point(blob, point);
        if self.preedit.is_empty() {
            self.caret = hit.index;
            self.affinity = hit.affinity;
        } else {
            self.caret = self.map_display_index_to_base(hit.index);
            self.clear_preedit();
            self.affinity = CaretAffinity::Downstream;
        }
    }
}
