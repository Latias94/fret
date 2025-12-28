//! Multiline text area widget (retained) providing IME/caret/selection engine behavior.
//!
//! This lives in the runtime crate because it needs platform hooks and hard-to-change editing
//! semantics (ADR 0012 / ADR 0071).
use fret_core::{
    CaretAffinity, Color, Corners, DrawOrder, Edges, Event, ImeEvent, MouseButton, Px, Rect,
    SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{Effect, Model};

use crate::{CommandCx, EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};

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
            self.style.focus_ring = Some(crate::element::RingStyle {
                placement: crate::element::RingPlacement::Outset,
                width: theme
                    .metric_by_key("component.ring.width")
                    .unwrap_or(Px(2.0)),
                offset: theme
                    .metric_by_key("component.ring.offset")
                    .unwrap_or(Px(2.0)),
                color: theme
                    .color_by_key("ring")
                    .unwrap_or(theme.colors.focus_ring),
                offset_color: Some(
                    theme
                        .color_by_key("ring-offset-background")
                        .unwrap_or(theme.colors.surface_background),
                ),
                corner_radii: Corners::all(theme.metrics.radius_md),
            });
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

impl<H: UiHost> Widget<H> for TextArea {
    fn is_focusable(&self) -> bool {
        true
    }

    fn is_text_input(&self) -> bool {
        true
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.queue_release_blob();
        self.flush_pending_releases(services);
        self.metrics = None;
        self.prepared_key = None;
    }

    fn semantics(&mut self, cx: &mut crate::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::TextField);
        cx.set_focusable(true);
        cx.set_value_editable(true);
        cx.set_value(self.text().to_string());
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        match event {
            Event::Pointer(fret_core::PointerEvent::Wheel { delta, .. }) => {
                self.offset_y = Px((self.offset_y.0 - delta.y.0).max(0.0));
                self.clamp_offset(self.last_content_height, self.last_viewport_height);
                self.ensure_caret_visible = false;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                button, position, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }

                if let Some((track, thumb)) = self.scrollbar_geometry(self.last_bounds)
                    && track.contains(*position)
                {
                    if thumb.contains(*position) {
                        self.dragging_thumb = true;
                        self.drag_pointer_start_y = position.y;
                        self.drag_offset_start_y = self.offset_y;
                        cx.capture_pointer(cx.node);
                    } else {
                        let centered = Px(position.y.0 - thumb.size.height.0 * 0.5);
                        self.set_offset_from_thumb_y(self.last_bounds, centered);
                        self.clamp_offset(self.last_content_height, self.last_viewport_height);
                    }

                    self.ensure_caret_visible = false;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                cx.request_focus(cx.node);
                cx.capture_pointer(cx.node);
                self.dragging_thumb = false;

                let had_preedit = !self.preedit.is_empty();
                let inner = self.content_bounds();
                let local =
                    fret_core::Point::new(position.x - inner.origin.x, position.y - inner.origin.y);
                let local = fret_core::Point::new(local.x, Px(local.y.0 + self.offset_y.0));
                self.set_caret_from_point(cx, local);
                self.selection_anchor = self.caret;
                self.ensure_caret_visible = true;

                if had_preedit {
                    cx.invalidate_self(Invalidation::Layout);
                }
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Pointer(fret_core::PointerEvent::Move {
                position, buttons, ..
            }) => {
                // Show the I-beam cursor when hovering the editable text region.
                if cx.captured == Some(cx.node) {
                    cx.set_cursor_icon(fret_core::CursorIcon::Text);
                } else if self.last_bounds.contains(*position) {
                    if let Some((track, _thumb)) = self.scrollbar_geometry(self.last_bounds)
                        && track.contains(*position)
                    {
                        // Keep the default cursor over the scrollbar.
                    } else if self.content_bounds().contains(*position) {
                        cx.set_cursor_icon(fret_core::CursorIcon::Text);
                    }
                }

                if !buttons.left {
                    return;
                }
                if cx.captured != Some(cx.node) {
                    return;
                }

                if self.dragging_thumb {
                    let dy = position.y.0 - self.drag_pointer_start_y.0;
                    let Some((_, thumb)) = self.scrollbar_geometry(self.last_bounds) else {
                        return;
                    };

                    let max_offset = self.max_offset().0;
                    let travel = (self.last_viewport_height.0 - thumb.size.height.0).max(0.0);
                    if travel <= 0.0 || max_offset <= 0.0 {
                        return;
                    }

                    let offset_delta = dy / travel * max_offset;
                    self.offset_y = Px(self.drag_offset_start_y.0 + offset_delta);
                    self.clamp_offset(self.last_content_height, self.last_viewport_height);
                    self.ensure_caret_visible = false;

                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let had_preedit = !self.preedit.is_empty();
                let inner = self.content_bounds();
                let local =
                    fret_core::Point::new(position.x - inner.origin.x, position.y - inner.origin.y);
                let local = fret_core::Point::new(local.x, Px(local.y.0 + self.offset_y.0));
                self.set_caret_from_point(cx, local);
                self.ensure_caret_visible = true;

                if had_preedit {
                    cx.invalidate_self(Invalidation::Layout);
                }
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Pointer(fret_core::PointerEvent::Up { button, .. }) => {
                if *button == MouseButton::Left && cx.captured == Some(cx.node) {
                    self.dragging_thumb = false;
                    cx.release_pointer_capture();
                }
            }
            Event::KeyDown { key, modifiers, .. } => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                if !self.is_ime_composing() {
                    return;
                }
                if modifiers.ctrl || modifiers.alt || modifiers.meta {
                    return;
                }

                // During IME composition (preedit), reserve common navigation/IME keys for the
                // platform IME path (ADR 0012).
                if matches!(
                    key,
                    fret_core::KeyCode::Tab
                        | fret_core::KeyCode::Space
                        | fret_core::KeyCode::Enter
                        | fret_core::KeyCode::NumpadEnter
                        | fret_core::KeyCode::Escape
                        | fret_core::KeyCode::ArrowUp
                        | fret_core::KeyCode::ArrowDown
                        | fret_core::KeyCode::ArrowLeft
                        | fret_core::KeyCode::ArrowRight
                        | fret_core::KeyCode::Backspace
                        | fret_core::KeyCode::Delete
                        | fret_core::KeyCode::Home
                        | fret_core::KeyCode::End
                        | fret_core::KeyCode::PageUp
                        | fret_core::KeyCode::PageDown
                ) {
                    cx.stop_propagation();
                }
            }
            Event::TextInput(text) => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                if self.is_ime_composing() {
                    return;
                }
                let tick = cx.app.tick_id();
                if self.last_ime_commit_tick == Some(tick)
                    && self.last_ime_commit_text.as_deref() == Some(text.as_str())
                {
                    return;
                }
                self.last_text_input_tick = Some(tick);
                self.last_text_input_text = Some(text.clone());

                self.replace_selection(text);
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
            }
            Event::ClipboardText(text) => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                self.clear_preedit();
                let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                if !normalized.is_empty() {
                    self.replace_selection(&normalized);
                    self.ensure_caret_visible = true;
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
            }
            Event::Ime(ime) => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                match ime {
                    ImeEvent::Enabled => {}
                    ImeEvent::Disabled => {
                        self.clear_preedit();
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                    ImeEvent::Commit(text) => {
                        let tick = cx.app.tick_id();
                        if self.last_text_input_tick == Some(tick)
                            && self.last_text_input_text.as_deref() == Some(text.as_str())
                        {
                            return;
                        }
                        self.last_ime_commit_tick = Some(tick);
                        self.last_ime_commit_text = Some(text.clone());

                        self.replace_selection(text);
                        self.ensure_caret_visible = true;
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                    ImeEvent::Preedit { text, cursor } => {
                        if text.is_empty() && cursor.is_none() {
                            self.clear_preedit();
                        } else {
                            self.preedit = text.clone();
                            self.preedit_cursor = *cursor;
                            self.selection_anchor = self.caret;
                            self.affinity = CaretAffinity::Downstream;
                            self.text_dirty = true;
                        }
                        self.ensure_caret_visible = true;
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &fret_runtime::CommandId) -> bool {
        if cx.focus != Some(cx.node) {
            return false;
        }

        let cmd = command.as_str();
        let is_vertical = matches!(
            cmd,
            "text.move_up" | "text.move_down" | "text.select_up" | "text.select_down"
        );
        let is_line_home_end = matches!(
            cmd,
            "text.move_home" | "text.move_end" | "text.select_home" | "text.select_end"
        );
        if cmd != "text.copy" && !is_vertical {
            self.preferred_x = None;
        }
        let had_preedit = !self.preedit.is_empty();
        if had_preedit && (is_vertical || is_line_home_end) {
            return true;
        }
        if had_preedit && cmd != "text.copy" {
            self.clear_preedit();
            cx.invalidate_self(Invalidation::Layout);
            cx.request_redraw();
        }

        let hit_test_line =
            |this: &mut Self, cx: &mut CommandCx<'_, H>, at_line_end: bool| -> bool {
                let Some(blob) = this.blob else {
                    return true;
                };

                let caret_index = this.caret_display_index();
                let caret_rect = cx.services.caret_rect(blob, caret_index, this.affinity);
                let y = Px(caret_rect.origin.y.0 + caret_rect.size.height.0 * 0.5);
                let x = if at_line_end { Px(1.0e6) } else { Px(-1.0e6) };
                let hit = cx
                    .services
                    .hit_test_point(blob, fret_core::Point::new(x, y));
                this.caret = this.map_display_index_to_base(hit.index);
                this.affinity = hit.affinity;
                true
            };

        match cmd {
            "text.clear" => {
                self.text.clear();
                self.caret = 0;
                self.selection_anchor = 0;
                self.affinity = CaretAffinity::Downstream;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
                true
            }
            "text.select_all" => {
                self.selection_anchor = 0;
                self.caret = self.text.len();
                self.affinity = CaretAffinity::Downstream;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.copy" => {
                let (a, b) = self.selection_range();
                if a != b {
                    cx.app.push_effect(Effect::ClipboardSetText {
                        text: self.text[a..b].to_string(),
                    });
                }
                true
            }
            "text.cut" => {
                let (a, b) = self.selection_range();
                if a != b {
                    cx.app.push_effect(Effect::ClipboardSetText {
                        text: self.text[a..b].to_string(),
                    });
                    self.delete_selection_if_any();
                    self.ensure_caret_visible = true;
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
                true
            }
            "text.paste" => self.request_clipboard_paste(cx),
            "text.move_left" => {
                self.caret = Self::prev_boundary(&self.text, self.caret);
                self.selection_anchor = self.caret;
                self.affinity = CaretAffinity::Downstream;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_right" => {
                self.caret = Self::next_boundary(&self.text, self.caret);
                self.selection_anchor = self.caret;
                self.affinity = CaretAffinity::Downstream;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_word_left" => {
                self.caret = Self::move_word_left(&self.text, self.caret);
                self.selection_anchor = self.caret;
                self.affinity = CaretAffinity::Downstream;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_word_right" => {
                self.caret = Self::move_word_right(&self.text, self.caret);
                self.selection_anchor = self.caret;
                self.affinity = CaretAffinity::Downstream;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_home" => {
                hit_test_line(self, cx, false);
                self.selection_anchor = self.caret;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_end" => {
                hit_test_line(self, cx, true);
                self.selection_anchor = self.caret;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_up" => {
                let Some(blob) = self.blob else {
                    return true;
                };
                let caret_index = self.caret_display_index();
                let caret_rect = cx.services.caret_rect(blob, caret_index, self.affinity);
                let x = self.preferred_x.unwrap_or(caret_rect.origin.x);
                let y = Px(caret_rect.origin.y.0 - 1.0);
                let hit = cx
                    .services
                    .hit_test_point(blob, fret_core::Point::new(x, y));
                self.caret = hit.index;
                self.selection_anchor = self.caret;
                self.affinity = hit.affinity;
                self.preferred_x = Some(x);
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_down" => {
                let Some(blob) = self.blob else {
                    return true;
                };
                let caret_index = self.caret_display_index();
                let caret_rect = cx.services.caret_rect(blob, caret_index, self.affinity);
                let x = self.preferred_x.unwrap_or(caret_rect.origin.x);
                let y = Px(caret_rect.origin.y.0 + caret_rect.size.height.0 + 1.0);
                let hit = cx
                    .services
                    .hit_test_point(blob, fret_core::Point::new(x, y));
                self.caret = hit.index;
                self.selection_anchor = self.caret;
                self.affinity = hit.affinity;
                self.preferred_x = Some(x);
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_left" => {
                self.caret = Self::prev_boundary(&self.text, self.caret);
                self.affinity = CaretAffinity::Downstream;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_right" => {
                self.caret = Self::next_boundary(&self.text, self.caret);
                self.affinity = CaretAffinity::Downstream;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_word_left" => {
                self.caret = Self::move_word_left(&self.text, self.caret);
                self.affinity = CaretAffinity::Downstream;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_word_right" => {
                self.caret = Self::move_word_right(&self.text, self.caret);
                self.affinity = CaretAffinity::Downstream;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_home" => {
                hit_test_line(self, cx, false);
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_end" => {
                hit_test_line(self, cx, true);
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_up" => {
                let Some(blob) = self.blob else {
                    return true;
                };
                let caret_index = self.caret_display_index();
                let caret_rect = cx.services.caret_rect(blob, caret_index, self.affinity);
                let x = self.preferred_x.unwrap_or(caret_rect.origin.x);
                let y = Px(caret_rect.origin.y.0 - 1.0);
                let hit = cx
                    .services
                    .hit_test_point(blob, fret_core::Point::new(x, y));
                self.caret = hit.index;
                self.affinity = hit.affinity;
                self.preferred_x = Some(x);
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_down" => {
                let Some(blob) = self.blob else {
                    return true;
                };
                let caret_index = self.caret_display_index();
                let caret_rect = cx.services.caret_rect(blob, caret_index, self.affinity);
                let x = self.preferred_x.unwrap_or(caret_rect.origin.x);
                let y = Px(caret_rect.origin.y.0 + caret_rect.size.height.0 + 1.0);
                let hit = cx
                    .services
                    .hit_test_point(blob, fret_core::Point::new(x, y));
                self.caret = hit.index;
                self.affinity = hit.affinity;
                self.preferred_x = Some(x);
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.delete_backward" => {
                if self.delete_selection_if_any() {
                    self.ensure_caret_visible = true;
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                    return true;
                }
                if self.caret == 0 {
                    return true;
                }
                let prev = Self::prev_boundary(&self.text, self.caret);
                self.text.replace_range(prev..self.caret, "");
                self.caret = prev;
                self.selection_anchor = self.caret;
                self.affinity = CaretAffinity::Downstream;
                self.text_dirty = true;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
                true
            }
            "text.delete_forward" => {
                if self.delete_selection_if_any() {
                    self.ensure_caret_visible = true;
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                    return true;
                }
                if self.caret >= self.text.len() {
                    return true;
                }
                let next = Self::next_boundary(&self.text, self.caret);
                self.text.replace_range(self.caret..next, "");
                self.selection_anchor = self.caret;
                self.affinity = CaretAffinity::Downstream;
                self.text_dirty = true;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
                true
            }
            "text.delete_word_backward" => {
                if self.delete_selection_if_any() {
                    self.ensure_caret_visible = true;
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                    return true;
                }
                if self.caret == 0 {
                    return true;
                }
                let prev = Self::move_word_left(&self.text, self.caret);
                self.text.replace_range(prev..self.caret, "");
                self.caret = prev;
                self.selection_anchor = self.caret;
                self.affinity = CaretAffinity::Downstream;
                self.text_dirty = true;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
                true
            }
            "text.delete_word_forward" => {
                if self.delete_selection_if_any() {
                    self.ensure_caret_visible = true;
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                    return true;
                }
                if self.caret >= self.text.len() {
                    return true;
                }
                let next = Self::move_word_right(&self.text, self.caret);
                self.text.replace_range(self.caret..next, "");
                self.selection_anchor = self.caret;
                self.affinity = CaretAffinity::Downstream;
                self.text_dirty = true;
                self.ensure_caret_visible = true;
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
                true
            }
            _ => false,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        self.caret = Self::clamp_to_boundary(&self.text, self.caret);
        self.selection_anchor = Self::clamp_to_boundary(&self.text, self.selection_anchor);

        let scrollbar_w = self.scrollbar_width;

        let inner = self.inner_bounds();
        let layout_text_owned = self.layout_text();
        let layout_text = layout_text_owned.as_deref().unwrap_or(&self.text);

        let mut constraints = TextConstraints {
            max_width: Some(inner.size.width),
            wrap: self.wrap,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let mut metrics = cx
            .services
            .text()
            .measure(layout_text, self.text_style, constraints);
        let show_scrollbar = metrics.size.height.0 > inner.size.height.0;
        if show_scrollbar {
            constraints.max_width = Some(Px((inner.size.width.0 - scrollbar_w.0).max(0.0)));
            metrics = cx
                .services
                .text()
                .measure(layout_text, self.text_style, constraints);
        }

        self.metrics = Some(metrics);
        self.show_scrollbar = show_scrollbar;

        let Some(metrics) = self.metrics else {
            return Size::new(cx.available.width, self.min_height);
        };

        self.last_content_height = metrics.size.height;
        self.last_viewport_height = inner.size.height;
        self.clamp_offset(self.last_content_height, self.last_viewport_height);

        Size::new(
            cx.available.width,
            Px((metrics.size.height.0 + self.style.padding_y.0 * 2.0).max(self.min_height.0)),
        )
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        if cx.focus != Some(cx.node) && self.is_ime_composing() {
            self.clear_preedit();
        }
        self.last_bounds = cx.bounds;
        self.flush_pending_releases(cx.services);

        let inner = self.inner_bounds();

        let max_width = if self.show_scrollbar {
            Px((inner.size.width.0 - self.scrollbar_width.0).max(0.0))
        } else {
            inner.size.width
        };
        let constraints = TextConstraints {
            max_width: Some(max_width),
            wrap: self.wrap,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let key = PreparedKey {
            max_width_bits: max_width.0.to_bits(),
            wrap: self.wrap,
            scale_bits: cx.scale_factor.to_bits(),
            show_scrollbar: self.show_scrollbar,
        };

        if self.text_dirty || self.blob.is_none() || self.prepared_key != Some(key) {
            self.queue_release_blob();
            self.flush_pending_releases(cx.services);
            let layout_text = match self.layout_text() {
                Some(s) => std::borrow::Cow::Owned(s),
                None => std::borrow::Cow::Borrowed(self.text.as_str()),
            };
            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare(layout_text.as_ref(), self.text_style, constraints);
            self.blob = Some(blob);
            self.metrics = Some(metrics);
            self.prepared_key = Some(key);
            self.text_dirty = false;
        }

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.style.background,
            border: self.style.border,
            border_color: self.style.border_color,
            corner_radii: self.style.corner_radii,
        });

        if cx.focus == Some(cx.node)
            && crate::focus_visible::is_focus_visible(cx.app, cx.window)
            && let Some(mut ring) = self.style.focus_ring
        {
            ring.corner_radii = self.style.corner_radii;
            crate::paint::paint_focus_ring(cx.scene, DrawOrder(1), cx.bounds, ring);
        }

        let Some(blob) = self.blob else {
            return;
        };
        let Some(metrics) = self.metrics else {
            return;
        };

        let padded_inner = self.inner_bounds();
        self.last_content_height = metrics.size.height;
        self.last_viewport_height = padded_inner.size.height;
        self.clamp_offset(self.last_content_height, self.last_viewport_height);

        let inner = self.content_bounds();
        cx.scene.push(SceneOp::PushClipRect { rect: inner });

        let map_base_to_display = |idx: usize| -> usize {
            if self.preedit.is_empty() || idx <= self.caret {
                idx
            } else {
                idx + self.preedit.len()
            }
        };

        cx.services.selection_rects(
            blob,
            (
                map_base_to_display(self.selection_anchor),
                map_base_to_display(self.caret),
            ),
            &mut self.selection_rects,
        );
        for r in &self.selection_rects {
            let rect = Rect::new(
                fret_core::Point::new(
                    inner.origin.x + r.origin.x,
                    Px(inner.origin.y.0 + r.origin.y.0 - self.offset_y.0),
                ),
                r.size,
            );
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect,
                background: self.style.selection_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        if !self.preedit.is_empty() {
            let start = self.caret;
            let end = self.caret + self.preedit.len();
            cx.services
                .selection_rects(blob, (start, end), &mut self.preedit_rects);
            for r in &self.preedit_rects {
                let rect = Rect::new(
                    fret_core::Point::new(
                        inner.origin.x + r.origin.x,
                        Px(inner.origin.y.0 + r.origin.y.0 - self.offset_y.0),
                    ),
                    r.size,
                );
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(0),
                    rect,
                    background: self.style.preedit_bg_color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        } else {
            self.preedit_rects.clear();
        }

        let text_origin = fret_core::Point::new(
            inner.origin.x,
            Px(inner.origin.y.0 + metrics.baseline.0 - self.offset_y.0),
        );
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(0),
            origin: text_origin,
            text: blob,
            color: self.style.text_color,
        });

        if cx.focus != Some(cx.node) {
            self.ensure_caret_visible = true;
        }

        if cx.focus == Some(cx.node) {
            let caret_index = self.caret_display_index();
            let affinity = if self.preedit.is_empty() {
                self.affinity
            } else {
                CaretAffinity::Downstream
            };
            let caret = cx.services.caret_rect(blob, caret_index, affinity);
            let hairline = Px((1.0 / cx.scale_factor.max(1.0)).max(1.0 / 8.0));
            if self.ensure_caret_visible {
                let caret_top = caret.origin.y.0;
                let caret_bottom = caret.origin.y.0 + caret.size.height.0;
                let viewport_top = self.offset_y.0;
                let viewport_bottom = self.offset_y.0 + inner.size.height.0;
                let mut desired_offset = self.offset_y.0;
                if caret_top < viewport_top {
                    desired_offset = caret_top;
                } else if caret_bottom > viewport_bottom {
                    desired_offset = caret_bottom - inner.size.height.0;
                }
                if (desired_offset - self.offset_y.0).abs() > 0.01 {
                    self.offset_y = Px(desired_offset);
                    self.clamp_offset(self.last_content_height, self.last_viewport_height);
                    if let Some(window) = cx.window {
                        cx.app.request_redraw(window);
                    }
                }
                self.ensure_caret_visible = false;
            }

            let caret_rect = Rect::new(
                fret_core::Point::new(
                    inner.origin.x + caret.origin.x,
                    Px(inner.origin.y.0 + caret.origin.y.0 - self.offset_y.0),
                ),
                Size::new(Px(hairline.0.max(1.0)), caret.size.height),
            );

            if let Some(window) = cx.window {
                cx.app.push_effect(Effect::ImeAllow {
                    window,
                    enabled: true,
                });
                if self.last_sent_cursor != Some(caret_rect) {
                    self.last_sent_cursor = Some(caret_rect);
                    cx.app.push_effect(Effect::ImeSetCursorArea {
                        window,
                        rect: caret_rect,
                    });
                }
            }

            if !self.preedit_rects.is_empty() {
                for r in &self.preedit_rects {
                    if r.size.width.0 <= 0.0 || r.size.height.0 <= 0.0 {
                        continue;
                    }
                    let y = inner.origin.y.0 + r.origin.y.0 - self.offset_y.0 + r.size.height.0
                        - hairline.0;
                    let underline = Rect::new(
                        fret_core::Point::new(inner.origin.x + r.origin.x, Px(y)),
                        Size::new(Px(r.size.width.0.max(hairline.0)), hairline),
                    );
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect: underline,
                        background: self.style.preedit_underline_color,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(Px(0.0)),
                    });
                }
            }

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: caret_rect,
                background: self.style.caret_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        } else {
            self.last_sent_cursor = None;
        }

        cx.scene.push(SceneOp::PopClip);

        if let Some((track, thumb)) = self.scrollbar_geometry(cx.bounds) {
            let (track_bg, thumb_bg, thumb_hover_bg, radius) = {
                let theme = cx.theme();
                (
                    theme.colors.scrollbar_track,
                    theme.colors.scrollbar_thumb,
                    theme.colors.scrollbar_thumb_hover,
                    theme.metrics.radius_sm,
                )
            };
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(100),
                rect: track,
                background: track_bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(radius),
            });

            let thumb_bg = if self.dragging_thumb {
                thumb_hover_bg
            } else {
                thumb_bg
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(101),
                rect: thumb,
                background: thumb_bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(radius),
            });
        }
    }
}

pub struct BoundTextArea {
    model: Model<String>,
    last_revision: Option<u64>,
    dirty_since_sync: bool,
    area: TextArea,
}

impl BoundTextArea {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            last_revision: None,
            dirty_since_sync: false,
            area: TextArea::default(),
        }
    }

    pub fn with_text_style(mut self, style: TextStyle) -> Self {
        self.area.text_style = style;
        self.area.text_style_override = true;
        self.area.last_text_style_theme_revision = None;
        self.area.text_dirty = true;
        self
    }

    pub fn set_text_style(&mut self, style: TextStyle) {
        self.area.text_style = style;
        self.area.text_style_override = true;
        self.area.last_text_style_theme_revision = None;
        self.area.text_dirty = true;
    }

    pub fn with_min_height(mut self, min_height: Px) -> Self {
        self.area.min_height = min_height;
        self
    }

    pub fn set_min_height(&mut self, min_height: Px) {
        self.area.min_height = min_height;
    }

    pub fn with_style(mut self, style: TextAreaStyle) -> Self {
        self.area.style = style;
        self.area.style_override = true;
        self.area.last_theme_revision = None;
        self
    }

    pub fn set_style(&mut self, style: TextAreaStyle) {
        self.area.style = style;
        self.area.style_override = true;
        self.area.last_theme_revision = None;
    }

    fn sync_from_model<H: UiHost>(&mut self, app: &H, force: bool) {
        let revision = app.models().revision(self.model);
        if revision == self.last_revision {
            return;
        }
        self.last_revision = revision;

        let Some(text) = app.models().get(self.model) else {
            return;
        };

        if force || !self.dirty_since_sync {
            self.area.set_text(text.clone());
            self.dirty_since_sync = false;
        }
    }

    fn maybe_update_model<H: UiHost>(&mut self, app: &mut H) {
        let text = self.area.text.clone();
        if app
            .models_mut()
            .update(self.model, move |v| *v = text)
            .is_ok()
        {
            self.dirty_since_sync = false;
            self.last_revision = app.models().revision(self.model);
        }
    }
}

impl<H: UiHost> Widget<H> for BoundTextArea {
    fn is_focusable(&self) -> bool {
        true
    }

    fn is_text_input(&self) -> bool {
        true
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        <TextArea as Widget<H>>::cleanup_resources(&mut self.area, services);
    }

    fn semantics(&mut self, cx: &mut crate::widget::SemanticsCx<'_, H>) {
        self.area.semantics(cx);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if cx.focus != Some(cx.node) {
            self.sync_from_model(cx.app, false);
        }

        let before = self.area.text.clone();
        self.area.event(cx, event);
        if self.area.text != before {
            self.dirty_since_sync = true;
            self.maybe_update_model(cx.app);
            cx.invalidate_self(Invalidation::Layout);
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let force = !self.dirty_since_sync;
        self.sync_from_model(cx.app, force);
        self.area.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.area.paint(cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UiTree;
    use crate::test_host::TestHost;
    use fret_core::{
        AppWindowId, CaretAffinity, Event, PlatformCapabilities, Point, Px, Rect, Scene, Size,
        TextConstraints, TextMetrics,
        TextService, TextStyle,
    };
    use fret_runtime::Effect;

    #[derive(Default)]
    struct FakeTextService {}

    impl TextService for FakeTextService {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
            _constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn caret_rect(&mut self, _blob: fret_core::TextBlobId, index: usize, _affinity: CaretAffinity) -> Rect {
            Rect::new(
                Point::new(Px(index as f32), Px(0.0)),
                Size::new(Px(1.0), Px(10.0)),
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for FakeTextService {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for FakeTextService {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            false
        }
    }

    #[test]
    fn text_area_hover_sets_text_cursor_effect() {
        let window = AppWindowId::default();

        let mut ui = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TextArea::default());
        ui.set_root(root);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut text = FakeTextService::default();

        let _ = ui.layout(
            &mut app,
            &mut text,
            root,
            Size::new(Px(300.0), Px(200.0)),
            1.0,
        );
        let _ = app.take_effects();

        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(12.0), Px(12.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        let effects = app.take_effects();
        assert!(
            effects.iter().any(|e| matches!(
                e,
                Effect::CursorSetIcon { window: w, icon }
                    if *w == window && *icon == fret_core::CursorIcon::Text
            )),
            "expected a text cursor effect when hovering a text area"
        );
    }

    #[test]
    fn ime_cursor_area_moves_with_preedit_cursor() {
        let window = AppWindowId::default();

        let mut ui = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TextArea::default());
        ui.set_root(root);
        ui.set_focus(Some(root));

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut text = FakeTextService::default();

        let _ = ui.layout(
            &mut app,
            &mut text,
            root,
            Size::new(Px(300.0), Px(200.0)),
            1.0,
        );
        let _ = app.take_effects();

        fn paint_once(ui: &mut UiTree<TestHost>, root: fret_core::NodeId, app: &mut TestHost, text: &mut FakeTextService) -> f32 {
            let mut scene = Scene::default();
            ui.paint(
                app,
                text,
                root,
                Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(300.0), Px(200.0))),
                &mut scene,
                1.0,
            );
            app.take_effects()
                .into_iter()
                .find_map(|e| match e {
                    Effect::ImeSetCursorArea { rect, .. } => Some(rect.origin.x.0),
                    _ => None,
                })
                .expect("expected an IME cursor area effect")
        }

        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::Ime(fret_core::ImeEvent::Preedit {
                text: "abcd".to_string(),
                cursor: Some((0, 0)),
            }),
        );
        let x0 = paint_once(&mut ui, root, &mut app, &mut text);

        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::Ime(fret_core::ImeEvent::Preedit {
                text: "abcd".to_string(),
                cursor: Some((0, 2)),
            }),
        );
        let x2 = paint_once(&mut ui, root, &mut app, &mut text);

        assert!(
            (x2 - x0 - 2.0).abs() < 0.001,
            "expected IME cursor x to move by preedit prefix width"
        );
    }
}
