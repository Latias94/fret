use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, ExternalDragKind, MouseButton, Px, SceneOp, Size,
};
use fret_ui_app::{App, EventCx, GenericWidget, Invalidation, LayoutCx, PaintCx};

#[derive(Debug, Default)]
pub struct DndProbe {
    hovering: bool,
    last_name: Option<String>,
}

impl DndProbe {
    pub fn new() -> Self {
        Self::default()
    }
}

impl GenericWidget<App> for DndProbe {
    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        match event {
            Event::ExternalDrag(drag) => {
                match &drag.kind {
                    ExternalDragKind::EnterFiles(files) | ExternalDragKind::OverFiles(files) => {
                        self.hovering = true;
                        self.last_name = files.files.last().map(|f| f.name.clone());
                        tracing::info!(count = files.files.len(), "external drag hover");
                    }
                    ExternalDragKind::DropFiles(files) => {
                        self.hovering = false;
                        self.last_name = files.files.last().map(|f| f.name.clone());
                        tracing::info!(count = files.files.len(), "external drag drop");
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
                    self.last_name = None;
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
        let theme = cx.theme().snapshot();
        let (bg, border_color) = if self.hovering {
            (
                Color {
                    a: 0.22,
                    ..theme.colors.accent
                },
                Color {
                    a: 0.85,
                    ..theme.colors.accent
                },
            )
        } else {
            (theme.colors.panel_background, theme.colors.panel_border)
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: bg,
            border: Edges::all(Px(1.0)),
            border_color,
            corner_radii: Corners::all(theme.metrics.radius_md),
        });

        // Note: no text rendering yet; rely on logs for path details.
        let _ = &self.last_name;
    }
}
