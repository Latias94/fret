use fret_core::{Color, Corners, DrawOrder, Edges, Event, MouseButton, Px, Rect, SceneOp, Size};
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Widget, elements};

#[derive(Default)]
struct RootState {
    order: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
struct ItemState {
    color: Color,
}

pub struct ElementsMvp2Demo;

impl ElementsMvp2Demo {
    pub fn new() -> Self {
        Self
    }

    fn ensure_root_state(state: &mut RootState) {
        if state.order.is_empty() {
            state.order = vec![1, 2, 3, 4, 5, 6];
        }
    }

    fn item_color(seed: u32) -> Color {
        let a = (seed as f32 * 0.6180339).fract();
        let b = (seed as f32 * 0.3819660).fract();
        Color {
            r: 0.25 + a * 0.55,
            g: 0.25 + b * 0.55,
            b: 0.30 + (1.0 - a) * 0.45,
            a: 1.0,
        }
    }
}

impl Default for ElementsMvp2Demo {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ElementsMvp2Demo {
    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };

        let Event::Pointer(pe) = event else {
            return;
        };

        if let fret_core::PointerEvent::Down { button, .. } = pe {
            if *button != MouseButton::Left {
                return;
            }

            let app = &mut *cx.app;
            elements::with_element_cx(app, window, Rect::default(), "mvp2-demo", |ecx| {
                ecx.with_state(RootState::default, |root| {
                    Self::ensure_root_state(root);
                    root.order.reverse();
                });
            });

            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.request_focus(cx.node);
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        Size::new(cx.available.width, Px(220.0))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let Some(window) = cx.window else {
            return;
        };

        let bounds = cx.bounds;
        let app = &mut *cx.app;
        let scene = &mut *cx.scene;

        elements::with_element_cx(app, window, bounds, "mvp2-demo", |ecx| {
            let items = ecx.with_state(RootState::default, |root| {
                Self::ensure_root_state(root);
                root.order.clone()
            });

            let header_h = Px(26.0);
            let header_bg = Color {
                r: 0.10,
                g: 0.10,
                b: 0.12,
                a: 1.0,
            };

            scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: Rect::new(bounds.origin, Size::new(bounds.size.width, header_h)),
                background: header_bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(8.0)),
            });

            // Keyed row: item color is stored per item id (stable across reorder).
            let y0 = header_h + Px(12.0);
            let pad = Px(10.0);
            let item_w = Px(44.0);
            let item_h = Px(28.0);
            let gap = Px(8.0);

            scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: Rect::new(
                    fret_core::Point::new(bounds.origin.x, bounds.origin.y + y0),
                    Size::new(bounds.size.width, item_h + Px(18.0)),
                ),
                background: Color {
                    r: 0.12,
                    g: 0.14,
                    b: 0.18,
                    a: 1.0,
                },
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(8.0)),
            });

            ecx.for_each_keyed(
                &items,
                |k| *k,
                |ecx, index, key| {
                    let color = ecx.with_state(
                        || ItemState {
                            color: Self::item_color(*key),
                        },
                        |s| s.color,
                    );

                    let x = bounds.origin.x + pad + (item_w + gap) * index as f32;
                    let rect = Rect::new(
                        fret_core::Point::new(x, bounds.origin.y + y0 + Px(9.0)),
                        Size::new(item_w, item_h),
                    );
                    scene.push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect,
                        background: color,
                        border: Edges::all(Px(1.0)),
                        border_color: Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.35,
                        },
                        corner_radii: Corners::all(Px(6.0)),
                    });
                },
            );

            // Unkeyed row: item color is stored per position (will "move" on reorder),
            // and we emit a debug warning when the order changes without keys.
            let y1 = y0 + Px(66.0);
            scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: Rect::new(
                    fret_core::Point::new(bounds.origin.x, bounds.origin.y + y1),
                    Size::new(bounds.size.width, item_h + Px(18.0)),
                ),
                background: Color {
                    r: 0.14,
                    g: 0.12,
                    b: 0.16,
                    a: 1.0,
                },
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(8.0)),
            });

            ecx.for_each_unkeyed(&items, |ecx, index, key| {
                let color = ecx.with_state(
                    || ItemState {
                        color: Self::item_color(*key + 1000),
                    },
                    |s| s.color,
                );
                let x = bounds.origin.x + pad + (item_w + gap) * index as f32;
                let rect = Rect::new(
                    fret_core::Point::new(x, bounds.origin.y + y1 + Px(9.0)),
                    Size::new(item_w, item_h),
                );
                scene.push(SceneOp::Quad {
                    order: DrawOrder(0),
                    rect,
                    background: color,
                    border: Edges::all(Px(1.0)),
                    border_color: Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.35,
                    },
                    corner_radii: Corners::all(Px(6.0)),
                });
            });
        });
    }
}
