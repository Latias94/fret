use fret_app::{CommandId, InputContext, Menu, MenuItem};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, Modifiers, MouseButton, Px, Rect, SceneOp,
    Size, TextStyle,
};
use fret_editor::{AssetGuid, ProjectEntryKind, ProjectSelectionService, ProjectService};
use fret_ui_app::{
    App, ContextMenuRequest, ContextMenuService, EventCx, GenericWidget, Invalidation, LayoutCx,
    PaintCx, TreeView,
};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProjectDragPayload {
    pub(crate) guid: AssetGuid,
}

#[derive(Debug)]
pub struct ProjectPanel {
    tree: TreeView,
    last_revision: Option<u64>,
    last_selected: Option<u64>,
    last_selection_revision: Option<u64>,
    drag: Option<ProjectDragState>,
    last_click: Option<(u64, Instant)>,
}

#[derive(Debug, Clone)]
struct ProjectDragState {
    guid: AssetGuid,
    start: fret_core::Point,
    dragging: bool,
    hover_folder: Option<AssetGuid>,
}

impl ProjectPanel {
    pub fn new() -> Self {
        Self {
            tree: TreeView::new(Vec::new()),
            last_revision: None,
            last_selected: None,
            last_selection_revision: None,
            drag: None,
            last_click: None,
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
            caps: cx.input_ctx.caps.clone(),
            ui_has_modal: cx.input_ctx.ui_has_modal,
            focus_is_text_input: false,
        };

        let menu = Menu {
            title: std::sync::Arc::from("Project"),
            items: vec![
                MenuItem::Command {
                    command: CommandId::from("asset.open_selected"),
                    when: None,
                },
                MenuItem::Separator,
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
                        menu_bar: None,
                    },
                );
            });
        cx.dispatch_command(CommandId::from("context_menu.open"));
        cx.request_redraw();
    }

    fn begin_drag_candidate(&mut self, app: &fret_app::App, position: fret_core::Point) {
        let Some(id) = self.tree.row_id_at(position) else {
            return;
        };
        let Some(project) = app.global::<ProjectService>() else {
            return;
        };
        let Some(guid) = project.guid_for_id(id) else {
            return;
        };
        self.drag = Some(ProjectDragState {
            guid,
            start: position,
            dragging: false,
            hover_folder: None,
        });
    }

    fn folder_guid_at(
        &mut self,
        app: &fret_app::App,
        position: fret_core::Point,
    ) -> Option<AssetGuid> {
        let Some(project) = app.global::<ProjectService>() else {
            return None;
        };
        let row_id = self.tree.row_id_at(position)?;
        match project.kind_for_id(row_id)? {
            ProjectEntryKind::Directory => project.guid_for_id(row_id),
            ProjectEntryKind::File => {
                let path = project.path_for_id(row_id)?;
                let parent = path.parent()?;
                project.guid_for_path(parent)
            }
        }
    }

    fn update_drag_preview(&mut self, app: &fret_app::App, position: fret_core::Point) {
        let hover = {
            let Some(drag) = self.drag.as_ref() else {
                return;
            };
            if !drag.dragging {
                return;
            }

            let content = self.tree.content_bounds();
            if !content.contains(position) {
                None
            } else {
                self.folder_guid_at(app, position)
            }
        };

        if let Some(drag) = self.drag.as_mut() {
            drag.hover_folder = hover;
        }
    }
}

impl GenericWidget<App> for ProjectPanel {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        if let Event::KeyDown {
            key: KeyCode::Enter,
            modifiers,
            repeat: false,
        } = event
        {
            if modifiers.ctrl || modifiers.meta || modifiers.shift || modifiers.alt {
                self.tree.event(cx, event);
                return;
            }
            cx.dispatch_command(CommandId::from("asset.open_selected"));
            cx.stop_propagation();
            return;
        }

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

        match event {
            Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button: MouseButton::Left,
                modifiers,
            }) => {
                if modifiers.ctrl || modifiers.meta || modifiers.shift || modifiers.alt {
                    self.tree.event(cx, event);
                    return;
                }
                self.tree.event(cx, event);
                if cx.app.drag().is_none() {
                    self.begin_drag_candidate(cx.app, *position);
                }
            }
            Event::Pointer(fret_core::PointerEvent::Move {
                position, buttons, ..
            }) => {
                if buttons.left {
                    if let Some(mut drag) = self.drag.take() {
                        if !drag.dragging {
                            let dx = position.x.0 - drag.start.x.0;
                            let dy = position.y.0 - drag.start.y.0;
                            let dist2 = dx * dx + dy * dy;
                            if dist2 > 16.0 {
                                drag.dragging = true;
                                if let Some(window) = cx.window {
                                    cx.app.begin_cross_window_drag_with_kind(
                                        fret_app::DragKind::Custom,
                                        window,
                                        drag.start,
                                        ProjectDragPayload { guid: drag.guid },
                                    );
                                }
                                cx.capture_pointer(cx.node);
                            }
                        }

                        self.drag = Some(drag);
                        self.update_drag_preview(cx.app, *position);

                        if self.drag.as_ref().is_some_and(|d| d.dragging) {
                            if let Some(session) = cx.app.drag_mut() {
                                if session.payload::<ProjectDragPayload>().is_some() {
                                    session.position = *position;
                                    session.dragging = true;
                                }
                            }
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                            return;
                        }
                    }
                }
                self.tree.event(cx, event);
            }
            Event::Pointer(fret_core::PointerEvent::Up {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers { .. },
            }) => {
                if let Some(drag) = self.drag.take() {
                    if drag.dragging {
                        let dropped_outside_source_window = cx.window.is_some_and(|window| {
                            cx.app.drag().is_some_and(|session| {
                                session.payload::<ProjectDragPayload>().is_some()
                                    && session.current_window != window
                            })
                        });

                        if dropped_outside_source_window {
                            cx.release_pointer_capture();
                            if cx
                                .app
                                .drag()
                                .and_then(|d| d.payload::<ProjectDragPayload>())
                                .is_some()
                            {
                                cx.app.cancel_drag();
                            }
                            self.drag = None;
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                            return;
                        }

                        let target = drag
                            .hover_folder
                            .or_else(|| self.folder_guid_at(cx.app, *position));

                        let mut ok = false;
                        if let (Some(target), Some(project)) =
                            (target, cx.app.global_mut::<ProjectService>())
                        {
                            if let Err(err) = project.move_guid_into_folder(drag.guid, target) {
                                tracing::error!(error = %err, "project drag move failed");
                            } else if let Err(err) = project.rescan() {
                                tracing::error!(error = %err, "failed to rescan project after drag move");
                            } else {
                                ok = true;
                            }
                        }

                        if ok {
                            cx.app
                                .with_global_mut(ProjectSelectionService::default, |s, _app| {
                                    s.set_selected_guid(Some(drag.guid));
                                });
                        }

                        cx.release_pointer_capture();
                        if cx
                            .app
                            .drag()
                            .and_then(|d| d.payload::<ProjectDragPayload>())
                            .is_some()
                        {
                            cx.app.cancel_drag();
                        }
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
                }
                self.tree.event(cx, event);

                if let Some(row_id) = self.tree.row_id_at(*position) {
                    let now = Instant::now();
                    let is_double_click = self.last_click.is_some_and(|(prev, at)| {
                        prev == row_id && now.duration_since(at) <= Duration::from_millis(400)
                    });
                    self.last_click = Some((row_id, now));

                    if is_double_click {
                        if let Some(project) = cx.app.global::<ProjectService>() {
                            if project.kind_for_id(row_id) == Some(ProjectEntryKind::File) {
                                let guid = project.guid_for_id(row_id);
                                cx.app.with_global_mut(
                                    ProjectSelectionService::default,
                                    |s, _app| {
                                        s.set_selected_guid(guid);
                                    },
                                );
                                cx.dispatch_command(CommandId::from("asset.open_selected"));
                                cx.request_redraw();
                                cx.stop_propagation();
                                return;
                            }
                        }
                    }
                }
            }
            _ => {
                self.tree.event(cx, event);
            }
        }

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
        let theme = cx.theme().snapshot();
        let _ = self.maybe_refresh(cx.app);
        self.sync_selection_from_service(cx.app);
        self.tree.paint(cx);

        if let Some(drag) = self.drag.as_ref() {
            if drag.dragging {
                if let Some(target_guid) = drag.hover_folder {
                    if let Some(project) = cx.app.global::<ProjectService>() {
                        if let Some(target_id) = project.id_for_guid(target_guid) {
                            if let Some(rect) = self.tree.row_rect(target_id) {
                                let stroke = Color {
                                    a: 0.9,
                                    ..theme.colors.accent
                                };
                                cx.scene.push(SceneOp::Quad {
                                    order: DrawOrder(190),
                                    rect,
                                    background: Color::TRANSPARENT,
                                    border: Edges::all(Px(2.0)),
                                    border_color: stroke,
                                    corner_radii: Corners::all(Px(0.0)),
                                });
                            }
                        }
                    }
                }
            }
        }

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
                a: 0.92,
                ..theme.colors.panel_background
            },
            border: Edges::all(Px(1.0)),
            border_color: theme.colors.panel_border,
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
            color: theme.colors.text_primary,
        });
        cx.text.release(blob);
    }
}
