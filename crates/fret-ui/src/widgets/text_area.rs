use fret_app::Effect;
use fret_core::{
    CaretAffinity, Color, Corners, DrawOrder, Edges, Event, ImeEvent, MouseButton, Px, Rect,
    SceneOp, Size, TextConstraints, TextMetrics, TextStyle, TextWrap,
};

use crate::{CommandCx, EventCx, Invalidation, LayoutCx, PaintCx, Widget};

#[derive(Debug, Clone)]
pub struct TextAreaStyle {
    pub padding: Px,
    pub background: Color,
    pub border: Edges,
    pub border_color: Color,
    pub corner_radii: Corners,
    pub text_color: Color,
    pub selection_color: Color,
    pub caret_color: Color,
}

impl Default for TextAreaStyle {
    fn default() -> Self {
        Self {
            padding: Px(10.0),
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

    blob: Option<fret_core::TextBlobId>,
    metrics: Option<TextMetrics>,

    offset_y: Px,
    dragging_thumb: bool,
    drag_pointer_start_y: Px,
    drag_offset_start_y: Px,
    last_content_height: Px,
    last_viewport_height: Px,

    caret: usize,
    selection_anchor: usize,
    affinity: CaretAffinity,
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
            },
            wrap: TextWrap::Word,
            min_height: Px(0.0),
            style: TextAreaStyle::default(),
            blob: None,
            metrics: None,
            offset_y: Px(0.0),
            dragging_thumb: false,
            drag_pointer_start_y: Px(0.0),
            drag_offset_start_y: Px(0.0),
            last_content_height: Px(0.0),
            last_viewport_height: Px(0.0),
            caret: 0,
            selection_anchor: 0,
            affinity: CaretAffinity::Downstream,
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

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.caret = self.text.len();
        self.selection_anchor = self.caret;
        self
    }

    pub fn with_text_style(mut self, style: TextStyle) -> Self {
        self.text_style = style;
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
        self
    }

    pub fn offset_y(&self) -> Px {
        self.offset_y
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
        self.affinity = CaretAffinity::Downstream;
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
        self.affinity = CaretAffinity::Downstream;
    }

    fn request_clipboard_paste(&mut self, cx: &mut CommandCx<'_>) -> bool {
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

        let w = Px(10.0);
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
        let p = self.style.padding;
        Rect::new(
            fret_core::Point::new(self.last_bounds.origin.x + p, self.last_bounds.origin.y + p),
            Size::new(
                Px((self.last_bounds.size.width.0 - p.0 * 2.0).max(0.0)),
                Px((self.last_bounds.size.height.0 - p.0 * 2.0).max(0.0)),
            ),
        )
    }

    fn set_caret_from_point(&mut self, cx: &mut EventCx<'_>, point: fret_core::Point) {
        let Some(blob) = self.blob else {
            return;
        };
        let hit = cx.text.hit_test_point(blob, point);
        self.caret = hit.index;
        self.affinity = hit.affinity;
    }
}

impl Widget for TextArea {
    fn is_focusable(&self) -> bool {
        true
    }

    fn is_text_input(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        match event {
            Event::Pointer(fret_core::PointerEvent::Wheel { delta, .. }) => {
                self.offset_y = Px((self.offset_y.0 - delta.y.0).max(0.0));
                self.clamp_offset(self.last_content_height, self.last_viewport_height);
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

                if let Some((track, thumb)) = self.scrollbar_geometry(self.last_bounds) {
                    if track.contains(*position) {
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

                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
                }

                cx.request_focus(cx.node);
                cx.capture_pointer(cx.node);
                self.dragging_thumb = false;

                let inner = self.inner_bounds();
                let local =
                    fret_core::Point::new(position.x - inner.origin.x, position.y - inner.origin.y);
                let local = fret_core::Point::new(local.x, Px(local.y.0 + self.offset_y.0));
                self.set_caret_from_point(cx, local);
                self.selection_anchor = self.caret;

                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Pointer(fret_core::PointerEvent::Move {
                position, buttons, ..
            }) => {
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

                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let inner = self.inner_bounds();
                let local =
                    fret_core::Point::new(position.x - inner.origin.x, position.y - inner.origin.y);
                let local = fret_core::Point::new(local.x, Px(local.y.0 + self.offset_y.0));
                self.set_caret_from_point(cx, local);

                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Pointer(fret_core::PointerEvent::Up { button, .. }) => {
                if *button == MouseButton::Left && cx.captured == Some(cx.node) {
                    self.dragging_thumb = false;
                    cx.release_pointer_capture();
                }
            }
            Event::TextInput(text) => {
                if cx.focus != Some(cx.node) {
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
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
            }
            Event::ClipboardText(text) => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                if !normalized.is_empty() {
                    self.replace_selection(&normalized);
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
                    ImeEvent::Disabled => {}
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
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                    ImeEvent::Preedit { .. } => {}
                }
            }
            _ => {}
        }
    }

    fn command(&mut self, cx: &mut CommandCx<'_>, command: &fret_app::CommandId) -> bool {
        if cx.focus != Some(cx.node) {
            return false;
        }

        match command.as_str() {
            "text.clear" => {
                self.text.clear();
                self.caret = 0;
                self.selection_anchor = 0;
                self.affinity = CaretAffinity::Downstream;
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
                true
            }
            "text.select_all" => {
                self.selection_anchor = 0;
                self.caret = self.text.len();
                self.affinity = CaretAffinity::Downstream;
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
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_right" => {
                self.caret = Self::next_boundary(&self.text, self.caret);
                self.selection_anchor = self.caret;
                self.affinity = CaretAffinity::Downstream;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_word_left" => {
                self.caret = Self::move_word_left(&self.text, self.caret);
                self.selection_anchor = self.caret;
                self.affinity = CaretAffinity::Downstream;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_word_right" => {
                self.caret = Self::move_word_right(&self.text, self.caret);
                self.selection_anchor = self.caret;
                self.affinity = CaretAffinity::Downstream;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_home" => {
                self.caret = 0;
                self.selection_anchor = self.caret;
                self.affinity = CaretAffinity::Downstream;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_end" => {
                self.caret = self.text.len();
                self.selection_anchor = self.caret;
                self.affinity = CaretAffinity::Downstream;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_left" => {
                self.caret = Self::prev_boundary(&self.text, self.caret);
                self.affinity = CaretAffinity::Downstream;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_right" => {
                self.caret = Self::next_boundary(&self.text, self.caret);
                self.affinity = CaretAffinity::Downstream;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_word_left" => {
                self.caret = Self::move_word_left(&self.text, self.caret);
                self.affinity = CaretAffinity::Downstream;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_word_right" => {
                self.caret = Self::move_word_right(&self.text, self.caret);
                self.affinity = CaretAffinity::Downstream;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_home" => {
                self.caret = 0;
                self.affinity = CaretAffinity::Downstream;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_end" => {
                self.caret = self.text.len();
                self.affinity = CaretAffinity::Downstream;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.delete_backward" => {
                if self.delete_selection_if_any() {
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
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
                true
            }
            "text.delete_forward" => {
                if self.delete_selection_if_any() {
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
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
                true
            }
            "text.delete_word_backward" => {
                if self.delete_selection_if_any() {
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
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
                true
            }
            "text.delete_word_forward" => {
                if self.delete_selection_if_any() {
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
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
                true
            }
            _ => false,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        self.last_bounds = cx.bounds;

        self.caret = Self::clamp_to_boundary(&self.text, self.caret);
        self.selection_anchor = Self::clamp_to_boundary(&self.text, self.selection_anchor);

        let inner = self.inner_bounds();
        let constraints = TextConstraints {
            max_width: Some(inner.size.width),
            wrap: self.wrap,
            scale_factor: cx.scale_factor,
        };

        let old_blob = self.blob.take();
        let (blob, metrics) = cx.text.prepare(&self.text, self.text_style, constraints);
        self.blob = Some(blob);
        self.metrics = Some(metrics);

        if let Some(b) = old_blob {
            cx.text.release(b);
        }

        let Some(metrics) = self.metrics else {
            return Size::new(cx.available.width, self.min_height);
        };

        self.last_content_height = metrics.size.height;
        self.last_viewport_height = inner.size.height;
        self.clamp_offset(self.last_content_height, self.last_viewport_height);

        Size::new(
            cx.available.width,
            Px((metrics.size.height.0 + self.style.padding.0 * 2.0).max(self.min_height.0)),
        )
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.last_bounds = cx.bounds;

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.style.background,
            border: self.style.border,
            border_color: self.style.border_color,
            corner_radii: self.style.corner_radii,
        });

        let Some(blob) = self.blob else {
            return;
        };
        let Some(metrics) = self.metrics else {
            return;
        };

        let inner = self.inner_bounds();
        self.last_content_height = metrics.size.height;
        self.last_viewport_height = inner.size.height;
        self.clamp_offset(self.last_content_height, self.last_viewport_height);

        cx.scene.push(SceneOp::PushClipRect { rect: inner });

        cx.text.selection_rects(
            blob,
            (self.selection_anchor, self.caret),
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

        if cx.focus == Some(cx.node) {
            let caret = cx.text.caret_rect(blob, self.caret, self.affinity);
            let hairline = Px((1.0 / cx.scale_factor.max(1.0)).max(1.0 / 8.0));
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
                if self.last_sent_cursor.map_or(true, |r| r != caret_rect) {
                    self.last_sent_cursor = Some(caret_rect);
                    cx.app.push_effect(Effect::ImeSetCursorArea {
                        window,
                        rect: caret_rect,
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
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(100),
                rect: track,
                background: Color {
                    r: 0.10,
                    g: 0.10,
                    b: 0.11,
                    a: 0.9,
                },
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(6.0)),
            });

            let thumb_bg = if self.dragging_thumb {
                Color {
                    r: 0.55,
                    g: 0.55,
                    b: 0.58,
                    a: 0.9,
                }
            } else {
                Color {
                    r: 0.42,
                    g: 0.42,
                    b: 0.45,
                    a: 0.9,
                }
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(101),
                rect: thumb,
                background: thumb_bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(6.0)),
            });
        }
    }
}
