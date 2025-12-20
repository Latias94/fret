use fret_core::{Color, Corners, DrawOrder, Edges, Event, Px, Rect, SceneOp, Size, TextStyle};
use fret_editor::{AssetGuid, ProjectEntryKind, ProjectSelectionService, ProjectService};
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, TreeView, Widget};

#[derive(Debug)]
pub struct ProjectPanel {
    tree: TreeView,
    last_revision: Option<u64>,
    last_selected: Option<u64>,
}

impl ProjectPanel {
    pub fn new() -> Self {
        Self {
            tree: TreeView::new(Vec::new()),
            last_revision: None,
            last_selected: None,
        }
    }

    fn maybe_refresh(&mut self, app: &fret_app::App) -> bool {
        let Some(service) = app.global::<ProjectService>() else {
            return false;
        };
        let revision = service.revision();
        if self.last_revision == Some(revision) {
            return false;
        }
        let snapshot = service.snapshot();
        self.tree.set_roots(snapshot.roots);
        self.last_revision = Some(revision);
        true
    }

    fn selected_details(
        &self,
        app: &fret_app::App,
    ) -> Option<(String, ProjectEntryKind, AssetGuid)> {
        let Some(service) = app.global::<ProjectService>() else {
            return None;
        };
        let id = self.tree.selected()?;
        let path = service.path_for_id(id)?;
        let kind = service.kind_for_id(id)?;
        let guid = service.guid_for_id(id)?;
        Some((path.to_string_lossy().to_string(), kind, guid))
    }
}

impl Widget for ProjectPanel {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        self.tree.event(cx, event);

        let selected = self.tree.selected();
        if selected != self.last_selected {
            self.last_selected = selected;
            cx.app
                .with_global_mut(ProjectSelectionService::default, |s, _app| {
                    s.set_selected(selected);
                });
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        let _ = self.maybe_refresh(cx.app);
        self.tree.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let _ = self.maybe_refresh(cx.app);
        self.tree.paint(cx);

        let Some((path, kind, guid)) = self.selected_details(cx.app) else {
            return;
        };

        let h = Px(28.0);
        if cx.bounds.size.height.0 <= h.0 {
            return;
        }

        let rect = Rect::new(
            fret_core::Point::new(
                cx.bounds.origin.x,
                Px(cx.bounds.origin.y.0 + cx.bounds.size.height.0 - h.0),
            ),
            Size::new(cx.bounds.size.width, h),
        );

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(200),
            rect,
            background: Color {
                r: 0.08,
                g: 0.09,
                b: 0.11,
                a: 0.92,
            },
            border: Edges::all(Px(1.0)),
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.40,
            },
            corner_radii: Corners::all(Px(0.0)),
        });

        let label = match kind {
            ProjectEntryKind::Directory => format!("Folder  GUID: {}  {}", guid.0, path),
            ProjectEntryKind::File => format!("Asset   GUID: {}  {}", guid.0, path),
        };
        let style = TextStyle {
            font: fret_core::FontId::default(),
            size: Px(12.0),
        };
        let (blob, metrics) = cx.text.prepare(
            label.as_str(),
            style,
            fret_core::TextConstraints {
                max_width: Some(Px(rect.size.width.0 - 12.0)),
                wrap: fret_core::TextWrap::None,
                scale_factor: cx.scale_factor,
            },
        );
        let baseline_y =
            rect.origin.y.0 + (h.0 - metrics.size.height.0) * 0.5 + metrics.baseline.0;
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(201),
            origin: fret_core::Point::new(Px(rect.origin.x.0 + 6.0), Px(baseline_y)),
            text: blob,
            color: Color {
                r: 0.90,
                g: 0.90,
                b: 0.92,
                a: 1.0,
            },
        });
        cx.text.release(blob);
    }
}
