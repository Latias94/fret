use fret_core::{
    Color, DrawOrder, Event, FontId, MouseButton, Px, Rect, SceneOp, SemanticsRole, Size,
    TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};

use crate::widget::{CommandCx, EventCx, LayoutCx, PaintCx, Widget};
use crate::{Invalidation, UiHost};
use fret_core::KeyCode;
use fret_runtime::{CommandId, Effect, Model};

use crate::TextInputStyle;

#[derive(Debug)]
pub struct TextInput {
    text: String,
    caret: usize,
    selection_anchor: usize,
    preedit: String,
    preedit_cursor: Option<(usize, usize)>,
    ime_replace_range: Option<(usize, usize)>,
    ime_deduper: crate::text_edit::ime::Deduper,
    style: TextStyle,
    text_blob: Option<fret_core::TextBlobId>,
    text_metrics: Option<TextMetrics>,
    prefix_blob: Option<fret_core::TextBlobId>,
    prefix_metrics: Option<TextMetrics>,
    suffix_blob: Option<fret_core::TextBlobId>,
    suffix_metrics: Option<TextMetrics>,
    preedit_blob: Option<fret_core::TextBlobId>,
    preedit_metrics: Option<TextMetrics>,
    caret_stops: Vec<(usize, Px)>,
    pending_release: Vec<fret_core::TextBlobId>,
    prepared_scale_factor_bits: Option<u32>,
    last_bounds: Rect,
    last_sent_cursor: Option<Rect>,

    chrome_style: TextInputStyle,
    chrome_override: bool,
    last_theme_revision: Option<u64>,

    text_style_override: bool,
    last_text_style_theme_revision: Option<u64>,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            caret: 0,
            selection_anchor: 0,
            preedit: String::new(),
            preedit_cursor: None,
            ime_replace_range: None,
            ime_deduper: crate::text_edit::ime::Deduper::default(),
            style: TextStyle {
                font: FontId::default(),
                size: Px(13.0),
                ..Default::default()
            },
            text_blob: None,
            text_metrics: None,
            prefix_blob: None,
            prefix_metrics: None,
            suffix_blob: None,
            suffix_metrics: None,
            preedit_blob: None,
            preedit_metrics: None,
            caret_stops: Vec::new(),
            pending_release: Vec::new(),
            prepared_scale_factor_bits: None,
            last_bounds: Rect::default(),
            last_sent_cursor: None,

            chrome_style: TextInputStyle::default(),
            chrome_override: false,
            last_theme_revision: None,

            text_style_override: false,
            last_text_style_theme_revision: None,
        }
    }

    pub fn set_chrome_style(&mut self, style: TextInputStyle) {
        self.chrome_style = style;
        self.chrome_override = true;
        self.last_theme_revision = None;
    }

    pub fn set_text_style(&mut self, style: TextStyle) {
        if self.style == style {
            return;
        }
        self.queue_release_all_text_blobs();
        self.style = style;
        self.text_style_override = true;
        self.last_text_style_theme_revision = None;
        self.last_sent_cursor = None;
    }

    fn sync_chrome_from_theme(&mut self, theme: crate::ThemeSnapshot) {
        if self.chrome_override {
            return;
        }
        if self.last_theme_revision == Some(theme.revision) {
            return;
        }
        self.last_theme_revision = Some(theme.revision);
        self.chrome_style = TextInputStyle::from_theme(theme);
    }

    fn sync_text_style_from_theme(&mut self, theme: crate::ThemeSnapshot) {
        if self.text_style_override {
            return;
        }
        if self.last_text_style_theme_revision == Some(theme.revision) {
            return;
        }
        self.last_text_style_theme_revision = Some(theme.revision);

        let next_size = theme.metrics.font_size;
        if self.style.size != next_size {
            self.queue_release_all_text_blobs();
            self.style.size = next_size;
            self.prepared_scale_factor_bits = None;
            self.last_sent_cursor = None;
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.caret = self.text.len();
        self.selection_anchor = self.caret;
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.queue_release_all_text_blobs();
        self.text = text.into();
        self.caret = self.text.len();
        self.selection_anchor = self.caret;
        self.clear_ime_composition();
        self.ime_deduper = crate::text_edit::ime::Deduper::default();
        self.text_blob = None;
        self.text_metrics = None;
        self.prefix_blob = None;
        self.prefix_metrics = None;
        self.suffix_blob = None;
        self.suffix_metrics = None;
        self.preedit_blob = None;
        self.preedit_metrics = None;
        self.caret_stops.clear();
        self.last_sent_cursor = None;
    }

    fn is_ime_composing(&self) -> bool {
        crate::text_edit::ime::is_composing(&self.preedit, self.preedit_cursor)
    }

    fn preedit_cursor_end(&self) -> usize {
        crate::text_edit::ime::preedit_cursor_end(&self.preedit, self.preedit_cursor)
    }

    fn clear_ime_composition(&mut self) {
        crate::text_edit::ime::clear_state(
            &mut self.preedit,
            &mut self.preedit_cursor,
            &mut self.ime_replace_range,
        );
    }

    fn queue_release_all_text_blobs(&mut self) {
        for blob in [
            self.text_blob.take(),
            self.prefix_blob.take(),
            self.suffix_blob.take(),
            self.preedit_blob.take(),
        ]
        .into_iter()
        .flatten()
        {
            self.pending_release.push(blob);
        }
        self.text_metrics = None;
        self.prefix_metrics = None;
        self.suffix_metrics = None;
        self.preedit_metrics = None;
        self.caret_stops.clear();
        self.prepared_scale_factor_bits = None;
    }

    fn flush_pending_releases(&mut self, services: &mut dyn fret_core::UiServices) {
        for blob in self.pending_release.drain(..) {
            services.text().release(blob);
        }
    }
}

include!("bound.rs");

impl TextInput {
    fn is_focused<H: UiHost>(&self, cx: &EventCx<'_, H>) -> bool {
        cx.focus == Some(cx.node)
    }

    fn edit_state(&mut self) -> crate::text_edit::state::TextEditState<'_> {
        crate::text_edit::state::TextEditState::new(
            &mut self.text,
            &mut self.caret,
            &mut self.selection_anchor,
            &mut self.preedit,
            &mut self.preedit_cursor,
            &mut self.ime_replace_range,
        )
    }

    fn selection_range(&self) -> (usize, usize) {
        crate::text_edit::buffer::selection_range(self.selection_anchor, self.caret)
    }

    fn has_selection(&self) -> bool {
        crate::text_edit::buffer::has_selection(self.selection_anchor, self.caret)
    }

    fn replace_selection(&mut self, insert: &str) {
        self.edit_state().replace_selection(insert);
    }

    fn delete_selection_if_any(&mut self) -> bool {
        self.edit_state().delete_selection_if_any()
    }

    fn caret_rect<H: UiHost>(
        &self,
        cx: &mut PaintCx<'_, H>,
        bounds: Rect,
        scale_factor: f32,
    ) -> Rect {
        let padding_left = self.chrome_style.padding.left;
        let padding_top = self.chrome_style.padding.top;

        let caret_x = self
            .text_blob
            .map(|blob| cx.services.caret_x(blob, self.caret))
            .unwrap_or(Px(0.0));

        let mut x = padding_left + caret_x;

        if self.is_ime_composing() && !self.preedit.is_empty() {
            let cursor =
                crate::text_edit::ime::preedit_cursor_end(&self.preedit, self.preedit_cursor);
            let constraints = TextConstraints {
                max_width: Some(bounds.size.width),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };
            let pre_metrics =
                cx.services
                    .text()
                    .measure(&self.preedit[..cursor], self.style, constraints);
            x = x + pre_metrics.size.width;
        }

        let h = self.text_metrics.map(|m| m.size.height).unwrap_or(Px(16.0));
        let hairline = Px((1.0 / scale_factor.max(1.0)).max(1.0 / 8.0));
        Rect::new(
            fret_core::geometry::Point::new(x, padding_top),
            Size::new(Px(hairline.0.max(1.0)), Px(h.0.max(16.0))),
        )
    }

    fn caret_from_x(&self, x: Px) -> usize {
        if self.caret_stops.is_empty() {
            return 0;
        }
        let mut best = self.caret_stops[0].0;
        let mut best_dist = (self.caret_stops[0].1.0 - x.0).abs();
        for (idx, px) in &self.caret_stops {
            let dist = (px.0 - x.0).abs();
            if dist < best_dist {
                best = *idx;
                best_dist = dist;
            }
        }
        best
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for TextInput {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.queue_release_all_text_blobs();
        self.flush_pending_releases(services);
        self.text_metrics = None;
        self.prefix_metrics = None;
        self.suffix_metrics = None;
        self.preedit_metrics = None;
        self.caret_stops.clear();
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn is_text_input(&self) -> bool {
        true
    }

    fn semantics(&mut self, cx: &mut crate::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::TextField);
        cx.set_focusable(true);
        cx.set_value_editable(true);
        cx.set_text_selection_supported(true);

        let (value, text_selection, text_composition) = if self.is_ime_composing()
            && let Some(value) =
                crate::text_edit::ime::compose_text_at_caret(&self.text, self.caret, &self.preedit)
        {
            let caret_display = crate::text_edit::ime::caret_display_index(
                self.caret,
                &self.preedit,
                self.preedit_cursor,
            );
            (
                value,
                Some((caret_display as u32, caret_display as u32)),
                Some((self.caret as u32, (self.caret + self.preedit.len()) as u32)),
            )
        } else {
            (
                self.text().to_string(),
                Some((self.selection_anchor as u32, self.caret as u32)),
                None,
            )
        };

        cx.set_value(value);
        if let Some((anchor, focus)) = text_selection {
            cx.set_text_selection(anchor, focus);
        } else {
            cx.clear_text_selection();
        }
        if let Some((start, end)) = text_composition {
            cx.set_text_composition(start, end);
        } else {
            cx.clear_text_composition();
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let focused = self.is_focused(cx);

        match event {
            Event::SetTextSelection { anchor, focus } => {
                if !focused {
                    return;
                }
                let mut edit = self.edit_state();
                edit.clear_ime_composition();
                edit.set_selection_char_clamped(*anchor as usize, *focus as usize);

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
                cx.request_focus(cx.node);
                cx.capture_pointer(cx.node);
                self.last_sent_cursor = None;
                let padding = self.chrome_style.padding.left;
                let local_x =
                    Px((position.x.0 - (self.last_bounds.origin.x.0 + padding.0)).max(0.0));
                self.caret = self
                    .text_blob
                    .map(|blob| cx.services.hit_test_x(blob, local_x))
                    .unwrap_or_else(|| self.caret_from_x(local_x));
                self.selection_anchor = self.caret;
                self.clear_ime_composition();
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
            }
            Event::Pointer(fret_core::PointerEvent::Move {
                position, buttons, ..
            }) => {
                // Ensure the I-beam cursor while hovering (or dragging) inside the text field.
                if cx.captured == Some(cx.node) || self.last_bounds.contains(*position) {
                    cx.set_cursor_icon(fret_core::CursorIcon::Text);
                }
                if cx.captured != Some(cx.node) || !buttons.left {
                    return;
                }
                let padding = self.chrome_style.padding.left;
                let local_x =
                    Px((position.x.0 - (self.last_bounds.origin.x.0 + padding.0)).max(0.0));
                self.caret = self
                    .text_blob
                    .map(|blob| cx.services.hit_test_x(blob, local_x))
                    .unwrap_or_else(|| self.caret_from_x(local_x));
                self.clear_ime_composition();
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Pointer(fret_core::PointerEvent::Up { button, .. }) => {
                if cx.captured == Some(cx.node) && *button == MouseButton::Left {
                    cx.release_pointer_capture();
                }
            }
            Event::KeyDown { key, modifiers, .. } => {
                if !focused {
                    return;
                }

                if self.is_ime_composing() && !modifiers.ctrl && !modifiers.alt && !modifiers.meta {
                    // During IME composition (preedit), reserve common navigation/IME keys for the
                    // platform IME path. The runtime may still map these keys to focus traversal or
                    // global shortcuts, so we must explicitly stop propagation here (ADR 0012).
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
                        return;
                    }
                }

                if !self.is_ime_composing() {
                    match key {
                        fret_core::KeyCode::Backspace => {
                            let _ = self.edit_state().delete_backward_char();
                            cx.invalidate_self(Invalidation::Layout);
                            cx.request_redraw();
                        }
                        fret_core::KeyCode::Delete => {
                            let _ = self.edit_state().delete_forward_char();
                            cx.invalidate_self(Invalidation::Layout);
                            cx.request_redraw();
                        }
                        fret_core::KeyCode::ArrowLeft => {
                            let extend = modifiers.shift;
                            if modifiers.ctrl || modifiers.alt {
                                let _ = self.edit_state().move_word_left(extend);
                            } else {
                                let _ = self.edit_state().move_left(extend);
                            }
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                        fret_core::KeyCode::ArrowRight => {
                            let extend = modifiers.shift;
                            if modifiers.ctrl || modifiers.alt {
                                let _ = self.edit_state().move_word_right(extend);
                            } else {
                                let _ = self.edit_state().move_right(extend);
                            }
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                        fret_core::KeyCode::Home => {
                            let _ = self.edit_state().move_home(modifiers.shift);
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                        fret_core::KeyCode::End => {
                            let _ = self.edit_state().move_end(modifiers.shift);
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                        _ => {}
                    }
                }
            }
            Event::TextInput(text) => {
                if !focused {
                    return;
                }
                let tick = cx.app.tick_id();
                if self
                    .ime_deduper
                    .ignore_text_input_after_ime_commit(tick, text.as_str())
                {
                    return;
                }
                self.ime_deduper.record_text_input(tick, text.as_str());

                if !self.is_ime_composing() {
                    self.replace_selection(text);
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
            }
            Event::ClipboardText(text) => {
                if !focused {
                    return;
                }
                if !self.is_ime_composing() {
                    if let Some(sanitized) =
                        crate::text_edit::clipboard::normalize_single_line(text.as_str())
                    {
                        self.replace_selection(&sanitized);
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                }
            }
            Event::Ime(ime) => {
                if !focused {
                    return;
                }
                let tick = cx.app.tick_id();
                let result = crate::text_edit::ime::apply_event(
                    ime,
                    tick,
                    false,
                    &mut self.ime_deduper,
                    &mut self.text,
                    &mut self.caret,
                    &mut self.selection_anchor,
                    &mut self.preedit,
                    &mut self.preedit_cursor,
                    &mut self.ime_replace_range,
                );
                if result != crate::text_edit::ime::ApplyResult::Noop {
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        if cx.focus != Some(cx.node) {
            return false;
        }

        match command.as_str() {
            "text.clear" => {
                self.text.clear();
                self.clear_ime_composition();
                self.caret = 0;
                self.selection_anchor = 0;
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
                true
            }
            "text.select_all" => {
                self.selection_anchor = 0;
                self.caret = self.text.len();
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
                    self.clear_ime_composition();
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
                true
            }
            "text.paste" => {
                let Some(window) = cx.window else {
                    return true;
                };
                cx.app.push_effect(Effect::ClipboardGetText { window });
                true
            }
            "text.move_left" => {
                let _ = self.edit_state().move_left(false);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_right" => {
                let _ = self.edit_state().move_right(false);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_word_left" => {
                let _ = self.edit_state().move_word_left(false);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_word_right" => {
                let _ = self.edit_state().move_word_right(false);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_home" => {
                let _ = self.edit_state().move_home(false);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_end" => {
                let _ = self.edit_state().move_end(false);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_up" => {
                let _ = self.edit_state().move_home(false);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_down" => {
                let _ = self.edit_state().move_end(false);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_left" => {
                let _ = self.edit_state().move_left(true);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_right" => {
                let _ = self.edit_state().move_right(true);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_word_left" => {
                let _ = self.edit_state().move_word_left(true);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_word_right" => {
                let _ = self.edit_state().move_word_right(true);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_home" => {
                let _ = self.edit_state().move_home(true);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_end" => {
                let _ = self.edit_state().move_end(true);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_up" => {
                let _ = self.edit_state().move_home(true);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_down" => {
                let _ = self.edit_state().move_end(true);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.delete_backward" => {
                if !self.is_ime_composing() {
                    let _ = self.edit_state().delete_backward_char();
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
                true
            }
            "text.delete_forward" => {
                if !self.is_ime_composing() {
                    let _ = self.edit_state().delete_forward_char();
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
                true
            }
            "text.delete_word_backward" => {
                if !self.is_ime_composing() {
                    let _ = self.edit_state().delete_word_backward();
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
                true
            }
            "text.delete_word_forward" => {
                if !self.is_ime_composing() {
                    let _ = self.edit_state().delete_word_forward();
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
                true
            }
            _ => false,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.last_bounds = cx.bounds;

        self.edit_state().clamp_caret_and_anchor_to_char_boundary();

        let theme = cx.theme().snapshot();
        self.sync_chrome_from_theme(theme);
        self.sync_text_style_from_theme(theme);

        let base_constraints = TextConstraints {
            max_width: Some(cx.available.width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let metrics = cx
            .services
            .text()
            .measure(&self.text, self.style, base_constraints);
        self.text_metrics = Some(metrics);

        let base_h = self.text_metrics.map(|m| m.size.height.0).unwrap_or(0.0);
        let chrome = &self.chrome_style;
        let border_h = chrome.border.top.0.max(0.0) + chrome.border.bottom.0.max(0.0);
        let pad_h = chrome.padding.top.0.max(0.0) + chrome.padding.bottom.0.max(0.0);
        let h = Px((base_h + pad_h + border_h).max(0.0));
        Size::new(cx.available.width, h)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.flush_pending_releases(cx.services);

        let Some(window) = cx.window else {
            return;
        };

        let theme = cx.theme().snapshot();
        self.sync_chrome_from_theme(theme);
        self.sync_text_style_from_theme(theme);
        let focused = cx.focus == Some(cx.node);
        if !focused && self.is_ime_composing() {
            self.clear_ime_composition();
        }
        let border_color = if focused && self.chrome_style.focus_ring.is_some() {
            self.chrome_style.border_color
        } else if focused {
            self.chrome_style.border_color_focused
        } else {
            self.chrome_style.border_color
        };

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        let scale_bits = cx.scale_factor.to_bits();
        if self.prepared_scale_factor_bits != Some(scale_bits) {
            self.queue_release_all_text_blobs();
            self.flush_pending_releases(cx.services);
            self.prepared_scale_factor_bits = Some(scale_bits);
        }

        if self.text_blob.is_none() {
            let (blob, metrics) = cx
                .services
                .text()
                .prepare(&self.text, self.style, constraints);
            self.text_blob = Some(blob);
            self.text_metrics = Some(metrics);
            cx.services.caret_stops(blob, &mut self.caret_stops);
        }

        if self.preedit.is_empty() {
            if self.prefix_blob.is_some()
                || self.suffix_blob.is_some()
                || self.preedit_blob.is_some()
            {
                self.queue_release_all_text_blobs();
                // The call above also clears `text_blob`, so re-prepare it.
                self.flush_pending_releases(cx.services);
                let (blob, metrics) =
                    cx.services
                        .text()
                        .prepare(&self.text, self.style, constraints);
                self.text_blob = Some(blob);
                self.text_metrics = Some(metrics);
                cx.services.caret_stops(blob, &mut self.caret_stops);
            }
        } else if self.prefix_blob.is_none()
            || self.suffix_blob.is_none()
            || self.preedit_blob.is_none()
        {
            // Preedit mode: render prefix/preedit/suffix as separate runs.
            self.queue_release_all_text_blobs();
            self.flush_pending_releases(cx.services);

            let (blob, metrics) = cx
                .services
                .text()
                .prepare(&self.text, self.style, constraints);
            self.text_blob = Some(blob);
            self.text_metrics = Some(metrics);
            cx.services.caret_stops(blob, &mut self.caret_stops);

            let (prefix_blob, prefix_metrics) =
                cx.services
                    .text()
                    .prepare(&self.text[..self.caret], self.style, constraints);
            let (suffix_blob, suffix_metrics) =
                cx.services
                    .text()
                    .prepare(&self.text[self.caret..], self.style, constraints);
            let (pre_blob, pre_metrics) =
                cx.services
                    .text()
                    .prepare(&self.preedit, self.style, constraints);

            self.prefix_blob = Some(prefix_blob);
            self.prefix_metrics = Some(prefix_metrics);
            self.suffix_blob = Some(suffix_blob);
            self.suffix_metrics = Some(suffix_metrics);
            self.preedit_blob = Some(pre_blob);
            self.preedit_metrics = Some(pre_metrics);
        }

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.chrome_style.background,
            border: self.chrome_style.border,
            border_color,
            corner_radii: self.chrome_style.corner_radii,
        });

        if focused
            && crate::focus_visible::is_focus_visible(cx.app, cx.window)
            && let Some(mut ring) = self.chrome_style.focus_ring
        {
            ring.corner_radii = self.chrome_style.corner_radii;
            crate::paint::paint_focus_ring(cx.scene, DrawOrder(1), cx.bounds, ring);
        }

        let padding_left = self.chrome_style.padding.left;
        let _padding_right = self.chrome_style.padding.right;
        let padding_top = self.chrome_style.padding.top;
        let padding_bottom = self.chrome_style.padding.bottom;
        if self.has_selection() && !self.is_ime_composing() {
            let (a, b) = self.selection_range();
            let start_x = self
                .text_blob
                .map(|blob| cx.services.caret_x(blob, a))
                .unwrap_or(Px(0.0));
            let end_x = self
                .text_blob
                .map(|blob| cx.services.caret_x(blob, b))
                .unwrap_or(Px(0.0));

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: Rect::new(
                    fret_core::geometry::Point::new(
                        cx.bounds.origin.x + padding_left + start_x,
                        cx.bounds.origin.y + padding_top,
                    ),
                    Size::new(
                        Px((end_x.0 - start_x.0).max(0.0)),
                        Px((cx.bounds.size.height.0 - padding_top.0 - padding_bottom.0).max(0.0)),
                    ),
                ),
                background: self.chrome_style.selection_color,
                border: fret_core::geometry::Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: self.chrome_style.corner_radii,
            });
        }
        let base_origin = if let Some(metrics) = self.text_metrics {
            fret_core::geometry::Point::new(
                cx.bounds.origin.x + padding_left,
                cx.bounds.origin.y + padding_top + metrics.baseline,
            )
        } else {
            fret_core::geometry::Point::new(
                cx.bounds.origin.x + padding_left,
                cx.bounds.origin.y + padding_top + Px(10.0),
            )
        };

        if self.preedit.is_empty() {
            if let Some(blob) = self.text_blob {
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(0),
                    origin: base_origin,
                    text: blob,
                    color: self.chrome_style.text_color,
                });
            }
        } else {
            let prefix_w = self
                .text_blob
                .map(|blob| cx.services.caret_x(blob, self.caret))
                .unwrap_or(Px(0.0));
            let pre_w = self
                .preedit_metrics
                .map(|m| m.size.width)
                .unwrap_or(Px(0.0));

            if let Some(blob) = self.prefix_blob {
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(0),
                    origin: base_origin,
                    text: blob,
                    color: self.chrome_style.text_color,
                });
            }
            if let Some(pre_blob) = self.preedit_blob {
                let pre_origin =
                    fret_core::geometry::Point::new(base_origin.x + prefix_w, base_origin.y);
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(0),
                    origin: pre_origin,
                    text: pre_blob,
                    color: self.chrome_style.preedit_color,
                });
            }
            if let Some(suffix_blob) = self.suffix_blob {
                let suffix_origin = fret_core::geometry::Point::new(
                    base_origin.x + prefix_w + pre_w,
                    base_origin.y,
                );
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(0),
                    origin: suffix_origin,
                    text: suffix_blob,
                    color: self.chrome_style.text_color,
                });
            }
        }

        if !focused {
            return;
        }

        let caret_local = self.caret_rect(cx, cx.bounds, cx.scale_factor);
        let caret = Rect::new(
            fret_core::Point::new(
                cx.bounds.origin.x + caret_local.origin.x,
                cx.bounds.origin.y + caret_local.origin.y,
            ),
            caret_local.size,
        );
        if self.last_sent_cursor != Some(caret) {
            self.last_sent_cursor = Some(caret);
            cx.app.push_effect(Effect::ImeSetCursorArea {
                window,
                rect: caret,
            });
        }

        // Draw caret as a thin quad (always visible in MVP).
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: caret,
            background: self.chrome_style.caret_color,
            border: fret_core::geometry::Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::geometry::Corners::all(Px(1.0)),
        });
    }
}

#[cfg(test)]
include!("tests.rs");
