use fret_core::{
    Color, DrawOrder, Event, FontId, ImeEvent, MouseButton, Px, Rect, SceneOp, Size, TextConstraints,
    TextMetrics, TextStyle, TextWrap,
};

use crate::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};

#[derive(Debug, Clone)]
pub struct Text {
    text: String,
    style: TextStyle,
    color: Color,
    blob: Option<fret_core::TextBlobId>,
    metrics: Option<TextMetrics>,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextStyle {
                font: FontId::default(),
                size: Px(13.0),
            },
            color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 1.0,
            },
            blob: None,
            metrics: None,
        }
    }

    pub fn with_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Widget for Text {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        let constraints = TextConstraints {
            max_width: Some(cx.available.width),
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };
        let (blob, metrics) = cx.text.prepare(&self.text, self.style, constraints);
        self.blob = Some(blob);
        self.metrics = Some(metrics);
        metrics.size
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let Some(blob) = self.blob else {
            return;
        };
        let Some(metrics) = self.metrics else {
            return;
        };

        let origin = fret_core::geometry::Point::new(cx.bounds.origin.x, cx.bounds.origin.y + metrics.baseline);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(0),
            origin,
            text: blob,
            color: self.color,
        });
    }
}

#[derive(Debug)]
pub struct TextInput {
    text: String,
    preedit: String,
    style: TextStyle,
    base_blob: Option<fret_core::TextBlobId>,
    base_metrics: Option<TextMetrics>,
    preedit_blob: Option<fret_core::TextBlobId>,
    preedit_metrics: Option<TextMetrics>,
    last_bounds: Rect,
    last_sent_cursor: Option<Rect>,
    last_text_input_tick: Option<fret_core::TickId>,
    last_text_input_text: Option<String>,
    last_ime_commit_tick: Option<fret_core::TickId>,
    last_ime_commit_text: Option<String>,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            preedit: String::new(),
            style: TextStyle {
                font: FontId::default(),
                size: Px(13.0),
            },
            base_blob: None,
            base_metrics: None,
            preedit_blob: None,
            preedit_metrics: None,
            last_bounds: Rect::default(),
            last_sent_cursor: None,
            last_text_input_tick: None,
            last_text_input_text: None,
            last_ime_commit_tick: None,
            last_ime_commit_text: None,
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
}

impl TextInput {
    fn is_focused(&self, cx: &EventCx<'_>) -> bool {
        cx.focus == Some(cx.node)
    }

    fn caret_rect(&self, bounds: Rect, scale_factor: f32) -> Rect {
        let base_w = self
            .base_metrics
            .map(|m| m.size.width)
            .unwrap_or(Px(0.0));
        let preedit_w = if self.preedit.is_empty() {
            Px(0.0)
        } else {
            self.preedit_metrics
                .map(|m| m.size.width)
                .unwrap_or(Px(0.0))
        };

        let padding = Px(8.0);
        let x = bounds.origin.x + padding + base_w + preedit_w;
        let h = self
            .base_metrics
            .map(|m| m.size.height)
            .unwrap_or(Px(16.0));
        let hairline = Px((1.0 / scale_factor.max(1.0)).max(1.0 / 8.0));
        Rect::new(
            fret_core::geometry::Point::new(x, bounds.origin.y + Px(6.0)),
            Size::new(Px(hairline.0.max(1.0)), Px(h.0.max(16.0))),
        )
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextInput {
    fn is_text_input(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        let focused = self.is_focused(cx);
        let Some(window) = cx.window else {
            return;
        };

        match event {
            Event::Pointer(fret_core::PointerEvent::Down { button, .. }) => {
                if *button != MouseButton::Left {
                    return;
                }
                cx.request_focus(cx.node);
                cx.app.push_effect(fret_app::Effect::ImeAllow {
                    window,
                    enabled: true,
                });
                self.last_sent_cursor = None;
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
            }
            Event::KeyDown { key, .. } => {
                if !focused {
                    return;
                }
                if *key == fret_core::KeyCode::Backspace && self.preedit.is_empty() {
                    let _ = self.text.pop();
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
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

                if self.preedit.is_empty() {
                    self.text.push_str(text);
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                }
            }
            Event::Ime(ime) => {
                if !focused {
                    return;
                }
                match ime {
                    ImeEvent::Enabled => {}
                    ImeEvent::Disabled => {
                        self.preedit.clear();
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

                        self.text.push_str(text);
                        self.preedit.clear();
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                    ImeEvent::Preedit { text, .. } => {
                        self.preedit = text.clone();
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }

    fn command(&mut self, cx: &mut crate::CommandCx<'_>, command: &fret_app::CommandId) -> bool {
        if cx.focus != Some(cx.node) {
            return false;
        }

        match command.as_str() {
            "text.clear" => {
                self.text.clear();
                self.preedit.clear();
                cx.invalidate_self(Invalidation::Layout);
                cx.request_redraw();
                true
            }
            _ => false,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        self.last_bounds = cx.bounds;

        let base_constraints = TextConstraints {
            max_width: Some(cx.available.width),
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };
        let (base_blob, base_metrics) = cx.text.prepare(&self.text, self.style, base_constraints);
        self.base_blob = Some(base_blob);
        self.base_metrics = Some(base_metrics);

        if self.preedit.is_empty() {
            self.preedit_blob = None;
            self.preedit_metrics = None;
        } else {
            let (pre_blob, pre_metrics) =
                cx.text
                    .prepare(&self.preedit, self.style, base_constraints);
            self.preedit_blob = Some(pre_blob);
            self.preedit_metrics = Some(pre_metrics);
        }

        let h = Px(34.0_f32.max(base_metrics.size.height.0 + 12.0));
        Size::new(cx.available.width, h)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let Some(window) = cx.window else {
            return;
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: Color {
                r: 0.10,
                g: 0.11,
                b: 0.14,
                a: 1.0,
            },
            border: fret_core::geometry::Edges::all(Px(1.0)),
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.35,
            },
            corner_radii: fret_core::geometry::Corners::all(Px(6.0)),
        });

        let padding = Px(8.0);
        let base_origin = if let Some(metrics) = self.base_metrics {
            fret_core::geometry::Point::new(
                cx.bounds.origin.x + padding,
                cx.bounds.origin.y + Px(6.0) + metrics.baseline,
            )
        } else {
            fret_core::geometry::Point::new(cx.bounds.origin.x + padding, cx.bounds.origin.y + Px(16.0))
        };

        if let Some(base_blob) = self.base_blob {
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(0),
                origin: base_origin,
                text: base_blob,
                color: Color {
                    r: 0.92,
                    g: 0.92,
                    b: 0.92,
                    a: 1.0,
                },
            });
        }

        if let (Some(pre_blob), Some(base_m)) = (self.preedit_blob, self.base_metrics) {
            let pre_origin = fret_core::geometry::Point::new(
                base_origin.x + base_m.size.width,
                base_origin.y,
            );
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(0),
                origin: pre_origin,
                text: pre_blob,
                color: Color {
                    r: 0.65,
                    g: 0.82,
                    b: 1.0,
                    a: 1.0,
                },
            });
        }

        let focused = cx.focus == Some(cx.node);
        if !focused {
            return;
        }

        cx.app.push_effect(fret_app::Effect::ImeAllow {
            window,
            enabled: true,
        });

        let caret = self.caret_rect(cx.bounds, cx.scale_factor);
        if self.last_sent_cursor.map_or(true, |r| r != caret) {
            self.last_sent_cursor = Some(caret);
            cx.app.push_effect(fret_app::Effect::ImeSetCursorArea {
                window,
                rect: caret,
            });
        }

        // Draw caret as a thin quad (always visible in MVP).
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: caret,
            background: Color {
                r: 0.95,
                g: 0.95,
                b: 0.95,
                a: 1.0,
            },
            border: fret_core::geometry::Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::geometry::Corners::all(Px(1.0)),
        });
    }
}
