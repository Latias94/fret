use crate::asset_drop::CurrentSceneService;
use crate::asset_drop::{AssetDropRequest, AssetDropService, AssetDropTarget};
use crate::hierarchy::{DemoHierarchy, HierarchyDropKind, HierarchyDropTarget};
use crate::project_panel::ProjectDragPayload;
use crate::scene_document::SceneDocumentService;
use crate::undo::{EditCommand, SelectionSnapshot, UndoStack};
use crate::world::DemoWorld;
use fret_app::{App, Model};
use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect,
    SceneOp, Size, TextConstraints, TextMetrics, TextStyle, TextWrap,
};
use fret_editor::{
    InspectorEditKind, InspectorEditRequest, InspectorEditService, InspectorEditorKind,
    InspectorEditorRegistry, ProjectEntryKind, ProjectSelectionService, ProjectService,
    PropertyEditKind, PropertyEditRequest, PropertyEditService, PropertyLeaf, PropertyMeta,
    PropertyNode, PropertyPath, PropertyTree, PropertyTypeTag, PropertyValue,
};
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, ThemeSnapshot, TreeView, Widget};

#[derive(Debug, Default, Clone)]
pub struct DemoSelection {
    pub lead_entity: Option<u64>,
    pub selected_entities: Vec<u64>,
}

#[derive(Debug, Clone)]
enum InspectorRow {
    Header {
        label: String,
    },
    Property {
        label: String,
        value: String,
        action: Option<InspectorRowAction>,
    },
}

#[derive(Debug, Clone)]
enum InspectorRowAction {
    ToggleBool {
        targets: Vec<u64>,
        path: PropertyPath,
        current: Option<bool>,
    },
    EditValue {
        request: InspectorEditRequest,
    },
}

#[derive(Debug, Clone)]
struct InspectorDataSource {
    rows: Vec<InspectorRow>,
}

impl InspectorDataSource {
    fn empty() -> Self {
        Self { rows: Vec::new() }
    }

    fn new(app: &App, world: Model<DemoWorld>, targets: Vec<u64>) -> Self {
        let mut rows: Vec<InspectorRow> = Vec::new();

        let current_scene_guid = app.global::<CurrentSceneService>().and_then(|s| s.guid());
        if let Some(guid) = current_scene_guid {
            let name = app
                .global::<ProjectService>()
                .and_then(|p| p.id_for_guid(guid).and_then(|id| p.path_for_id(id)))
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("Scene {}", guid.0));
            rows.push(InspectorRow::Header {
                label: format!("Scene: {name}"),
            });
        }

        let project_selected_guid = app
            .global::<ProjectSelectionService>()
            .and_then(|s| s.selected_guid());
        if let Some(guid) = project_selected_guid {
            let Some(project) = app.global::<ProjectService>() else {
                rows.push(InspectorRow::Header {
                    label: "Project model missing".to_string(),
                });
                return Self { rows };
            };

            let (path, kind) = match project.id_for_guid(guid) {
                Some(id) => {
                    let kind = project.kind_for_id(id);
                    let path = project
                        .path_for_id(id)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "<missing path>".to_string());
                    (path, kind)
                }
                None => ("<missing entry for GUID>".to_string(), None),
            };

            rows.push(InspectorRow::Header {
                label: "Asset".to_string(),
            });
            rows.push(InspectorRow::Property {
                label: "Path".to_string(),
                value: path,
                action: None,
            });
            rows.push(InspectorRow::Property {
                label: "Kind".to_string(),
                value: match kind {
                    Some(ProjectEntryKind::Directory) => "Folder".to_string(),
                    Some(ProjectEntryKind::File) => "File".to_string(),
                    None => "Unknown".to_string(),
                },
                action: None,
            });
            rows.push(InspectorRow::Property {
                label: "GUID".to_string(),
                value: guid.0.to_string(),
                action: None,
            });
            return Self { rows };
        }

        if targets.is_empty() {
            rows.push(InspectorRow::Header {
                label: "No selection (click in Hierarchy)".to_string(),
            });
            return Self { rows };
        }

        let Some(world) = world.get(app) else {
            rows.push(InspectorRow::Header {
                label: "World model missing".to_string(),
            });
            return Self { rows };
        };

        fn mixed_value(world: &DemoWorld, targets: &[u64], path: &PropertyPath) -> PropertyValue {
            let Some(first) = targets.first().and_then(|id| world.get_property(*id, path)) else {
                return PropertyValue::Mixed;
            };
            for &id in targets.iter().skip(1) {
                if world.get_property(id, path) != Some(first.clone()) {
                    return PropertyValue::Mixed;
                }
            }
            first
        }

        let path_name = PropertyPath::new().field("name");
        let path_active = PropertyPath::new().field("active");
        let path_pos = PropertyPath::new().field("transform").field("position");
        let path_rot = PropertyPath::new().field("transform").field("rotation_y");
        let path_scale = PropertyPath::new().field("transform").field("scale");

        let mut registry = InspectorEditorRegistry::default();
        registry.register_path_prefix(
            PropertyTypeTag::new("f32"),
            path_rot.clone(),
            InspectorEditorKind::AngleDegreesPopup,
        );

        let tree = PropertyTree {
            roots: vec![
                PropertyNode::Group {
                    label: format!("GameObject ({})", targets.len()),
                    children: vec![
                        PropertyNode::Leaf(PropertyLeaf {
                            path: path_name.clone(),
                            label: "Name".to_string(),
                            type_tag: PropertyTypeTag::new("string"),
                            value: mixed_value(world, &targets, &path_name),
                            meta: PropertyMeta::default(),
                        }),
                        PropertyNode::Leaf(PropertyLeaf {
                            path: path_active.clone(),
                            label: "Active".to_string(),
                            type_tag: PropertyTypeTag::new("bool"),
                            value: mixed_value(world, &targets, &path_active),
                            meta: PropertyMeta::default(),
                        }),
                    ],
                },
                PropertyNode::Group {
                    label: "Transform".to_string(),
                    children: vec![
                        PropertyNode::Leaf(PropertyLeaf {
                            path: path_pos.clone(),
                            label: "Position".to_string(),
                            type_tag: PropertyTypeTag::new("vec3"),
                            value: mixed_value(world, &targets, &path_pos),
                            meta: PropertyMeta::default(),
                        }),
                        PropertyNode::Leaf(PropertyLeaf {
                            path: path_rot.clone(),
                            label: "Rotation Y".to_string(),
                            type_tag: PropertyTypeTag::new("f32"),
                            value: mixed_value(world, &targets, &path_rot),
                            meta: PropertyMeta::default(),
                        }),
                        PropertyNode::Leaf(PropertyLeaf {
                            path: path_scale.clone(),
                            label: "Scale".to_string(),
                            type_tag: PropertyTypeTag::new("f32"),
                            value: mixed_value(world, &targets, &path_scale),
                            meta: PropertyMeta::default(),
                        }),
                    ],
                },
            ],
        };

        fn flatten(
            out: &mut Vec<InspectorRow>,
            node: &PropertyNode,
            registry: &InspectorEditorRegistry,
            targets: &[u64],
        ) {
            match node {
                PropertyNode::Group { label, children } => {
                    out.push(InspectorRow::Header {
                        label: label.clone(),
                    });
                    for c in children {
                        flatten(out, c, registry, targets);
                    }
                }
                PropertyNode::Leaf(leaf) => {
                    let value = registry.display_value(leaf);
                    let kind = registry.resolve_kind(leaf);
                    let action = match kind {
                        InspectorEditorKind::BoolToggle => match &leaf.value {
                            PropertyValue::Bool(v) => Some(InspectorRowAction::ToggleBool {
                                targets: targets.to_vec(),
                                path: leaf.path.clone(),
                                current: Some(*v),
                            }),
                            PropertyValue::Mixed => Some(InspectorRowAction::ToggleBool {
                                targets: targets.to_vec(),
                                path: leaf.path.clone(),
                                current: None,
                            }),
                            _ => None,
                        },
                        InspectorEditorKind::TextPopup => Some(InspectorRowAction::EditValue {
                            request: InspectorEditRequest {
                                targets: targets.to_vec(),
                                path: leaf.path.clone(),
                                kind: InspectorEditKind::String,
                                initial_text: match &leaf.value {
                                    PropertyValue::String(v) => v.clone(),
                                    _ => String::new(),
                                },
                                anchor: None,
                                preferred_width: None,
                            },
                        }),
                        InspectorEditorKind::NumberPopup
                        | InspectorEditorKind::AngleDegreesPopup => {
                            Some(InspectorRowAction::EditValue {
                                request: InspectorEditRequest {
                                    targets: targets.to_vec(),
                                    path: leaf.path.clone(),
                                    kind: InspectorEditKind::F32,
                                    initial_text: match &leaf.value {
                                        PropertyValue::F32(v) => match kind {
                                            InspectorEditorKind::AngleDegreesPopup => {
                                                format!("{v:.1}")
                                            }
                                            _ => format!("{v:.3}"),
                                        },
                                        _ => String::new(),
                                    },
                                    anchor: None,
                                    preferred_width: Some(Px(240.0)),
                                },
                            })
                        }
                        InspectorEditorKind::Vec3Popup => Some(InspectorRowAction::EditValue {
                            request: InspectorEditRequest {
                                targets: targets.to_vec(),
                                path: leaf.path.clone(),
                                kind: InspectorEditKind::Vec3,
                                initial_text: match &leaf.value {
                                    PropertyValue::Vec3([x, y, z]) => {
                                        format!("{x:.3}, {y:.3}, {z:.3}")
                                    }
                                    _ => String::new(),
                                },
                                anchor: None,
                                preferred_width: Some(Px(320.0)),
                            },
                        }),
                    };

                    out.push(InspectorRow::Property {
                        label: leaf.label.clone(),
                        value,
                        action,
                    });
                }
            }
        }

        for n in &tree.roots {
            flatten(&mut rows, n, &registry, &targets);
        }

        Self { rows }
    }

    fn action_at(&self, index: usize) -> Option<InspectorRowAction> {
        match self.rows.get(index)? {
            InspectorRow::Property { action, .. } => action.clone(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
}

#[derive(Debug, Clone)]
enum PreparedValue {
    Text(PreparedText),
    Vec3([PreparedText; 3]),
}

#[derive(Debug, Clone)]
struct PreparedInspectorRow {
    kind: PreparedInspectorRowKind,
    action: Option<InspectorRowAction>,
}

#[derive(Debug, Clone)]
enum PreparedInspectorRowKind {
    Header {
        label: PreparedText,
    },
    Property {
        label: PreparedText,
        value: PreparedValue,
        value_kind: InspectorEditKind,
    },
}

#[derive(Debug, Clone)]
enum ScrubValue {
    F32 {
        base: f32,
        current: f32,
    },
    Vec3 {
        axis: usize,
        base: [f32; 3],
        current: [f32; 3],
    },
}

#[derive(Debug, Clone)]
struct ScrubState {
    window: AppWindowId,
    targets: Vec<u64>,
    path: PropertyPath,
    start_x: Px,
    value: ScrubValue,
}

#[derive(Debug, Clone)]
struct ToggleGlyph {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
}

pub struct HierarchyPanel {
    tree: TreeView,
    selection: Model<DemoSelection>,
    hierarchy: Model<DemoHierarchy>,
    undo: Model<UndoStack>,
    drag: Option<HierarchyDragState>,
    asset_drop: Option<AssetDropPreview>,
    last_selected: Option<u64>,
    last_selected_keys: Vec<u64>,
    last_revision: Option<u64>,
    last_hierarchy_revision: Option<u64>,
    did_init_expanded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssetDropPreviewKind {
    HighlightRow,
    AppendRoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AssetDropPreview {
    kind: AssetDropPreviewKind,
    parent: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HierarchyDropPreviewKind {
    InsertLineAbove,
    InsertLineBelow,
    HighlightRow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HierarchyDropPreview {
    target: HierarchyDropTarget,
    kind: HierarchyDropPreviewKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct HierarchyDragState {
    id: u64,
    start: fret_core::Point,
    dragging: bool,
    preview: Option<HierarchyDropPreview>,
}

impl HierarchyPanel {
    pub fn new(
        selection: Model<DemoSelection>,
        hierarchy: Model<DemoHierarchy>,
        undo: Model<UndoStack>,
    ) -> Self {
        Self {
            tree: TreeView::new(Vec::new()),
            selection,
            hierarchy,
            undo,
            drag: None,
            asset_drop: None,
            last_selected: None,
            last_selected_keys: Vec::new(),
            last_revision: None,
            last_hierarchy_revision: None,
            did_init_expanded: false,
        }
    }

    fn asset_drop_preview_at(
        &mut self,
        app: &App,
        position: fret_core::Point,
    ) -> Option<AssetDropPreview> {
        let Some(session) = app.drag() else {
            return None;
        };
        if !session.dragging {
            return None;
        }
        let payload = session.payload::<ProjectDragPayload>()?;
        let Some(project) = app.global::<ProjectService>() else {
            return None;
        };
        let Some(id) = project.id_for_guid(payload.guid) else {
            return None;
        };
        if project.kind_for_id(id) != Some(ProjectEntryKind::File) {
            return None;
        }

        let content = self.tree.content_bounds();
        if !content.contains(position) {
            return None;
        }

        if let Some(target) = self.tree.row_id_at(position) {
            Some(AssetDropPreview {
                kind: AssetDropPreviewKind::HighlightRow,
                parent: Some(target),
            })
        } else {
            Some(AssetDropPreview {
                kind: AssetDropPreviewKind::AppendRoot,
                parent: None,
            })
        }
    }

    fn commit_asset_drop(&mut self, cx: &mut EventCx<'_>, preview: AssetDropPreview) {
        let Some(window) = cx.window else {
            return;
        };
        let guid = cx
            .app
            .drag()
            .and_then(|session| session.dragging.then_some(session))
            .and_then(|session| session.payload::<ProjectDragPayload>())
            .map(|p| p.guid);
        let Some(guid) = guid else {
            return;
        };

        cx.app
            .with_global_mut(AssetDropService::default, |s, _app| {
                s.push(AssetDropRequest {
                    window,
                    guid,
                    target: AssetDropTarget::Hierarchy {
                        parent: preview.parent,
                    },
                });
            });
    }

    fn maybe_sync_hierarchy(&mut self, app: &App) {
        let revision = self.hierarchy.revision(app);
        if revision == self.last_hierarchy_revision {
            return;
        }
        let roots = self
            .hierarchy
            .get(app)
            .map(|h| h.roots.clone())
            .unwrap_or_default();
        self.tree.set_roots(roots);

        if !self.did_init_expanded {
            let mut ids: Vec<u64> = Vec::new();
            for root in self
                .hierarchy
                .get(app)
                .map(|h| h.roots.iter())
                .into_iter()
                .flatten()
            {
                ids.push(root.id);
                for child in &root.children {
                    ids.push(child.id);
                }
            }
            self.tree.set_expanded(ids);
            self.did_init_expanded = true;
        }

        self.last_hierarchy_revision = revision;
    }

    fn maybe_sync_from_model(&mut self, app: &App) {
        self.maybe_sync_hierarchy(app);

        let revision = self.selection.revision(app);
        if revision == self.last_revision {
            return;
        }

        let next = self.selection.get(app).cloned().unwrap_or_default();
        let lead = next.lead_entity;
        let selected = next.selected_entities;

        let mut cur: Vec<u64> = self.tree.selected_keys().iter().copied().collect();
        cur.sort_unstable();

        if lead == self.tree.selected() && cur == selected {
            self.last_revision = revision;
            return;
        }

        if let Some(lead) = lead {
            self.tree.reveal(lead);
        }
        self.tree.set_selected_keys(selected.clone(), lead);
        self.last_selected = self.tree.selected();
        self.last_selected_keys = self.tree.selected_keys().iter().copied().collect();
        self.last_selected_keys.sort_unstable();
        self.last_revision = revision;
    }

    fn sync_selection_model(&mut self, cx: &mut EventCx<'_>) {
        let next = self.tree.selected();
        let mut selected: Vec<u64> = self.tree.selected_keys().iter().copied().collect();
        selected.sort_unstable();
        if next == self.last_selected && selected == self.last_selected_keys {
            return;
        }
        self.last_selected = next;
        self.last_selected_keys = selected.clone();
        let _ = self.selection.update(cx.app, |state, _cx| {
            state.lead_entity = next;
            state.selected_entities = selected;
        });
        self.last_revision = self.selection.revision(cx.app);

        cx.app
            .with_global_mut(ProjectSelectionService::default, |s, _app| {
                s.set_selected_guid(None);
            });

        cx.request_redraw();
    }

    fn begin_drag_candidate(&mut self, position: fret_core::Point) {
        let Some(id) = self.tree.row_id_at(position) else {
            return;
        };
        self.drag = Some(HierarchyDragState {
            id,
            start: position,
            dragging: false,
            preview: None,
        });
    }

    fn update_drag_preview(&mut self, app: &App, position: fret_core::Point) {
        let Some(drag) = self.drag.as_mut() else {
            return;
        };
        if !drag.dragging {
            return;
        }

        let content = self.tree.content_bounds();
        if !content.contains(position) {
            drag.preview = None;
            return;
        }

        let Some(h) = self.hierarchy.get(app) else {
            drag.preview = None;
            return;
        };

        let preview = if let Some(target_id) = self.tree.row_id_at(position) {
            if target_id == drag.id || h.is_descendant_of(drag.id, target_id) {
                None
            } else if let Some(rect) = self.tree.row_rect(target_id) {
                let rel_y = (position.y.0 - rect.origin.y.0) / rect.size.height.0.max(1.0);
                if rel_y < 0.25 {
                    Some(HierarchyDropPreview {
                        target: HierarchyDropTarget {
                            kind: HierarchyDropKind::InsertAbove,
                            target_id: Some(target_id),
                        },
                        kind: HierarchyDropPreviewKind::InsertLineAbove,
                    })
                } else if rel_y > 0.75 {
                    Some(HierarchyDropPreview {
                        target: HierarchyDropTarget {
                            kind: HierarchyDropKind::InsertBelow,
                            target_id: Some(target_id),
                        },
                        kind: HierarchyDropPreviewKind::InsertLineBelow,
                    })
                } else {
                    Some(HierarchyDropPreview {
                        target: HierarchyDropTarget {
                            kind: HierarchyDropKind::ReparentInto,
                            target_id: Some(target_id),
                        },
                        kind: HierarchyDropPreviewKind::HighlightRow,
                    })
                }
            } else {
                None
            }
        } else if self.tree.content_bounds().contains(position) {
            Some(HierarchyDropPreview {
                target: HierarchyDropTarget {
                    kind: HierarchyDropKind::AppendRoot,
                    target_id: None,
                },
                kind: HierarchyDropPreviewKind::InsertLineBelow,
            })
        } else {
            None
        };

        drag.preview = preview;
    }

    fn commit_drag(&mut self, cx: &mut EventCx<'_>) -> bool {
        let Some(drag) = self.drag.take() else {
            return false;
        };
        if !drag.dragging {
            return false;
        }

        let Some(preview) = drag.preview else {
            return true;
        };

        let Some(op) = self
            .hierarchy
            .get(cx.app)
            .and_then(|h| h.move_op_for_drop(drag.id, preview.target))
        else {
            return true;
        };

        let Some((from_parent, from_index)) =
            self.hierarchy.get(cx.app).and_then(|h| h.locate(drag.id))
        else {
            return true;
        };

        let moved = self.hierarchy.update(cx.app, |h, _cx| h.apply_move(op));
        let moved = moved.unwrap_or(false);
        if !moved {
            return true;
        }

        let selection_snapshot = self
            .selection
            .get(cx.app)
            .map(SelectionSnapshot::from_selection)
            .unwrap_or(SelectionSnapshot {
                lead_entity: None,
                selected_entities: Vec::new(),
            });

        let _ = self.undo.update(cx.app, |stack, _cx| {
            stack.push(EditCommand::HierarchyMove {
                op,
                from_parent,
                from_index,
                selection: selection_snapshot,
            });
        });
        let has_scene = cx
            .app
            .global::<CurrentSceneService>()
            .and_then(|s| s.guid())
            .is_some();
        if has_scene {
            cx.app
                .with_global_mut(SceneDocumentService::default, |s, _app| {
                    s.set_dirty(true);
                });
        }

        self.last_hierarchy_revision = self.hierarchy.revision(cx.app);

        cx.invalidate_self(Invalidation::Layout);
        cx.invalidate_self(Invalidation::Paint);
        cx.request_redraw();
        true
    }
}

impl Widget for HierarchyPanel {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        if let Event::InternalDrag(drag) = event {
            // Only handle internal drags that carry a Project payload. Unconditionally stopping
            // propagation breaks cross-window DockPanel drags (DockSpace must still observe them).
            let is_project_drag = cx
                .app
                .drag()
                .and_then(|d| d.payload::<ProjectDragPayload>())
                .is_some();
            if !is_project_drag {
                return;
            }
            match drag.kind {
                fret_core::InternalDragKind::Enter | fret_core::InternalDragKind::Over => {
                    let next = self.asset_drop_preview_at(cx.app, drag.position);
                    if self.asset_drop != next {
                        self.asset_drop = next;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                }
                fret_core::InternalDragKind::Leave | fret_core::InternalDragKind::Cancel => {
                    if self.asset_drop.take().is_some() {
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                }
                fret_core::InternalDragKind::Drop => {
                    if let Some(preview) = self.asset_drop_preview_at(cx.app, drag.position) {
                        self.commit_asset_drop(cx, preview);
                    }
                    self.asset_drop = None;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
            }
            cx.stop_propagation();
            return;
        }

        match event {
            Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button: fret_core::MouseButton::Left,
                ..
            }) => {
                self.tree.event(cx, event);
                self.sync_selection_model(cx);
                if cx.stop_propagation {
                    return;
                }
                self.begin_drag_candidate(*position);
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
                                cx.capture_pointer(cx.node);
                            }
                        }

                        self.drag = Some(drag);
                        self.update_drag_preview(cx.app, *position);

                        if self.drag.as_ref().is_some_and(|d| d.dragging) {
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                            return;
                        }
                    }
                }
                self.tree.event(cx, event);
                self.sync_selection_model(cx);
            }
            Event::Pointer(fret_core::PointerEvent::Up {
                button: fret_core::MouseButton::Left,
                ..
            }) => {
                if self.drag.is_some() {
                    let _ = self.commit_drag(cx);
                    cx.release_pointer_capture();
                    cx.stop_propagation();
                    return;
                }
                self.tree.event(cx, event);
                self.sync_selection_model(cx);
            }
            _ => {
                self.tree.event(cx, event);
                self.sync_selection_model(cx);
            }
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        cx.observe_model(self.selection, Invalidation::Layout);
        cx.observe_model(self.hierarchy, Invalidation::Layout);

        // Ensure selection changes originating outside the hierarchy (viewport tools) are reflected.
        self.maybe_sync_from_model(cx.app);
        self.tree.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        cx.observe_model(self.selection, Invalidation::Paint);
        cx.observe_model(self.hierarchy, Invalidation::Paint);

        // Selection changes may request only a redraw (paint), so sync here as well.
        self.maybe_sync_from_model(cx.app);
        self.tree.paint(cx);

        if let Some(preview) = self.asset_drop {
            let theme = cx.theme().snapshot();
            let stroke = Color {
                a: 0.9,
                ..theme.colors.accent
            };
            let t = Px(2.0);

            match preview.kind {
                AssetDropPreviewKind::HighlightRow => {
                    if let Some(parent) = preview.parent {
                        if let Some(rect) = self.tree.row_rect(parent) {
                            cx.scene.push(fret_core::SceneOp::Quad {
                                order: fret_core::DrawOrder(49),
                                rect,
                                background: Color::TRANSPARENT,
                                border: Edges::all(t),
                                border_color: stroke,
                                corner_radii: Corners::all(Px(0.0)),
                            });
                        }
                    }
                }
                AssetDropPreviewKind::AppendRoot => {
                    let y = if let Some(last) = self.tree.last_row_rect() {
                        Px(last.origin.y.0 + last.size.height.0 - t.0)
                    } else {
                        let content = self.tree.content_bounds();
                        Px(content.origin.y.0 + content.size.height.0 - t.0)
                    };
                    let content = self.tree.content_bounds();
                    let line = fret_core::Rect::new(
                        fret_core::Point::new(content.origin.x, y),
                        fret_core::Size::new(content.size.width, t),
                    );
                    cx.scene.push(fret_core::SceneOp::Quad {
                        order: fret_core::DrawOrder(49),
                        rect: line,
                        background: stroke,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(Px(0.0)),
                    });
                }
            }
        }

        let Some(drag) = self.drag.as_ref() else {
            return;
        };
        if !drag.dragging {
            return;
        }
        let Some(preview) = drag.preview else {
            return;
        };

        let stroke = Color {
            r: 0.20,
            g: 0.75,
            b: 1.0,
            a: 0.9,
        };
        let t = Px(2.0);

        match preview.kind {
            HierarchyDropPreviewKind::HighlightRow => {
                let Some(target) = preview.target.target_id else {
                    return;
                };
                let Some(rect) = self.tree.row_rect(target) else {
                    return;
                };
                cx.scene.push(fret_core::SceneOp::Quad {
                    order: fret_core::DrawOrder(50),
                    rect,
                    background: Color::TRANSPARENT,
                    border: Edges::all(t),
                    border_color: stroke,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
            HierarchyDropPreviewKind::InsertLineAbove
            | HierarchyDropPreviewKind::InsertLineBelow => {
                let y = if let Some(target) = preview.target.target_id {
                    let Some(rect) = self.tree.row_rect(target) else {
                        return;
                    };
                    if preview.kind == HierarchyDropPreviewKind::InsertLineAbove {
                        rect.origin.y
                    } else {
                        Px(rect.origin.y.0 + rect.size.height.0 - t.0)
                    }
                } else {
                    let Some(last) = self.tree.last_row_rect() else {
                        return;
                    };
                    Px(last.origin.y.0 + last.size.height.0 - t.0)
                };

                let content = self.tree.content_bounds();
                let line = fret_core::Rect::new(
                    fret_core::Point::new(content.origin.x, y),
                    fret_core::Size::new(content.size.width, t),
                );
                cx.scene.push(fret_core::SceneOp::Quad {
                    order: fret_core::DrawOrder(50),
                    rect: line,
                    background: stroke,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        }
    }
}

pub struct InspectorPanel {
    selection: Model<DemoSelection>,
    world: Model<DemoWorld>,
    last_revision: Option<u64>,
    last_world_revision: Option<u64>,
    last_project_revision: Option<u64>,
    last_project_tree_revision: Option<u64>,
    last_scene_revision: Option<u64>,
    last_selected: Option<u64>,
    data: InspectorDataSource,
    prepared: Vec<PreparedInspectorRow>,
    toggle_glyph: Option<ToggleGlyph>,
    last_theme_revision: Option<u64>,
    last_scale_factor: Option<f32>,
    text_style: TextStyle,
    hover_row: Option<usize>,
    scroll_y: Px,
    content_h: Px,
    last_bounds: Rect,
    label_col_w: Px,
    scrub: Option<ScrubState>,
}

impl InspectorPanel {
    pub fn new(selection: Model<DemoSelection>, world: Model<DemoWorld>) -> Self {
        Self {
            selection,
            world,
            last_revision: None,
            last_world_revision: None,
            last_project_revision: None,
            last_project_tree_revision: None,
            last_scene_revision: None,
            last_selected: None,
            data: InspectorDataSource::empty(),
            prepared: Vec::new(),
            toggle_glyph: None,
            last_theme_revision: None,
            last_scale_factor: None,
            text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
            },
            hover_row: None,
            scroll_y: Px(0.0),
            content_h: Px(0.0),
            last_bounds: Rect::default(),
            label_col_w: Px(180.0),
            scrub: None,
        }
    }

    fn maybe_refresh(&mut self, app: &App, text: &mut dyn fret_core::TextService) -> bool {
        let revision = self.selection.revision(app);
        let world_revision = self.world.revision(app);
        let project_revision = app
            .global::<ProjectSelectionService>()
            .map(|s| s.revision())
            .unwrap_or(0);
        let project_tree_revision = app.global::<ProjectService>().map(|p| p.revision());
        let scene_revision = app
            .global::<CurrentSceneService>()
            .map(|s| s.revision())
            .unwrap_or(0);

        if revision == self.last_revision
            && world_revision == self.last_world_revision
            && self.last_project_revision == Some(project_revision)
            && self.last_project_tree_revision == project_tree_revision
            && self.last_scene_revision == Some(scene_revision)
        {
            return false;
        }

        let lead = self.selection.get(app).and_then(|s| s.lead_entity);
        let selected = self
            .selection
            .get(app)
            .map(|s| s.selected_entities.clone())
            .unwrap_or_default();

        self.last_selected = lead;
        self.last_revision = revision;
        self.last_world_revision = world_revision;
        self.last_project_revision = Some(project_revision);
        self.last_project_tree_revision = project_tree_revision;
        self.last_scene_revision = Some(scene_revision);

        self.release_prepared(text);
        self.data = InspectorDataSource::new(app, self.world, selected);
        self.hover_row = None;
        true
    }

    fn row_height() -> Px {
        Px(22.0)
    }

    fn padding(theme: ThemeSnapshot) -> (Px, Px) {
        (theme.metrics.padding_md, theme.metrics.padding_sm)
    }

    fn clamp_scroll(&mut self, viewport_h: Px) {
        let max_scroll = Px((self.content_h.0 - viewport_h.0).max(0.0));
        self.scroll_y = Px(self.scroll_y.0.clamp(0.0, max_scroll.0));
    }

    fn release_prepared(&mut self, text: &mut dyn fret_core::TextService) {
        if let Some(glyph) = self.toggle_glyph.take() {
            text.release(glyph.blob);
        }
        for row in self.prepared.drain(..) {
            match row.kind {
                PreparedInspectorRowKind::Header { label } => {
                    text.release(label.blob);
                }
                PreparedInspectorRowKind::Property { label, value, .. } => {
                    text.release(label.blob);
                    match value {
                        PreparedValue::Text(v) => text.release(v.blob),
                        PreparedValue::Vec3(v) => {
                            for comp in v {
                                text.release(comp.blob);
                            }
                        }
                    }
                }
            }
        }
    }

    fn ensure_prepared(&mut self, cx: &mut LayoutCx<'_>) {
        let theme = cx.theme().snapshot();
        let needs_rebuild = self.prepared.is_empty()
            || self.last_theme_revision != Some(theme.revision)
            || self.last_scale_factor != Some(cx.scale_factor);
        if !needs_rebuild {
            return;
        }

        self.release_prepared(cx.text);
        self.last_theme_revision = Some(theme.revision);
        self.last_scale_factor = Some(cx.scale_factor);

        let label_constraints = TextConstraints {
            max_width: Some(Px(360.0)),
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };
        let value_constraints = TextConstraints {
            max_width: Some(Px(640.0)),
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };

        for row in &self.data.rows {
            match row {
                InspectorRow::Header { label } => {
                    let (blob, metrics) =
                        cx.text.prepare(label, self.text_style, label_constraints);
                    self.prepared.push(PreparedInspectorRow {
                        kind: PreparedInspectorRowKind::Header {
                            label: PreparedText { blob, metrics },
                        },
                        action: None,
                    });
                }
                InspectorRow::Property {
                    label,
                    value,
                    action,
                } => {
                    let (l_blob, l_metrics) =
                        cx.text.prepare(label, self.text_style, label_constraints);

                    let value_kind = match action {
                        Some(InspectorRowAction::EditValue { request }) => request.kind,
                        _ => InspectorEditKind::String,
                    };

                    let prepared_value = if value_kind == InspectorEditKind::Vec3 {
                        let parts: Vec<&str> = value
                            .split(|c| c == ',' || c == ' ')
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .collect();
                        let mut texts: [String; 3] =
                            ["—".to_string(), "—".to_string(), "—".to_string()];
                        if parts.len() == 3 {
                            if let (Ok(x), Ok(y), Ok(z)) = (
                                parts[0].parse::<f32>(),
                                parts[1].parse::<f32>(),
                                parts[2].parse::<f32>(),
                            ) {
                                texts = [format!("{x:.3}"), format!("{y:.3}"), format!("{z:.3}")];
                            } else {
                                texts = [
                                    parts[0].to_string(),
                                    parts[1].to_string(),
                                    parts[2].to_string(),
                                ];
                            }
                        }
                        let (xb, xm) =
                            cx.text
                                .prepare(texts[0].as_str(), self.text_style, value_constraints);
                        let (yb, ym) =
                            cx.text
                                .prepare(texts[1].as_str(), self.text_style, value_constraints);
                        let (zb, zm) =
                            cx.text
                                .prepare(texts[2].as_str(), self.text_style, value_constraints);
                        PreparedValue::Vec3([
                            PreparedText {
                                blob: xb,
                                metrics: xm,
                            },
                            PreparedText {
                                blob: yb,
                                metrics: ym,
                            },
                            PreparedText {
                                blob: zb,
                                metrics: zm,
                            },
                        ])
                    } else {
                        let (v_blob, v_metrics) =
                            cx.text.prepare(value, self.text_style, value_constraints);
                        PreparedValue::Text(PreparedText {
                            blob: v_blob,
                            metrics: v_metrics,
                        })
                    };

                    self.prepared.push(PreparedInspectorRow {
                        kind: PreparedInspectorRowKind::Property {
                            label: PreparedText {
                                blob: l_blob,
                                metrics: l_metrics,
                            },
                            value: prepared_value,
                            value_kind,
                        },
                        action: action.clone(),
                    });
                }
            }
        }

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };
        let (blob, metrics) = cx.text.prepare("✓", self.text_style, constraints);
        self.toggle_glyph = Some(ToggleGlyph { blob, metrics });
    }

    fn layout_columns(&mut self, theme: ThemeSnapshot, bounds: Rect) {
        let (pad_x, _pad_y) = Self::padding(theme);
        let width = if bounds.size.width.0.is_finite() {
            bounds.size.width.0.max(0.0)
        } else {
            0.0
        };
        let max_w = 260.0f32.min(width * 0.55).max(0.0);
        let min_w = 120.0f32.min(max_w);

        let mut desired = 0.0f32;
        for row in &self.prepared {
            let PreparedInspectorRowKind::Property { label, .. } = &row.kind else {
                continue;
            };
            desired = desired.max(label.metrics.size.width.0);
        }
        if max_w <= 0.0 {
            self.label_col_w = Px(0.0);
            return;
        }

        let desired = (desired + pad_x.0 * 2.0).clamp(min_w, max_w);
        self.label_col_w = Px(desired);
    }

    fn row_index_at(&self, theme: ThemeSnapshot, pos: Point) -> Option<usize> {
        if !self.last_bounds.contains(pos) {
            return None;
        }
        let (_pad_x, pad_y) = Self::padding(theme);
        let top = self.last_bounds.origin.y.0 + pad_y.0;
        let local_y = pos.y.0 + self.scroll_y.0 - top;
        if local_y < 0.0 {
            return None;
        }
        let index = (local_y / Self::row_height().0).floor() as isize;
        if index < 0 {
            return None;
        }
        let index = index as usize;
        (index < self.prepared.len()).then_some(index)
    }

    fn row_rect(&self, theme: ThemeSnapshot, index: usize) -> Rect {
        let (_pad_x, pad_y) = Self::padding(theme);
        let y = self.last_bounds.origin.y.0 + pad_y.0 - self.scroll_y.0
            + Self::row_height().0 * index as f32;
        Rect::new(
            Point::new(self.last_bounds.origin.x, Px(y)),
            Size::new(self.last_bounds.size.width, Self::row_height()),
        )
    }

    fn value_rect(&self, row_rect: Rect) -> Rect {
        Rect::new(
            Point::new(
                Px(row_rect.origin.x.0 + self.label_col_w.0),
                row_rect.origin.y,
            ),
            Size::new(
                Px((row_rect.size.width.0 - self.label_col_w.0).max(0.0)),
                row_rect.size.height,
            ),
        )
    }
}

impl Widget for InspectorPanel {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        let theme = cx.theme().snapshot();

        if let Some(scrub) = self.scrub.as_mut() {
            match event {
                Event::Pointer(fret_core::PointerEvent::Move {
                    position,
                    modifiers,
                    ..
                }) => {
                    let dx = position.x.0 - scrub.start_x.0;
                    let mut step = 0.01f32;
                    if modifiers.shift {
                        step = 0.001;
                    }
                    if modifiers.ctrl || modifiers.meta {
                        step = 0.05;
                    }

                    let next_value = match &mut scrub.value {
                        ScrubValue::F32 { base, current } => {
                            let next = *base + dx * step;
                            if (next - *current).abs() < 1.0e-6 {
                                cx.stop_propagation();
                                return;
                            }
                            *current = next;
                            PropertyValue::F32(next)
                        }
                        ScrubValue::Vec3 {
                            axis,
                            base,
                            current,
                        } => {
                            let next = base[*axis] + dx * step;
                            if (next - current[*axis]).abs() < 1.0e-6 {
                                cx.stop_propagation();
                                return;
                            }
                            current[*axis] = next;
                            PropertyValue::Vec3(*current)
                        }
                    };

                    cx.app
                        .with_global_mut(PropertyEditService::default, |s, _app| {
                            s.set(
                                scrub.window,
                                PropertyEditRequest {
                                    targets: scrub.targets.clone(),
                                    path: scrub.path.clone(),
                                    value: next_value,
                                    kind: PropertyEditKind::Update,
                                },
                            );
                        });
                    cx.dispatch_command(fret_app::CommandId::from("property_edit.commit"));
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
                Event::Pointer(fret_core::PointerEvent::Up { button, .. }) => {
                    if *button == MouseButton::Left {
                        let done = self.scrub.take().unwrap();
                        cx.release_pointer_capture();
                        cx.app
                            .with_global_mut(PropertyEditService::default, |s, _app| {
                                s.set(
                                    done.window,
                                    PropertyEditRequest {
                                        targets: done.targets,
                                        path: done.path,
                                        value: match done.value {
                                            ScrubValue::F32 { current, .. } => {
                                                PropertyValue::F32(current)
                                            }
                                            ScrubValue::Vec3 { current, .. } => {
                                                PropertyValue::Vec3(current)
                                            }
                                        },
                                        kind: PropertyEditKind::Commit,
                                    },
                                );
                            });
                        cx.dispatch_command(fret_app::CommandId::from("property_edit.commit"));
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
                }
                Event::KeyDown {
                    key: KeyCode::Escape,
                    ..
                } => {
                    let canceled = self.scrub.take().unwrap();
                    cx.release_pointer_capture();
                    cx.app
                        .with_global_mut(PropertyEditService::default, |s, _app| {
                            s.set(
                                canceled.window,
                                PropertyEditRequest {
                                    targets: canceled.targets,
                                    path: canceled.path,
                                    value: match canceled.value {
                                        ScrubValue::F32 { base, .. } => PropertyValue::F32(base),
                                        ScrubValue::Vec3 { base, .. } => PropertyValue::Vec3(base),
                                    },
                                    kind: PropertyEditKind::Cancel,
                                },
                            );
                        });
                    cx.dispatch_command(fret_app::CommandId::from("property_edit.commit"));
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
                _ => {}
            }
        }

        match event {
            Event::Pointer(fret_core::PointerEvent::Wheel { delta, .. }) => {
                if delta.y.0.abs() > 0.0 {
                    self.scroll_y = Px(self.scroll_y.0 - delta.y.0);
                    self.clamp_scroll(self.last_bounds.size.height);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }
            Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
                let hovered = self.row_index_at(theme, *position);
                if hovered != self.hover_row {
                    self.hover_row = hovered;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
            }
            _ => {}
        }

        if let Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Left,
            modifiers,
        }) = event
        {
            let Some(window) = cx.window else {
                return;
            };

            if self.prepared.is_empty() {
                return;
            };

            let Some(row_index) = self.row_index_at(theme, *position) else {
                return;
            };

            let row_rect = self.row_rect(theme, row_index);
            let value_rect = self.value_rect(row_rect);

            if let Some(action) = self.data.action_at(row_index) {
                match action {
                    InspectorRowAction::ToggleBool {
                        targets,
                        path,
                        current,
                    } => {
                        if !value_rect.contains(*position) {
                            return;
                        }
                        let next = current.map(|v| !v).unwrap_or(true);
                        cx.app
                            .with_global_mut(PropertyEditService::default, |s, _app| {
                                s.set(
                                    window,
                                    PropertyEditRequest {
                                        targets,
                                        path,
                                        value: PropertyValue::Bool(next),
                                        kind: PropertyEditKind::Commit,
                                    },
                                );
                            });
                        cx.dispatch_command(fret_app::CommandId::from("property_edit.commit"));
                        cx.stop_propagation();
                        return;
                    }
                    InspectorRowAction::EditValue { request } => {
                        if modifiers.alt
                            && (request.kind == InspectorEditKind::F32
                                || request.kind == InspectorEditKind::Vec3)
                            && value_rect.contains(*position)
                        {
                            let base = self
                                .world
                                .get(cx.app)
                                .and_then(|w| {
                                    request.targets.first().and_then(|&id| {
                                        w.get_property(id, &request.path).and_then(|v| match v {
                                            PropertyValue::F32(f) => Some(PropertyValue::F32(f)),
                                            PropertyValue::Vec3(v) => Some(PropertyValue::Vec3(v)),
                                            _ => None,
                                        })
                                    })
                                })
                                .unwrap_or_else(|| match request.kind {
                                    InspectorEditKind::Vec3 => PropertyValue::Vec3([0.0, 0.0, 0.0]),
                                    _ => PropertyValue::F32(0.0),
                                });

                            let scrub_value = match (request.kind, &base) {
                                (InspectorEditKind::Vec3, PropertyValue::Vec3(v)) => {
                                    let (pad_x, _pad_y) = Self::padding(theme);
                                    let inner_x0 = value_rect.origin.x.0 + pad_x.0;
                                    let inner_w =
                                        (value_rect.size.width.0 - pad_x.0 * 2.0).max(1.0);
                                    let rel_x = (position.x.0 - inner_x0)
                                        .clamp(0.0, (inner_w - 1.0).max(0.0));
                                    let seg_w = (inner_w / 3.0).max(1.0);
                                    let axis = (rel_x / seg_w).floor().clamp(0.0, 2.0) as usize;
                                    ScrubValue::Vec3 {
                                        axis,
                                        base: *v,
                                        current: *v,
                                    }
                                }
                                (_, PropertyValue::F32(f)) => ScrubValue::F32 {
                                    base: *f,
                                    current: *f,
                                },
                                _ => ScrubValue::F32 {
                                    base: 0.0,
                                    current: 0.0,
                                },
                            };

                            cx.request_focus(cx.node);
                            cx.capture_pointer(cx.node);

                            cx.app
                                .with_global_mut(PropertyEditService::default, |s, _app| {
                                    s.set(
                                        window,
                                        PropertyEditRequest {
                                            targets: request.targets.clone(),
                                            path: request.path.clone(),
                                            value: base.clone(),
                                            kind: PropertyEditKind::Begin,
                                        },
                                    );
                                });
                            cx.dispatch_command(fret_app::CommandId::from("property_edit.commit"));

                            self.scrub = Some(ScrubState {
                                window,
                                targets: request.targets,
                                path: request.path,
                                start_x: position.x,
                                value: scrub_value,
                            });
                            cx.stop_propagation();
                            return;
                        }

                        let mut request = request;
                        request.anchor = Some(value_rect);
                        cx.app
                            .with_global_mut(InspectorEditService::default, |service, _app| {
                                service.set_request(window, request);
                            });
                        cx.dispatch_command(fret_app::CommandId::from("inspector_edit.open"));
                        cx.stop_propagation();
                        return;
                    }
                }
            }
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        cx.observe_model(self.selection, Invalidation::Layout);
        cx.observe_model(self.world, Invalidation::Layout);

        let _ = self.maybe_refresh(cx.app, cx.text);
        self.last_bounds = cx.bounds;
        self.ensure_prepared(cx);

        let theme = cx.theme().snapshot();
        let (_pad_x, pad_y) = Self::padding(theme);
        self.content_h = Px(pad_y.0 * 2.0 + Self::row_height().0 * self.prepared.len() as f32);
        self.clamp_scroll(cx.bounds.size.height);
        self.layout_columns(theme, cx.bounds);

        Size::new(cx.available.width, cx.available.height)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        cx.observe_model(self.selection, Invalidation::Paint);
        cx.observe_model(self.world, Invalidation::Paint);

        let _ = self.maybe_refresh(cx.app, cx.text);
        let theme = cx.theme().snapshot();

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: theme.colors.panel_background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });

        if self.prepared.is_empty() {
            return;
        }

        let (pad_x, pad_y) = Self::padding(theme);
        let row_h = Self::row_height();
        let start_y = cx.bounds.origin.y.0 + pad_y.0 - self.scroll_y.0;

        for (i, row) in self.prepared.iter().enumerate() {
            let y = start_y + row_h.0 * i as f32;
            let rect = Rect::new(
                Point::new(cx.bounds.origin.x, Px(y)),
                Size::new(cx.bounds.size.width, row_h),
            );

            if rect.origin.y.0 + rect.size.height.0 < cx.bounds.origin.y.0
                || rect.origin.y.0 > cx.bounds.origin.y.0 + cx.bounds.size.height.0
            {
                continue;
            }

            let hovered = self.hover_row == Some(i);
            if hovered {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect,
                    background: theme.colors.hover_background,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            match &row.kind {
                PreparedInspectorRowKind::Header { label } => {
                    let origin = Point::new(
                        Px(rect.origin.x.0 + pad_x.0),
                        Px(rect.origin.y.0
                            + (rect.size.height.0 - label.metrics.size.height.0) * 0.5
                            + label.metrics.baseline.0),
                    );
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(2),
                        origin,
                        text: label.blob,
                        color: theme.colors.text_muted,
                    });
                }
                PreparedInspectorRowKind::Property {
                    label,
                    value,
                    value_kind,
                } => {
                    let label_origin = Point::new(
                        Px(rect.origin.x.0 + pad_x.0),
                        Px(rect.origin.y.0
                            + (rect.size.height.0 - label.metrics.size.height.0) * 0.5
                            + label.metrics.baseline.0),
                    );
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(2),
                        origin: label_origin,
                        text: label.blob,
                        color: theme.colors.text_primary,
                    });

                    let value_rect = self.value_rect(rect);
                    let inner = Rect::new(
                        Point::new(Px(value_rect.origin.x.0 + pad_x.0), value_rect.origin.y),
                        Size::new(
                            Px((value_rect.size.width.0 - pad_x.0 * 2.0).max(0.0)),
                            value_rect.size.height,
                        ),
                    );

                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(2),
                        rect: Rect::new(
                            Point::new(
                                Px(value_rect.origin.x.0 + pad_x.0 * 0.5),
                                Px(value_rect.origin.y.0 + 3.0),
                            ),
                            Size::new(
                                Px((value_rect.size.width.0 - pad_x.0).max(0.0)),
                                Px((value_rect.size.height.0 - 6.0).max(0.0)),
                            ),
                        ),
                        background: theme.colors.list_background,
                        border: Edges::all(Px(1.0)),
                        border_color: theme.colors.panel_border,
                        corner_radii: Corners::all(theme.metrics.radius_sm),
                    });

                    match value {
                        PreparedValue::Text(value) => {
                            let is_toggle =
                                matches!(row.action, Some(InspectorRowAction::ToggleBool { .. }));
                            let toggle_offset_x = if is_toggle { Px(20.0) } else { Px(0.0) };
                            let origin = Point::new(
                                Px(inner.origin.x.0 + toggle_offset_x.0),
                                Px(rect.origin.y.0
                                    + (rect.size.height.0 - value.metrics.size.height.0) * 0.5
                                    + value.metrics.baseline.0),
                            );
                            cx.scene.push(SceneOp::Text {
                                order: DrawOrder(3),
                                origin,
                                text: value.blob,
                                color: theme.colors.text_primary,
                            });
                        }
                        PreparedValue::Vec3(comps) => {
                            let seg_w = Px((inner.size.width.0 / 3.0).max(1.0));
                            for (axis, comp) in comps.iter().enumerate() {
                                let seg = Rect::new(
                                    Point::new(
                                        Px(inner.origin.x.0 + seg_w.0 * axis as f32),
                                        inner.origin.y,
                                    ),
                                    Size::new(seg_w, inner.size.height),
                                );
                                cx.scene.push(SceneOp::Quad {
                                    order: DrawOrder(3),
                                    rect: Rect::new(
                                        Point::new(
                                            Px(seg.origin.x.0 + 2.0),
                                            Px(seg.origin.y.0 + 4.0),
                                        ),
                                        Size::new(
                                            Px((seg.size.width.0 - 4.0).max(0.0)),
                                            Px((seg.size.height.0 - 8.0).max(0.0)),
                                        ),
                                    ),
                                    background: theme.colors.panel_background,
                                    border: Edges::all(Px(1.0)),
                                    border_color: theme.colors.panel_border,
                                    corner_radii: Corners::all(theme.metrics.radius_sm),
                                });

                                let x = seg.origin.x.0
                                    + (seg.size.width.0 - comp.metrics.size.width.0) * 0.5;
                                let y = rect.origin.y.0
                                    + (rect.size.height.0 - comp.metrics.size.height.0) * 0.5
                                    + comp.metrics.baseline.0;
                                cx.scene.push(SceneOp::Text {
                                    order: DrawOrder(4),
                                    origin: Point::new(Px(x), Px(y)),
                                    text: comp.blob,
                                    color: theme.colors.text_primary,
                                });
                            }
                        }
                    }

                    if matches!(row.action, Some(InspectorRowAction::ToggleBool { .. })) {
                        let box_size = Px(14.0);
                        let box_rect = Rect::new(
                            Point::new(
                                Px(value_rect.origin.x.0 + pad_x.0),
                                Px(rect.origin.y.0 + (rect.size.height.0 - box_size.0) * 0.5),
                            ),
                            Size::new(box_size, box_size),
                        );
                        cx.scene.push(SceneOp::Quad {
                            order: DrawOrder(5),
                            rect: box_rect,
                            background: theme.colors.panel_background,
                            border: Edges::all(Px(1.0)),
                            border_color: theme.colors.panel_border,
                            corner_radii: Corners::all(Px(3.0)),
                        });

                        if let Some(InspectorRowAction::ToggleBool { current, .. }) = &row.action {
                            if current.unwrap_or(false) {
                                if let Some(glyph) = self.toggle_glyph.as_ref() {
                                    let ox = box_rect.origin.x.0
                                        + (box_rect.size.width.0 - glyph.metrics.size.width.0)
                                            * 0.5;
                                    let oy = box_rect.origin.y.0
                                        + (box_rect.size.height.0 - glyph.metrics.size.height.0)
                                            * 0.5
                                        + glyph.metrics.baseline.0;
                                    cx.scene.push(SceneOp::Text {
                                        order: DrawOrder(6),
                                        origin: Point::new(Px(ox), Px(oy)),
                                        text: glyph.blob,
                                        color: theme.colors.accent,
                                    });
                                }
                            }
                        }
                    }

                    if *value_kind == InspectorEditKind::F32 {
                        // Intentionally left minimal: f32 is shown as a single field.
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod inspector_tests {
    use super::*;
    use fret_app::App;
    use fret_core::{TextBlobId, TextMetrics, TextService};

    #[derive(Default)]
    struct FakeTextService;

    impl TextService for FakeTextService {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    #[test]
    fn inspector_layout_does_not_panic_with_zero_width() {
        let mut app = App::new();
        let selection = app.models_mut().insert(DemoSelection::default());
        let world = app.models_mut().insert(DemoWorld::default());

        let mut ui = fret_ui::UiTree::new();
        ui.set_window(AppWindowId::default());

        let inspector = ui.create_node(InspectorPanel::new(selection, world));
        ui.set_root(inspector);

        let mut text = FakeTextService::default();
        let _ = ui.layout(
            &mut app,
            &mut text,
            inspector,
            Size::new(Px(0.0), Px(100.0)),
            1.0,
        );
    }
}
