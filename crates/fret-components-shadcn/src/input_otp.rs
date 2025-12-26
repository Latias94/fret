use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, FontId, KeyCode, MouseButton, Point, Px,
    Rect, SceneOp, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::{
    EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget, widget::SemanticsCx,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputOtpPattern {
    Digits,
    DigitsAndChars,
    Any,
}

impl Default for InputOtpPattern {
    fn default() -> Self {
        Self::DigitsAndChars
    }
}

impl InputOtpPattern {
    fn allows(self, ch: char) -> bool {
        match self {
            Self::Any => !ch.is_control(),
            Self::Digits => ch.is_ascii_digit(),
            Self::DigitsAndChars => ch.is_ascii_digit() || ch.is_ascii_alphabetic(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InputOTPSlot {
    pub index: usize,
}

impl InputOTPSlot {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

#[derive(Debug, Clone)]
pub struct InputOTPGroup {
    slots: Vec<InputOTPSlot>,
}

impl InputOTPGroup {
    pub fn new() -> Self {
        Self { slots: Vec::new() }
    }

    pub fn slot(mut self, index: usize) -> Self {
        self.slots.push(InputOTPSlot::new(index));
        self
    }
}

impl Default for InputOTPGroup {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct InputOTPSeparator;

impl InputOTPSeparator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for InputOTPSeparator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
enum InputOtpPart {
    Group(InputOTPGroup),
    Separator(InputOTPSeparator),
}

#[derive(Debug, Clone)]
struct PreparedGlyph {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
    ch: char,
}

fn slot_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.input_otp.slot_size")
        .unwrap_or(Px(36.0))
}

fn gap(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.input_otp.gap")
        .unwrap_or(Px(8.0))
}

fn separator_w(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.input_otp.separator_w")
        .unwrap_or(Px(16.0))
}

fn radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.input_otp.radius")
        .or_else(|| theme.metric_by_key("radius"))
        .unwrap_or(theme.metrics.radius_sm)
}

fn border_w(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.input_otp.border_w")
        .unwrap_or(Px(1.0))
}

fn bg(theme: &Theme) -> Color {
    theme
        .color_by_key("input.background")
        .unwrap_or(theme.colors.panel_background)
}

fn border_color(theme: &Theme) -> Color {
    theme
        .color_by_key("input.border")
        .unwrap_or(theme.colors.panel_border)
}

fn fg(theme: &Theme) -> Color {
    theme
        .color_by_key("input.foreground")
        .unwrap_or(theme.colors.text_primary)
}

fn caret_color(theme: &Theme) -> Color {
    theme
        .color_by_key("caret")
        .unwrap_or(theme.colors.text_primary)
}

fn ring_border(theme: &Theme) -> Color {
    theme
        .color_by_key("ring")
        .unwrap_or(theme.colors.focus_ring)
}

fn glyph_style(theme: &Theme) -> TextStyle {
    let size = theme
        .metric_by_key("component.input_otp.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.input_otp.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or(theme.metrics.font_line_height);
    TextStyle {
        font: FontId::default(),
        size,
        line_height: Some(line_height),
        ..Default::default()
    }
}

fn value_to_chars(value: &str) -> Vec<char> {
    value.chars().collect()
}

fn chars_to_value(chars: &[char]) -> String {
    chars.iter().collect()
}

fn apply_text_input(
    value: &str,
    caret: usize,
    max_len: usize,
    pattern: InputOtpPattern,
    input: &str,
) -> (String, usize) {
    let mut chars = value_to_chars(value);
    let mut caret = caret.min(max_len);
    if caret > chars.len() {
        caret = chars.len();
    }

    for ch in input.chars() {
        if chars.len() >= max_len {
            break;
        }
        if !pattern.allows(ch) {
            continue;
        }

        if caret < chars.len() {
            chars[caret] = ch;
        } else {
            chars.push(ch);
        }
        caret = (caret + 1).min(max_len);
    }

    (chars_to_value(&chars), caret)
}

fn apply_backspace(value: &str, caret: usize) -> (String, usize) {
    let mut chars = value_to_chars(value);
    let mut caret = caret.min(chars.len());
    if caret == 0 {
        return (value.to_string(), 0);
    }
    chars.remove(caret - 1);
    caret -= 1;
    (chars_to_value(&chars), caret)
}

fn apply_delete(value: &str, caret: usize) -> (String, usize) {
    let mut chars = value_to_chars(value);
    let caret = caret.min(chars.len());
    if caret >= chars.len() {
        return (value.to_string(), caret);
    }
    chars.remove(caret);
    (chars_to_value(&chars), caret)
}

/// shadcn/ui v4 Input OTP (prototype).
///
/// Upstream is built on `input-otp` (DOM). In Fret, this is implemented as a retained widget that:
/// - is focusable + text-input capable,
/// - draws slot boxes,
/// - updates a `Model<String>` value.
///
/// Non-goals (for now): animated blinking caret, IME-specific behaviors, paste heuristics beyond
/// sequential fill.
pub struct InputOTP {
    model: Model<String>,
    max_length: usize,
    parts: Vec<InputOtpPart>,
    disabled: bool,
    pattern: InputOtpPattern,
    on_complete: Option<CommandId>,

    caret: usize,
    slot_rects: Vec<Rect>,

    prepared: Vec<Option<PreparedGlyph>>,
    prepared_scale_bits: Option<u32>,
    prepared_theme_revision: Option<u64>,
}

impl InputOTP {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            max_length: 6,
            parts: Vec::new(),
            disabled: false,
            pattern: InputOtpPattern::default(),
            on_complete: None,
            caret: 0,
            slot_rects: Vec::new(),
            prepared: Vec::new(),
            prepared_scale_bits: None,
            prepared_theme_revision: None,
        }
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = max_length.max(1);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn pattern(mut self, pattern: InputOtpPattern) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn on_complete(mut self, command: impl Into<CommandId>) -> Self {
        self.on_complete = Some(command.into());
        self
    }

    pub fn group(mut self, group: InputOTPGroup) -> Self {
        self.parts.push(InputOtpPart::Group(group));
        self
    }

    pub fn separator(mut self) -> Self {
        self.parts
            .push(InputOtpPart::Separator(InputOTPSeparator::new()));
        self
    }

    fn effective_parts(&self) -> Vec<InputOtpPart> {
        if self.parts.is_empty() {
            let mut g = InputOTPGroup::new();
            for i in 0..self.max_length {
                g = g.slot(i);
            }
            return vec![InputOtpPart::Group(g)];
        }

        self.parts.clone()
    }

    fn read_value<H: UiHost>(&self, app: &H) -> String {
        app.models().get(self.model).cloned().unwrap_or_default()
    }

    fn write_value<H: UiHost>(&self, app: &mut H, next: String) {
        let _ = app.models_mut().update(self.model, |v| *v = next);
    }

    fn slot_index_at(&self, pos: Point) -> Option<usize> {
        self.slot_rects
            .iter()
            .enumerate()
            .find(|(_, r)| r.contains(pos))
            .map(|(i, _)| i)
    }

    fn ensure_prepared_capacity(&mut self) {
        if self.prepared.len() != self.max_length {
            self.prepared.clear();
            self.prepared.resize_with(self.max_length, || None);
        }
        if self.slot_rects.len() != self.max_length {
            self.slot_rects.clear();
            self.slot_rects.resize(self.max_length, Rect::default());
        }
    }

    fn clear_prepared(&mut self, services: &mut dyn fret_core::UiServices) {
        for slot in self.prepared.iter_mut() {
            if let Some(g) = slot.take() {
                services.text().release(g.blob);
            }
        }
        self.prepared_scale_bits = None;
        self.prepared_theme_revision = None;
    }
}

impl<H: UiHost> Widget<H> for InputOTP {
    fn is_focusable(&self) -> bool {
        !self.disabled
    }

    fn is_text_input(&self) -> bool {
        !self.disabled
    }

    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        // shadcn/Tailwind disabled semantics map well to `pointer-events: none`.
        !self.disabled
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.clear_prepared(services);
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(fret_core::SemanticsRole::TextField);
        cx.set_disabled(self.disabled);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(self.model, Invalidation::Layout);

        self.ensure_prepared_capacity();

        let theme = cx.theme();
        let slot = slot_size(theme);
        let gap = gap(theme);
        let sep_w = separator_w(theme);

        let parts = self.effective_parts();
        let mut w = 0.0f32;
        let mut first = true;
        for part in parts {
            if !first {
                w += gap.0;
            }
            first = false;
            match part {
                InputOtpPart::Group(g) => {
                    w += slot.0 * g.slots.len() as f32;
                }
                InputOtpPart::Separator(_) => {
                    w += sep_w.0;
                }
            }
        }

        let width = Px(w.max(0.0).min(cx.available.width.0.max(0.0)));
        let height = Px(slot.0.max(0.0).min(cx.available.height.0.max(0.0)));
        Size::new(width, height)
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(_window) = cx.window else {
            return;
        };
        let focused = cx.focus == Some(cx.node);

        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { .. } => {
                    if !self.disabled {
                        cx.set_cursor_icon(CursorIcon::Text);
                    }
                }
                fret_core::PointerEvent::Down {
                    button, position, ..
                } => {
                    if *button != MouseButton::Left || self.disabled {
                        return;
                    }

                    cx.request_focus(cx.node);
                    if let Some(slot) = self.slot_index_at(*position) {
                        self.caret = slot.min(self.max_length);
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                _ => {}
            },
            Event::TextInput(text) => {
                if !focused || self.disabled {
                    return;
                }
                let value = self.read_value(cx.app);
                let (next, caret) =
                    apply_text_input(&value, self.caret, self.max_length, self.pattern, text);
                if next != value {
                    self.write_value(cx.app, next.clone());
                    self.caret = caret;
                    cx.invalidate_self(Invalidation::Layout);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();

                    if next.chars().count() >= self.max_length {
                        if let Some(cmd) = self.on_complete.clone() {
                            cx.dispatch_command(cmd);
                        }
                    }
                }
            }
            Event::KeyDown { key, .. } => {
                if !focused || self.disabled {
                    return;
                }

                match key {
                    KeyCode::Backspace => {
                        let value = self.read_value(cx.app);
                        let (next, caret) = apply_backspace(&value, self.caret);
                        if next != value {
                            self.write_value(cx.app, next);
                        }
                        self.caret = caret;
                        cx.invalidate_self(Invalidation::Layout);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::Delete => {
                        let value = self.read_value(cx.app);
                        let (next, caret) = apply_delete(&value, self.caret);
                        if next != value {
                            self.write_value(cx.app, next);
                        }
                        self.caret = caret;
                        cx.invalidate_self(Invalidation::Layout);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::ArrowLeft => {
                        self.caret = self.caret.saturating_sub(1);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::ArrowRight => {
                        self.caret = (self.caret + 1).min(self.max_length);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::Home => {
                        self.caret = 0;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::End => {
                        let value = self.read_value(cx.app);
                        self.caret = value.chars().count().min(self.max_length);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(self.model, Invalidation::Paint);
        self.ensure_prepared_capacity();

        let focused = cx.focus == Some(cx.node);
        let focus_visible = focused && fret_ui::focus_visible::is_focus_visible(cx.app, cx.window);

        let (
            slot,
            gap,
            sep_w,
            r,
            bw,
            theme_revision,
            text_muted,
            bg,
            border_color,
            fg,
            caret_color,
            ring_border,
            style,
            ring,
        ) = {
            let theme = cx.theme();
            (
                slot_size(theme),
                gap(theme),
                separator_w(theme),
                radius(theme),
                border_w(theme),
                theme.revision(),
                theme.colors.text_muted,
                bg(theme),
                border_color(theme),
                fg(theme),
                caret_color(theme),
                ring_border(theme),
                glyph_style(theme),
                fret_components_ui::declarative::style::focus_ring(theme, radius(theme)),
            )
        };

        let value = self.read_value(cx.app);
        let chars = value_to_chars(&value);
        let len = chars.len().min(self.max_length);
        if self.caret > self.max_length {
            self.caret = self.max_length;
        }

        let scale_bits = cx.scale_factor.to_bits();
        if self.prepared_scale_bits != Some(scale_bits)
            || self.prepared_theme_revision != Some(theme_revision)
        {
            self.clear_prepared(cx.services);
            self.prepared_scale_bits = Some(scale_bits);
            self.prepared_theme_revision = Some(theme_revision);
        }

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        // Compute layout rects (slot + separators).
        let parts = self.effective_parts();
        let mut x = cx.bounds.origin.x;
        let y = cx.bounds.origin.y;
        let mut first_part = true;

        // Reset slot rects to defaults (only slots we paint will be hit-testable).
        for r in self.slot_rects.iter_mut() {
            *r = Rect::default();
        }

        for part in parts {
            if !first_part {
                x = Px(x.0 + gap.0);
            }
            first_part = false;

            match part {
                InputOtpPart::Separator(_) => {
                    let rect = Rect::new(Point::new(x, y), Size::new(sep_w, slot));
                    let glyph = "−";
                    let (blob, metrics) = cx.services.text().prepare(glyph, style, constraints);
                    let gx = rect.origin.x.0 + (rect.size.width.0 - metrics.size.width.0) * 0.5;
                    let top = rect.origin.y.0 + (rect.size.height.0 - metrics.size.height.0) * 0.5;
                    let gy = top + metrics.baseline.0;
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(10),
                        origin: Point::new(Px(gx), Px(gy)),
                        text: blob,
                        color: text_muted,
                    });
                    cx.services.text().release(blob);
                    x = Px(x.0 + sep_w.0);
                }
                InputOtpPart::Group(group) => {
                    for (slot_pos, slot_spec) in group.slots.iter().enumerate() {
                        let i = slot_spec.index;
                        if i >= self.max_length {
                            continue;
                        }

                        let rect = Rect::new(Point::new(x, y), Size::new(slot, slot));
                        self.slot_rects[i] = rect;

                        let mut border_edges = Edges::all(bw);
                        // Avoid double borders in a group: only the first slot draws the left edge.
                        if slot_pos != 0 {
                            border_edges.left = Px(0.0);
                        }

                        let mut corner = Corners::all(Px(0.0));
                        if slot_pos == 0 {
                            corner.top_left = r;
                            corner.bottom_left = r;
                        }
                        if slot_pos + 1 == group.slots.len() {
                            corner.top_right = r;
                            corner.bottom_right = r;
                        }

                        let is_active = focused
                            && (i == self.caret.min(self.max_length.saturating_sub(1))
                                || (self.caret == self.max_length && i + 1 == self.max_length));

                        let border_color = if is_active { ring_border } else { border_color };

                        cx.scene.push(SceneOp::Quad {
                            order: DrawOrder(0),
                            rect,
                            background: bg,
                            border: border_edges,
                            border_color,
                            corner_radii: corner,
                        });

                        if is_active && focus_visible {
                            fret_ui::paint::paint_focus_ring(cx.scene, DrawOrder(1), rect, ring);
                        }

                        // Slot character.
                        if i < len {
                            let ch = chars[i];
                            let needs_prepare =
                                self.prepared[i].as_ref().map_or(true, |p| p.ch != ch);
                            if needs_prepare {
                                if let Some(prev) = self.prepared[i].take() {
                                    cx.services.text().release(prev.blob);
                                }
                                let s = ch.to_string();
                                let (blob, metrics) =
                                    cx.services.text().prepare(&s, style, constraints);
                                self.prepared[i] = Some(PreparedGlyph { blob, metrics, ch });
                            }
                            if let Some(p) = self.prepared[i].as_ref() {
                                let gx = rect.origin.x.0
                                    + (rect.size.width.0 - p.metrics.size.width.0) * 0.5;
                                let top = rect.origin.y.0
                                    + (rect.size.height.0 - p.metrics.size.height.0) * 0.5;
                                let gy = top + p.metrics.baseline.0;
                                cx.scene.push(SceneOp::Text {
                                    order: DrawOrder(2),
                                    origin: Point::new(Px(gx), Px(gy)),
                                    text: p.blob,
                                    color: fg,
                                });
                            }
                        } else if is_active && focused {
                            // Fake caret (non-animated): draw a thin line when the active slot is empty.
                            let caret_w = Px(1.0);
                            let caret_h = Px((slot.0 * 0.45).max(0.0));
                            let caret_x = Px(rect.origin.x.0 + rect.size.width.0 * 0.5 - 0.5);
                            let caret_y =
                                Px(rect.origin.y.0 + (rect.size.height.0 - caret_h.0) * 0.5);
                            cx.scene.push(SceneOp::Quad {
                                order: DrawOrder(3),
                                rect: Rect::new(
                                    Point::new(caret_x, caret_y),
                                    Size::new(caret_w, caret_h),
                                ),
                                background: caret_color,
                                border: Edges::all(Px(0.0)),
                                border_color: Color::TRANSPARENT,
                                corner_radii: Corners::all(Px(0.0)),
                            });
                        }

                        x = Px(x.0 + slot.0);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_input_filters_and_fills_sequentially() {
        let (v, caret) = apply_text_input("", 0, 6, InputOtpPattern::Digits, "12a3");
        assert_eq!(v, "123");
        assert_eq!(caret, 3);
    }

    #[test]
    fn backspace_removes_previous_char() {
        let (v, caret) = apply_backspace("123", 3);
        assert_eq!(v, "12");
        assert_eq!(caret, 2);
    }

    #[test]
    fn delete_removes_at_caret() {
        let (v, caret) = apply_delete("123", 1);
        assert_eq!(v, "13");
        assert_eq!(caret, 1);
    }
}
