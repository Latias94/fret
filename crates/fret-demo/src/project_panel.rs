use fret_app::{CommandId, InputContext, Menu, MenuItem};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, Modifiers, MouseButton, Px, Rect, SceneOp, Size,
    TextStyle,
};
use fret_editor::{AssetGuid, ProjectEntryKind, ProjectSelectionService, ProjectService};
use fret_ui::{
    ContextMenuRequest, ContextMenuService, EventCx, Invalidation, LayoutCx, PaintCx, TreeView,
    Widget,
};

#[derive(Debug)]
pub struct ProjectPanel {
    tree: TreeView,
    last_revision: Option<u64>,
    last_selected: Option<u64>,
    last_selection_revision: Option<u64>,
}

impl ProjectPanel {
    pub fn new() -> Self {
        Self {
            tree: TreeView::new(Vec::new()),
            last_revision: None,
            last_selected: None,
            last_selection_revision: None,
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
        self.last_selection_revision = None;
        self.sync_selection_from_service(app);
        true
    }

    fn sync_selection_from_service(&mut self, app: &fret_app::App) {
        let selection_revision = app
            .global::<ProjectSelectionService>()
            .map(|s| s.revision())
            .unwrap_or(0);
        if self.last_selection_revision == Some(selection_revision) {
            return;
        }
        self.last_selection_revision = Some(selection_revision);

        let Some(project) = app.global::<ProjectService>() else {
            return;
        };
        let guid = app
            .global::<ProjectSelectionService>()
            .and_then(|s| s.selected_guid());
        let selected = guid.and_then(|g| project.id_for_guid(g));

        if selected != self.tree.selected() {
            if let Some(id) = selected {
                self.tree.set_selected_keys([id], Some(id));
            } else {
                self.tree.set_selected_keys([], None);
            }
            self.last_selected = self.tree.selected();
        }
    }

    fn selected_details(
        &self,
        app: &fret_app::App,
    ) -> Option<(String, ProjectEntryKind, AssetGuid)> {
        let Some(service) = app.global::<ProjectService>() else {
            return None;
        };
        let id = app
            .global::<ProjectSelectionService>()
            .and_then(|s| s.selected_guid())
            .and_then(|g| service.id_for_guid(g))
            .or_else(|| self.tree.selected())?;
        let path = service.path_for_id(id)?;
        let kind = service.kind_for_id(id)?;
        let guid = service.guid_for_id(id)?;
        Some((path.to_string_lossy().to_string(), kind, guid))
    }

    fn open_context_menu_at(
        &mut self,
        cx: &mut EventCx<'_>,
        position: fret_core::Point,
        row_id: u64,
    ) {
        let Some(window) = cx.window else {
            return;
        };

        if let Some(project) = cx.app.global::<ProjectService>() {
            let guid = project.guid_for_id(row_id);
            cx.app
                .with_global_mut(ProjectSelectionService::default, |s, _app| {
                    s.set_selected_guid(guid);
                });
        }

        let inv_ctx = InputContext {
            platform: cx.input_ctx.platform,
            ui_has_modal: cx.input_ctx.ui_has_modal,
            focus_is_text_input: false,
        };

        let menu = Menu {
            title: std::sync::Arc::from("Project"),
            items: vec![
                MenuItem::Command {
                    command: CommandId::from("project.rename_selected"),
                    when: None,
                },
                MenuItem::Command {
                    command: CommandId::from("project.move_selected_to_moved"),
                    when: None,
                },
                MenuItem::Separator,
                MenuItem::Command {
                    command: CommandId::from("project.refresh"),
                    when: None,
                },
            ],
        };

        cx.app
            .with_global_mut(ContextMenuService::default, |service, _app| {
                service.set_request(
                    window,
                    ContextMenuRequest {
                        position,
                        menu,
                        input_ctx: inv_ctx,
                    },
                );
            });
        cx.dispatch_command(CommandId::from("context_menu.open"));
        cx.request_redraw();
    }
}

impl Widget for ProjectPanel {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        if let Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Right,
            modifiers: Modifiers { .. },
        }) = event
        {
            let _ = self.maybe_refresh(cx.app);
            if let Some(row_id) = self.tree.row_id_at(*position) {
                cx.request_focus(cx.node);
                self.tree.set_selected_keys([row_id], Some(row_id));
                self.open_context_menu_at(cx, *position, row_id);
                cx.stop_propagation();
                return;
            }
        }

        self.tree.event(cx, event);

        let selected = self.tree.selected();
        if selected != self.last_selected {
            self.last_selected = selected;
            if let Some(project) = cx.app.global::<ProjectService>() {
                let guid = selected.and_then(|id| project.guid_for_id(id));
                cx.app
                    .with_global_mut(ProjectSelectionService::default, |s, _app| {
                        s.set_selected_guid(guid);
                    });
            }
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        let _ = self.maybe_refresh(cx.app);
        self.sync_selection_from_service(cx.app);
        self.tree.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let _ = self.maybe_refresh(cx.app);
        self.sync_selection_from_service(cx.app);
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
        let baseline_y = rect.origin.y.0 + (h.0 - metrics.size.height.0) * 0.5 + metrics.baseline.0;
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
