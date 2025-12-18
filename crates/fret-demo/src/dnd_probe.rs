use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, ExternalDragKind, MouseButton, Px, SceneOp, Size,
};
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};

#[derive(Debug, Default)]
pub struct DndProbe {
    hovering: bool,
    last_path: Option<String>,
}

impl DndProbe {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget for DndProbe {
    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        match event {
            Event::ExternalDrag(drag) => {
                match &drag.kind {
                    ExternalDragKind::EnterFiles(files) | ExternalDragKind::OverFiles(files) => {
                        self.hovering = true;
                        self.last_path = files.last().map(|p| p.display().to_string());
                        tracing::info!(count = files.len(), "external drag hover");
                    }
                    ExternalDragKind::DropFiles(files) => {
                        self.hovering = false;
                        self.last_path = files.last().map(|p| p.display().to_string());
                        tracing::info!(count = files.len(), "external drag drop");
                    }
                    ExternalDragKind::Leave => {
                        self.hovering = false;
                        tracing::info!("external drag leave");
                    }
                }
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Pointer(fret_core::PointerEvent::Down { button, .. }) => {
                if *button == MouseButton::Right {
                    self.last_path = None;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        Size::new(cx.available.width, Px(96.0))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let bg = if self.hovering {
            Color {
                r: 0.10,
                g: 0.16,
                b: 0.10,
                a: 1.0,
            }
        } else {
            Color {
                r: 0.12,
                g: 0.12,
                b: 0.16,
                a: 1.0,
            }
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: bg,
            border: Edges::all(Px(1.0)),
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.35,
            },
            corner_radii: Corners::all(Px(8.0)),
        });

        // Note: no text rendering yet; rely on logs for path details.
        let _ = &self.last_path;
    }
}
