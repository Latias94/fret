use fret_app::Effect;
use fret_core::{
    CaretAffinity, Color, Corners, DrawOrder, Edges, Event, MouseButton, Px, Rect, SceneOp, Size,
    TextConstraints, TextStyle, TextWrap,
};
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};

#[derive(Debug)]
pub struct MultilineProbe {
    text: String,
    style: TextStyle,
    blob: Option<fret_core::TextBlobId>,
    metrics: Option<fret_core::TextMetrics>,
    caret: usize,
    selection_anchor: usize,
    affinity: CaretAffinity,
    selection_rects: Vec<Rect>,
    last_bounds: Rect,
    last_sent_cursor: Option<Rect>,
}

impl Default for MultilineProbe {
    fn default() -> Self {
        Self {
            text: "Multiline probe: click/drag to place caret and select.\n\
This is wrapped text (TextWrap::Word) and exercises:\n\
- TextService::hit_test_point\n\
- TextService::caret_rect\n\
- TextService::selection_rects\n\
\n\
目标：为后续 Console/Inspector/代码编辑器打基础。"
                .to_string(),
            style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
            },
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

impl MultilineProbe {
    pub fn new() -> Self {
        Self::default()
    }

    fn padding() -> Px {
        Px(10.0)
    }

    fn inner_bounds(&self) -> Rect {
        let p = Self::padding();
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

impl Widget for MultilineProbe {
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
            Event::Ime(ime) => {
                tracing::debug!(?ime, "multiline probe ime");
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

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        self.last_bounds = cx.bounds;

        let constraints = TextConstraints {
            max_width: Some(self.inner_bounds().size.width),
            wrap: TextWrap::Word,
            scale_factor: cx.scale_factor,
        };
        let (blob, metrics) = cx.text.prepare(&self.text, self.style, constraints);
        self.blob = Some(blob);
        self.metrics = Some(metrics);

        Size::new(cx.available.width, Px(220.0_f32.min(cx.available.height.0)))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.last_bounds = cx.bounds;

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
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
                background: Color {
                    r: 0.24,
                    g: 0.34,
                    b: 0.52,
                    a: 0.65,
                },
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
            color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 1.0,
            },
        });

        let caret = cx.text.caret_rect(blob, self.caret, self.affinity);
        let hairline = Px((1.0 / cx.scale_factor.max(1.0)).max(1.0 / 8.0));
        let caret_rect = Rect::new(
            fret_core::Point::new(
                inner.origin.x + caret.origin.x,
                inner.origin.y + caret.origin.y,
            ),
            Size::new(Px(hairline.0.max(1.0)), caret.size.height),
        );

        if cx.focus == Some(cx.node) {
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
        } else {
            self.last_sent_cursor = None;
        }

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: caret_rect,
            background: Color {
                r: 0.90,
                g: 0.90,
                b: 0.92,
                a: 1.0,
            },
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });
    }
}
