use fret_app::Effect;
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, ImeEvent, MouseButton, Px, Rect, SceneOp, Size,
};
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};

#[derive(Debug, Default)]
pub struct ImeProbe {
    caret_x: Px,
    last_bounds: Rect,
    last_sent: Option<Rect>,
    preedit: Option<String>,
}

impl ImeProbe {
    pub fn new() -> Self {
        Self::default()
    }

    fn caret_rect(&self, bounds: Rect) -> Rect {
        let x = (bounds.origin.x.0 + self.caret_x.0).clamp(
            bounds.origin.x.0 + 8.0,
            bounds.origin.x.0 + bounds.size.width.0 - 8.0,
        );
        Rect::new(
            fret_core::Point::new(Px(x), bounds.origin.y + Px(18.0)),
            Size::new(Px(2.0), Px(18.0)),
        )
    }
}

impl Widget for ImeProbe {
    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
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

                // Enable IME for this window and place the cursor area near our caret.
                self.caret_x = Px(position.x.0 - self.last_bounds.origin.x.0);
                cx.app.push_effect(Effect::ImeAllow {
                    window,
                    enabled: true,
                });
                cx.app.push_effect(Effect::ImeSetCursorArea {
                    window,
                    rect: self.caret_rect(self.last_bounds),
                });

                cx.request_focus(cx.node);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Ime(ime) => {
                match ime {
                    ImeEvent::Enabled => {
                        self.preedit = None;
                    }
                    ImeEvent::Disabled => {
                        self.preedit = None;
                    }
                    ImeEvent::Commit(text) => {
                        tracing::info!(?text, "ime commit");
                        self.preedit = None;
                    }
                    ImeEvent::Preedit { text, cursor } => {
                        tracing::info!(?text, ?cursor, "ime preedit");
                        self.preedit = Some(text.clone());
                    }
                }
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::TextInput(text) => {
                tracing::info!(?text, "text input");
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        self.last_bounds = cx.bounds;
        Size::new(cx.available.width, Px(92.0))
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
            border: Edges::all(Px(1.0)),
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.35,
            },
            corner_radii: Corners::all(Px(8.0)),
        });

        let caret = self.caret_rect(cx.bounds);
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: caret,
            background: Color {
                r: 0.95,
                g: 0.95,
                b: 0.95,
                a: 1.0,
            },
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(1.0)),
        });

        // Keep the OS candidate window near the caret while the widget is visible,
        // but avoid spamming the platform API if the caret hasn't moved.
        if self.last_sent.map_or(true, |r| r != caret) {
            self.last_sent = Some(caret);
            cx.app.push_effect(Effect::ImeSetCursorArea {
                window,
                rect: caret,
            });
        }

        // Note: preedit is not rendered (text system not wired yet); we only log it to validate plumbing.
        let _ = &self.preedit;
    }
}
