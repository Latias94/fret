use fret_app::{CommandId, InputContext, KeymapService, Platform, format_chord};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect, SceneOp,
    Size, TextConstraints, TextMetrics, TextStyle, TextWrap,
    ids::FontId,
};
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};

#[derive(Debug)]
pub struct OverlayBackdrop {
    color: Color,
    close_command: CommandId,
}

impl OverlayBackdrop {
    pub fn new(color: Color, close_command: CommandId) -> Self {
        Self {
            color,
            close_command,
        }
    }
}

impl Widget for OverlayBackdrop {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        cx.available
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        if let Event::Pointer(pe) = event {
            if let fret_core::PointerEvent::Down { button, .. } = pe {
                if *button == MouseButton::Left {
                    cx.dispatch_command(self.close_command.clone());
                    cx.stop_propagation();
                }
            }
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });
    }
}

#[derive(Debug)]
pub struct OverlayAnchor {
    pub width: Px,
    pub height: Px,
    pub top: Px,
}

impl OverlayAnchor {
    pub fn new(width: Px, height: Px) -> Self {
        Self {
            width,
            height,
            top: Px(72.0),
        }
    }

    pub fn with_top(mut self, top: Px) -> Self {
        self.top = top;
        self
    }
}

impl Widget for OverlayAnchor {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        let mut child_bounds = cx.bounds;

        let w = self.width.0.min(cx.available.width.0).max(0.0);
        let h = self
            .height
            .0
            .min((cx.available.height.0 - self.top.0).max(0.0));

        child_bounds.origin.x = Px(cx.bounds.origin.x.0 + (cx.available.width.0 - w) * 0.5);
        child_bounds.origin.y = Px(cx.bounds.origin.y.0 + self.top.0);
        child_bounds.size = Size::new(Px(w), Px(h));

        for &child in cx.children {
            let _ = cx.layout_in(child, child_bounds);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct PaletteItem {
    command: CommandId,
    title_blob: fret_core::TextBlobId,
    title_metrics: TextMetrics,
    shortcut_blob: Option<fret_core::TextBlobId>,
    shortcut_metrics: Option<TextMetrics>,
}

#[derive(Debug)]
pub struct CommandPalette {
    query: String,
    selected: usize,
    items: Vec<PaletteItem>,
    query_blob: Option<fret_core::TextBlobId>,
    query_metrics: Option<TextMetrics>,
    prompt_blob: Option<fret_core::TextBlobId>,
    prompt_metrics: Option<TextMetrics>,
    last_bounds: Rect,
    pressed_index: Option<usize>,
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            selected: 0,
            items: Vec::new(),
            query_blob: None,
            query_metrics: None,
            prompt_blob: None,
            prompt_metrics: None,
            last_bounds: Rect::default(),
            pressed_index: None,
        }
    }

    fn normal_ctx() -> InputContext {
        InputContext {
            platform: Platform::current(),
            ui_has_modal: false,
            focus_is_text_input: false,
        }
    }

    fn text_style() -> TextStyle {
        TextStyle {
            font: FontId::default(),
            size: Px(13.0),
        }
    }

    fn rebuild_items(&mut self, cx: &mut LayoutCx<'_>) {
        for item in self.items.drain(..) {
            cx.text.release(item.title_blob);
            if let Some(blob) = item.shortcut_blob {
                cx.text.release(blob);
            }
        }

        let style = Self::text_style();
        let ctx = Self::normal_ctx();

        let keymap = cx.app.global::<KeymapService>();

        let mut entries: Vec<(CommandId, fret_app::CommandMeta)> = cx
            .app
            .commands()
            .iter()
            .filter_map(|(id, meta)| {
                if meta.hidden {
                    return None;
                }
                if let Some(when) = meta.when.as_ref() {
                    if !when.eval(&ctx) {
                        return None;
                    }
                }
                Some((id.clone(), meta.clone()))
            })
            .collect();

        entries.sort_by(|a, b| a.1.title.as_ref().cmp(b.1.title.as_ref()));

        let q = self.query.trim().to_ascii_lowercase();

        let max_width = Px((cx.bounds.size.width.0 - 24.0).max(0.0));
        let constraints = TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };

        for (command, meta) in entries {
            if !q.is_empty() {
                let title = meta.title.to_ascii_lowercase();
                let keyword_match = meta
                    .keywords
                    .iter()
                    .any(|k| k.as_ref().to_ascii_lowercase().contains(&q));
                if !title.contains(&q) && !keyword_match && !command.as_str().contains(&q) {
                    continue;
                }
            }

            let shortcut = keymap.and_then(|service| {
                service
                    .keymap
                    .shortcut_for_command(&ctx, &command)
                    .map(|chord| format_chord(ctx.platform, chord))
            });

            let (title_blob, title_metrics) = cx.text.prepare(meta.title.as_ref(), style, constraints);

            let (shortcut_blob, shortcut_metrics) = if let Some(s) = shortcut.as_deref() {
                let (b, m) = cx.text.prepare(s, style, constraints);
                (Some(b), Some(m))
            } else {
                (None, None)
            };

            self.items.push(PaletteItem {
                command,
                title_blob,
                title_metrics,
                shortcut_blob,
                shortcut_metrics,
            });
        }

        if self.selected >= self.items.len() {
            self.selected = self.items.len().saturating_sub(1);
        }
    }

    fn selected_command(&self) -> Option<CommandId> {
        self.items.get(self.selected).map(|i| i.command.clone())
    }

    fn item_index_at_point(&self, point: Point) -> Option<usize> {
        if !self.last_bounds.contains(point) {
            return None;
        }
        let x0 = self.last_bounds.origin.x.0;
        let y0 = self.last_bounds.origin.y.0;
        let x1 = x0 + self.last_bounds.size.width.0;
        let y1 = y0 + self.last_bounds.size.height.0;
        if point.x.0 < x0 || point.x.0 >= x1 || point.y.0 < y0 || point.y.0 >= y1 {
            return None;
        }

        let header_h = 34.0;
        let list_top = y0 + header_h;
        if point.y.0 < list_top {
            return None;
        }
        let row_h = 22.0;
        let idx = ((point.y.0 - list_top) / row_h).floor() as usize;
        Some(idx)
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for CommandPalette {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        self.last_bounds = cx.bounds;

        let style = Self::text_style();
        let constraints = TextConstraints {
            max_width: Some(Px((cx.bounds.size.width.0 - 24.0).max(0.0))),
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };

        if let Some(blob) = self.prompt_blob.take() {
            cx.text.release(blob);
        }
        let (prompt_blob, prompt_metrics) = cx.text.prepare(">", style, constraints);
        self.prompt_blob = Some(prompt_blob);
        self.prompt_metrics = Some(prompt_metrics);

        if let Some(blob) = self.query_blob.take() {
            cx.text.release(blob);
        }
        let (query_blob, query_metrics) = cx.text.prepare(&self.query, style, constraints);
        self.query_blob = Some(query_blob);
        self.query_metrics = Some(query_metrics);

        self.rebuild_items(cx);
        cx.available
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        match event {
            Event::TextInput(text) => {
                // Some platforms may report control characters (e.g. backspace) via text input.
                // For the palette query, only accept printable characters.
                let mut did_change = false;
                for ch in text.chars() {
                    if ch.is_control() {
                        continue;
                    }
                    self.query.push(ch);
                    did_change = true;
                }

                if did_change {
                    self.selected = 0;
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
                cx.stop_propagation();
            }
            Event::KeyDown { key, modifiers: _, repeat } => {
                match key {
                    KeyCode::Escape => {
                        if *repeat {
                            return;
                        }
                        cx.dispatch_command(CommandId::from("command_palette.close"));
                        cx.stop_propagation();
                    }
                    KeyCode::Enter => {
                        if *repeat {
                            return;
                        }
                        if let Some(command) = self.selected_command() {
                            cx.dispatch_command(CommandId::from("command_palette.close"));
                            cx.dispatch_command(command);
                            cx.stop_propagation();
                        }
                    }
                    KeyCode::ArrowUp => {
                        self.selected = self.selected.saturating_sub(1);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::ArrowDown => {
                        if !self.items.is_empty() {
                            self.selected = (self.selected + 1).min(self.items.len() - 1);
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                    }
                    KeyCode::Backspace => {
                        let _ = self.query.pop();
                        self.selected = 0;
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Down { button, position, .. } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    let Some(idx) = self.item_index_at_point(*position) else {
                        return;
                    };
                    self.pressed_index = Some(idx);
                    if idx < self.items.len() {
                        self.selected = idx;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                    cx.stop_propagation();
                }
                fret_core::PointerEvent::Up { button, position, .. } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    let pressed = self.pressed_index.take();
                    let Some(idx) = self.item_index_at_point(*position) else {
                        return;
                    };
                    if pressed == Some(idx) && idx < self.items.len() {
                        if let Some(command) = self.items.get(idx).map(|i| i.command.clone()) {
                            cx.dispatch_command(CommandId::from("command_palette.close"));
                            cx.dispatch_command(command);
                        }
                    }
                    cx.stop_propagation();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let panel_bg = Color {
            r: 0.10,
            g: 0.10,
            b: 0.12,
            a: 0.98,
        };
        let border = Color {
            r: 0.22,
            g: 0.22,
            b: 0.25,
            a: 1.0,
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: panel_bg,
            border: Edges::all(Px(1.0)),
            border_color: border,
            corner_radii: Corners::all(Px(10.0)),
        });

        let pad_x = 12.0;
        let header_y = cx.bounds.origin.y.0 + 10.0;

        if let (Some(prompt), Some(prompt_metrics)) = (self.prompt_blob, self.prompt_metrics) {
            let origin = Point::new(
                Px(cx.bounds.origin.x.0 + pad_x),
                Px(header_y + prompt_metrics.baseline.0),
            );
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(1),
                origin,
                text: prompt,
                color: Color {
                    r: 0.78,
                    g: 0.78,
                    b: 0.82,
                    a: 1.0,
                },
            });
        }

        if let (Some(blob), Some(metrics)) = (self.query_blob, self.query_metrics) {
            let origin = Point::new(
                Px(cx.bounds.origin.x.0 + pad_x + 14.0),
                Px(header_y + metrics.baseline.0),
            );
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(1),
                origin,
                text: blob,
                color: Color {
                    r: 0.90,
                    g: 0.90,
                    b: 0.92,
                    a: 1.0,
                },
            });
        }

        let list_top = cx.bounds.origin.y.0 + 34.0;
        let row_h = 22.0;
        let max_rows = ((cx.bounds.size.height.0 - 42.0) / row_h).floor().max(0.0) as usize;
        let visible = max_rows.min(self.items.len());

        for i in 0..visible {
            let y = list_top + i as f32 * row_h;
            if i == self.selected {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(Px(cx.bounds.origin.x.0 + 6.0), Px(y + 2.0)),
                        Size::new(Px(cx.bounds.size.width.0 - 12.0), Px(row_h - 4.0)),
                    ),
                    background: Color {
                        r: 0.18,
                        g: 0.20,
                        b: 0.26,
                        a: 1.0,
                    },
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(6.0)),
                });
            }

            let item = &self.items[i];
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(2),
                origin: Point::new(
                    Px(cx.bounds.origin.x.0 + pad_x),
                    Px(y + 2.0 + item.title_metrics.baseline.0),
                ),
                text: item.title_blob,
                color: Color {
                    r: 0.92,
                    g: 0.92,
                    b: 0.94,
                    a: 1.0,
                },
            });

            if let (Some(blob), Some(metrics)) = (item.shortcut_blob, item.shortcut_metrics) {
                let x = cx.bounds.origin.x.0 + cx.bounds.size.width.0 - pad_x - metrics.size.width.0;
                let origin = Point::new(Px(x), Px(y + metrics.baseline.0 + 2.0));
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(2),
                    origin,
                    text: blob,
                    color: Color {
                        r: 0.70,
                        g: 0.70,
                        b: 0.74,
                        a: 1.0,
                    },
                });
            }
        }
    }
}
