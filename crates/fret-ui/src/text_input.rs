use fret_core::{
    Color, DrawOrder, Event, FontId, ImeEvent, MouseButton, Px, Rect, SceneOp, SemanticsRole, Size,
    TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};

use crate::{EventCx, Invalidation, LayoutCx, PaintCx, UiHost, Widget};
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
    last_text_input_tick: Option<fret_core::TickId>,
    last_text_input_text: Option<String>,
    last_ime_commit_tick: Option<fret_core::TickId>,
    last_ime_commit_text: Option<String>,

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
            last_text_input_tick: None,
            last_text_input_text: None,
            last_ime_commit_tick: None,
            last_ime_commit_text: None,

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
        !self.preedit.is_empty() || self.preedit_cursor.is_some()
    }

    fn clear_ime_composition(&mut self) {
        self.preedit.clear();
        self.preedit_cursor = None;
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

pub struct BoundTextInput {
    model: Model<String>,
    last_revision: Option<u64>,
    dirty_since_sync: bool,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    input: TextInput,
}

impl BoundTextInput {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            last_revision: None,
            dirty_since_sync: false,
            submit_command: None,
            cancel_command: None,
            input: TextInput::new(),
        }
    }

    pub fn model_id(&self) -> fret_runtime::ModelId {
        self.model.id()
    }

    pub fn set_model(&mut self, model: Model<String>) {
        self.model = model;
        self.last_revision = None;
        self.dirty_since_sync = false;
    }

    pub fn with_submit_command(mut self, command: CommandId) -> Self {
        self.submit_command = Some(command);
        self
    }

    pub fn with_cancel_command(mut self, command: CommandId) -> Self {
        self.cancel_command = Some(command);
        self
    }

    pub fn set_submit_command(&mut self, command: Option<CommandId>) {
        self.submit_command = command;
    }

    pub fn set_cancel_command(&mut self, command: Option<CommandId>) {
        self.cancel_command = command;
    }

    pub fn with_chrome_style(mut self, style: TextInputStyle) -> Self {
        self.input.set_chrome_style(style);
        self
    }

    pub fn with_text_style(mut self, style: TextStyle) -> Self {
        self.input.set_text_style(style);
        self
    }

    pub fn set_chrome_style(&mut self, style: TextInputStyle) {
        self.input.set_chrome_style(style);
    }

    pub fn set_text_style(&mut self, style: TextStyle) {
        self.input.set_text_style(style);
    }

    pub fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.input.queue_release_all_text_blobs();
        self.input.flush_pending_releases(services);
        self.input.text_metrics = None;
        self.input.prefix_metrics = None;
        self.input.suffix_metrics = None;
        self.input.preedit_metrics = None;
        self.input.caret_stops.clear();
    }

    pub fn semantics<H: UiHost>(&mut self, cx: &mut crate::widget::SemanticsCx<'_, H>) {
        self.input.semantics(cx);
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
            self.input.set_text(text.clone());
            self.dirty_since_sync = false;
        }
    }

    fn maybe_update_model<H: UiHost>(&mut self, app: &mut H) {
        let text = self.input.text().to_string();
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

impl<H: UiHost> Widget<H> for BoundTextInput {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if cx.focus != Some(cx.node) {
            self.sync_from_model(cx.app, false);
        }

        if cx.focus == Some(cx.node)
            && let Event::KeyDown { key, modifiers, .. } = event
            && !modifiers.shift
            && !modifiers.ctrl
            && !modifiers.alt
            && !modifiers.meta
        {
            match key {
                KeyCode::Enter => {
                    if let Some(cmd) = self.submit_command.clone() {
                        cx.dispatch_command(cmd);
                        cx.stop_propagation();
                        return;
                    }
                }
                KeyCode::Escape => {
                    if let Some(cmd) = self.cancel_command.clone() {
                        cx.dispatch_command(cmd);
                        cx.stop_propagation();
                        return;
                    }
                }
                _ => {}
            }
        }

        let before = self.input.text().to_string();
        self.input.event(cx, event);
        if self.input.text() != before {
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
        self.input.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.input.paint(cx);
    }
}

impl TextInput {
    fn is_focused<H: UiHost>(&self, cx: &EventCx<'_, H>) -> bool {
        cx.focus == Some(cx.node)
    }

    fn selection_range(&self) -> (usize, usize) {
        let a = self.selection_anchor.min(self.caret);
        let b = self.selection_anchor.max(self.caret);
        (a, b)
    }

    fn has_selection(&self) -> bool {
        self.selection_anchor != self.caret
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
    }

    fn delete_selection_if_any(&mut self) -> bool {
        let (a, b) = self.selection_range();
        if a == b {
            return false;
        }
        self.text.replace_range(a..b, "");
        self.caret = a;
        self.selection_anchor = self.caret;
        true
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
            let cursor = self
                .preedit_cursor
                .map(|(_, end)| end.min(self.preedit.len()))
                .unwrap_or(self.preedit.len());
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
        cx.set_value(self.text().to_string());
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let focused = self.is_focused(cx);
        let Some(window) = cx.window else {
            return;
        };

        match event {
            Event::Pointer(fret_core::PointerEvent::Down {
                button, position, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                cx.request_focus(cx.node);
                cx.capture_pointer(cx.node);
                cx.app.push_effect(Effect::ImeAllow {
                    window,
                    enabled: true,
                });
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

                if self.is_ime_composing() && !modifiers.ctrl && !modifiers.alt && !modifiers.meta
                {
                    // During IME composition (preedit), reserve common navigation/IME keys for the
                    // platform IME path. The runtime may still map these keys to focus traversal or
                    // global shortcuts, so we must explicitly stop propagation here (ADR 0012).
                    if matches!(
                        key,
                        fret_core::KeyCode::Tab
                            | fret_core::KeyCode::Space
                            | fret_core::KeyCode::Enter
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
                            if !self.delete_selection_if_any() {
                                let prev = Self::prev_boundary(&self.text, self.caret);
                                self.text.replace_range(prev..self.caret, "");
                                self.caret = prev;
                                self.selection_anchor = self.caret;
                            }
                            cx.invalidate_self(Invalidation::Layout);
                            cx.request_redraw();
                        }
                        fret_core::KeyCode::Delete => {
                            if !self.delete_selection_if_any() && self.caret < self.text.len() {
                                let next = Self::next_boundary(&self.text, self.caret);
                                self.text.replace_range(self.caret..next, "");
                            }
                            cx.invalidate_self(Invalidation::Layout);
                            cx.request_redraw();
                        }
                        fret_core::KeyCode::ArrowLeft => {
                            let next = if modifiers.ctrl || modifiers.alt {
                                Self::move_word_left(&self.text, self.caret)
                            } else {
                                Self::prev_boundary(&self.text, self.caret)
                            };
                            self.caret = next;
                            if !modifiers.shift {
                                self.selection_anchor = self.caret;
                            }
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                        fret_core::KeyCode::ArrowRight => {
                            let next = if modifiers.ctrl || modifiers.alt {
                                Self::move_word_right(&self.text, self.caret)
                            } else {
                                Self::next_boundary(&self.text, self.caret)
                            };
                            self.caret = next;
                            if !modifiers.shift {
                                self.selection_anchor = self.caret;
                            }
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                        fret_core::KeyCode::Home => {
                            self.caret = 0;
                            if !modifiers.shift {
                                self.selection_anchor = self.caret;
                            }
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                        fret_core::KeyCode::End => {
                            self.caret = self.text.len();
                            if !modifiers.shift {
                                self.selection_anchor = self.caret;
                            }
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
                if self.last_ime_commit_tick == Some(tick)
                    && self.last_ime_commit_text.as_deref() == Some(text.as_str())
                {
                    return;
                }
                self.last_text_input_tick = Some(tick);
                self.last_text_input_text = Some(text.clone());

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
                    let sanitized = text.replace(['\n', '\r'], " ");
                    if !sanitized.is_empty() {
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
                match ime {
                    ImeEvent::Enabled => {}
                    ImeEvent::Disabled => {
                        self.clear_ime_composition();
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                    ImeEvent::Commit(text) => {
                        let tick = cx.app.tick_id();
                        if self.last_text_input_tick == Some(tick)
                            && self.last_text_input_text.as_deref() == Some(text.as_str())
                        {
                            self.preedit.clear();
                            cx.invalidate_self(Invalidation::Layout);
                            cx.request_redraw();
                            return;
                        }
                        self.last_ime_commit_tick = Some(tick);
                        self.last_ime_commit_text = Some(text.clone());

                        self.replace_selection(text);
                        self.clear_ime_composition();
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                    ImeEvent::Preedit { text, cursor } => {
                        self.preedit = text.clone();
                        self.preedit_cursor = *cursor;
                        self.selection_anchor = self.caret;
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }

    fn command(&mut self, cx: &mut crate::CommandCx<'_, H>, command: &CommandId) -> bool {
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
                self.caret = Self::prev_boundary(&self.text, self.caret);
                self.selection_anchor = self.caret;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_right" => {
                self.caret = Self::next_boundary(&self.text, self.caret);
                self.selection_anchor = self.caret;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_word_left" => {
                self.caret = Self::move_word_left(&self.text, self.caret);
                self.selection_anchor = self.caret;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_word_right" => {
                self.caret = Self::move_word_right(&self.text, self.caret);
                self.selection_anchor = self.caret;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_home" => {
                self.caret = 0;
                self.selection_anchor = self.caret;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_end" => {
                self.caret = self.text.len();
                self.selection_anchor = self.caret;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_up" => {
                self.caret = 0;
                self.selection_anchor = self.caret;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.move_down" => {
                self.caret = self.text.len();
                self.selection_anchor = self.caret;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_left" => {
                self.caret = Self::prev_boundary(&self.text, self.caret);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_right" => {
                self.caret = Self::next_boundary(&self.text, self.caret);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_word_left" => {
                self.caret = Self::move_word_left(&self.text, self.caret);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_word_right" => {
                self.caret = Self::move_word_right(&self.text, self.caret);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_home" => {
                self.caret = 0;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_end" => {
                self.caret = self.text.len();
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_up" => {
                self.caret = 0;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.select_down" => {
                self.caret = self.text.len();
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                true
            }
            "text.delete_backward" => {
                if !self.is_ime_composing() {
                    if !self.delete_selection_if_any() {
                        let prev = Self::prev_boundary(&self.text, self.caret);
                        self.text.replace_range(prev..self.caret, "");
                        self.caret = prev;
                        self.selection_anchor = self.caret;
                    }
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
                true
            }
            "text.delete_forward" => {
                if !self.is_ime_composing() {
                    if !self.delete_selection_if_any() && self.caret < self.text.len() {
                        let next = Self::next_boundary(&self.text, self.caret);
                        self.text.replace_range(self.caret..next, "");
                    }
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
                true
            }
            "text.delete_word_backward" => {
                if !self.is_ime_composing() {
                    if !self.delete_selection_if_any() {
                        let prev = Self::move_word_left(&self.text, self.caret);
                        self.text.replace_range(prev..self.caret, "");
                        self.caret = prev;
                        self.selection_anchor = self.caret;
                    }
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
                true
            }
            "text.delete_word_forward" => {
                if !self.is_ime_composing() {
                    if !self.delete_selection_if_any() {
                        let next = Self::move_word_right(&self.text, self.caret);
                        self.text.replace_range(self.caret..next, "");
                        self.selection_anchor = self.caret;
                    }
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

        self.caret = Self::clamp_to_boundary(&self.text, self.caret);
        self.selection_anchor = Self::clamp_to_boundary(&self.text, self.selection_anchor);

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

        cx.app.push_effect(Effect::ImeAllow {
            window,
            enabled: true,
        });

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
mod tests {
    use super::*;
    use crate::UiTree;
    use crate::test_host::TestHost;
    use fret_core::{
        AppWindowId, Event, PlatformCapabilities, Point, Px, Rect, Size, TextConstraints, TextMetrics,
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
    fn text_input_hover_sets_text_cursor_effect() {
        let window = AppWindowId::default();

        let mut ui = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TextInput::new());
        ui.set_root(root);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut text = FakeTextService::default();

        let _ = ui.layout(
            &mut app,
            &mut text,
            root,
            Size::new(Px(200.0), Px(40.0)),
            1.0,
        );
        let _ = app.take_effects();

        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(10.0), Px(10.0)),
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
            "expected a text cursor effect when hovering a text input"
        );
    }

    #[derive(Default)]
    struct ImeTextService {}

    impl TextService for ImeTextService {
        fn prepare(
            &mut self,
            text: &str,
            _style: TextStyle,
            _constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(text.len() as f32), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn caret_x(&mut self, _blob: fret_core::TextBlobId, index: usize) -> Px {
            Px(index as f32)
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for ImeTextService {
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

    impl fret_core::SvgService for ImeTextService {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            false
        }
    }

    #[test]
    fn ime_cursor_area_moves_with_preedit_cursor() {
        let window = AppWindowId::default();

        let mut ui = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TextInput::new().with_text("hello"));
        ui.set_root(root);
        ui.set_focus(Some(root));

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut text = ImeTextService::default();

        let _ = ui.layout(
            &mut app,
            &mut text,
            root,
            Size::new(Px(300.0), Px(40.0)),
            1.0,
        );
        let _ = app.take_effects();

        fn paint_once(
            ui: &mut UiTree<TestHost>,
            root: fret_core::NodeId,
            app: &mut TestHost,
            text: &mut ImeTextService,
        ) -> f32 {
            let mut scene = fret_core::Scene::default();
            ui.paint(
                app,
                text,
                root,
                Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(300.0), Px(40.0))),
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
            &Event::Ime(ImeEvent::Preedit {
                text: "abcd".to_string(),
                cursor: Some((0, 0)),
            }),
        );
        let x0 = paint_once(&mut ui, root, &mut app, &mut text);

        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::Ime(ImeEvent::Preedit {
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
