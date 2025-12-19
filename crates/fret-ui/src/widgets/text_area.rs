use fret_app::Effect;
use fret_core::{
    CaretAffinity, Color, Corners, DrawOrder, Edges, Event, MouseButton, Px, Rect, SceneOp, Size,
    TextConstraints, TextMetrics, TextStyle, TextWrap,
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

    caret: usize,
    selection_anchor: usize,
    affinity: CaretAffinity,
    selection_rects: Vec<Rect>,
    last_bounds: Rect,
    last_sent_cursor: Option<Rect>,
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
            caret: 0,
            selection_anchor: 0,
            affinity: CaretAffinity::Downstream,
            selection_rects: Vec::new(),
            last_bounds: Rect::default(),
            last_sent_cursor: None,
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

    fn selection_range(&self) -> (usize, usize) {
        let a = self.selection_anchor.min(self.caret);
        let b = self.selection_anchor.max(self.caret);
        (a, b)
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
            Event::Pointer(fret_core::PointerEvent::Down {
                button, position, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                cx.request_focus(cx.node);
                cx.capture_pointer(cx.node);

                let inner = self.inner_bounds();
                let local =
                    fret_core::Point::new(position.x - inner.origin.x, position.y - inner.origin.y);
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

                let inner = self.inner_bounds();
                let local =
                    fret_core::Point::new(position.x - inner.origin.x, position.y - inner.origin.y);
                self.set_caret_from_point(cx, local);

                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Pointer(fret_core::PointerEvent::Up { button, .. }) => {
                if *button == MouseButton::Left && cx.captured == Some(cx.node) {
                    cx.release_pointer_capture();
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
            _ => false,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        self.last_bounds = cx.bounds;

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

        cx.text.selection_rects(
            blob,
            (self.selection_anchor, self.caret),
            &mut self.selection_rects,
        );
        for r in &self.selection_rects {
            let rect = Rect::new(
                fret_core::Point::new(inner.origin.x + r.origin.x, inner.origin.y + r.origin.y),
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

        let text_origin = fret_core::Point::new(inner.origin.x, inner.origin.y + metrics.baseline);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(0),
            origin: text_origin,
            text: blob,
            color: self.style.text_color,
        });

        if cx.focus == Some(cx.node) {
            let caret = cx.text.caret_rect(blob, self.caret, self.affinity);
            let hairline = Px((1.0 / cx.scale_factor.max(1.0)).max(1.0 / 8.0));
            let caret_rect = Rect::new(
                fret_core::Point::new(
                    inner.origin.x + caret.origin.x,
                    inner.origin.y + caret.origin.y,
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
    }
}
