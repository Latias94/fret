use crate::hierarchy::{DemoHierarchy, HierarchyDropKind, HierarchyDropTarget};
use crate::inspector_edit::{InspectorEditKind, InspectorEditRequest, InspectorEditService};
use crate::inspector_protocol::{
    InspectorEditorKind, InspectorEditorRegistry, PropertyLeaf, PropertyMeta, PropertyNode,
    PropertyTree, PropertyTypeTag,
};
use crate::property::PropertyPath;
use crate::property_edit::{PropertyEditKind, PropertyEditRequest, PropertyEditService};
use crate::world::DemoWorld;
use fret_app::{App, Model};
use fret_core::{Color, Corners, Edges, Event, Px, Size, TextStyle};
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, TreeView, VirtualList, Widget};
use std::borrow::Cow;

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

        fn mixed_value(
            world: &DemoWorld,
            targets: &[u64],
            path: &PropertyPath,
        ) -> crate::property::PropertyValue {
            let Some(first) = targets.first().and_then(|id| world.get_property(*id, path)) else {
                return crate::property::PropertyValue::Mixed;
            };
            for &id in targets.iter().skip(1) {
                if world.get_property(id, path) != Some(first.clone()) {
                    return crate::property::PropertyValue::Mixed;
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
                            crate::property::PropertyValue::Bool(v) => {
                                Some(InspectorRowAction::ToggleBool {
                                    targets: targets.to_vec(),
                                    path: leaf.path.clone(),
                                    current: Some(*v),
                                })
                            }
                            crate::property::PropertyValue::Mixed => {
                                Some(InspectorRowAction::ToggleBool {
                                    targets: targets.to_vec(),
                                    path: leaf.path.clone(),
                                    current: None,
                                })
                            }
                            _ => None,
                        },
                        InspectorEditorKind::TextPopup => Some(InspectorRowAction::EditValue {
                            request: InspectorEditRequest {
                                targets: targets.to_vec(),
                                path: leaf.path.clone(),
                                kind: InspectorEditKind::String,
                                initial_text: match &leaf.value {
                                    crate::property::PropertyValue::String(v) => v.clone(),
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
                                        crate::property::PropertyValue::F32(v) => match kind {
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
                                    crate::property::PropertyValue::Vec3([x, y, z]) => {
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

    fn row_text(&self, index: usize) -> String {
        match &self.rows[index] {
            InspectorRow::Header { label } => label.clone(),
            InspectorRow::Property { label, value, .. } => format!("{label}: {value}"),
        }
    }

    fn action_at(&self, index: usize) -> Option<InspectorRowAction> {
        match self.rows.get(index)? {
            InspectorRow::Property { action, .. } => action.clone(),
            _ => None,
        }
    }
}

impl fret_ui::VirtualListDataSource for InspectorDataSource {
    type Key = usize;

    fn len(&self) -> usize {
        self.rows.len()
    }

    fn key_at(&self, index: usize) -> Self::Key {
        index
    }

    fn row_at(&self, index: usize) -> fret_ui::VirtualListRow<'_> {
        // Allocates on demand; row count is small for the inspector MVP.
        fret_ui::VirtualListRow::new(Cow::Owned(self.row_text(index)))
    }
}

pub struct HierarchyPanel {
    tree: TreeView,
    selection: Model<DemoSelection>,
    hierarchy: Model<DemoHierarchy>,
    drag: Option<HierarchyDragState>,
    last_selected: Option<u64>,
    last_selected_keys: Vec<u64>,
    last_revision: Option<u64>,
    last_hierarchy_revision: Option<u64>,
    did_init_expanded: bool,
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
    pub fn new(selection: Model<DemoSelection>, hierarchy: Model<DemoHierarchy>) -> Self {
        Self {
            tree: TreeView::new(Vec::new()),
            selection,
            hierarchy,
            drag: None,
            last_selected: None,
            last_selected_keys: Vec::new(),
            last_revision: None,
            last_hierarchy_revision: None,
            did_init_expanded: false,
        }
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

        let _ = self.hierarchy.update(cx.app, |h, _cx| {
            let _ = h.apply_move(op);
        });
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
        // Ensure selection changes originating outside the hierarchy (viewport tools) are reflected.
        self.maybe_sync_from_model(cx.app);
        self.tree.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        // Selection changes may request only a redraw (paint), so sync here as well.
        self.maybe_sync_from_model(cx.app);
        self.tree.paint(cx);

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
    last_selected: Option<u64>,
    list: VirtualList<InspectorDataSource>,
}

impl InspectorPanel {
    pub fn new(selection: Model<DemoSelection>, world: Model<DemoWorld>) -> Self {
        let mut list = VirtualList::new(InspectorDataSource::empty())
            .with_row_height(Px(22.0))
            .with_style(fret_ui::VirtualListStyle {
                padding_x: Px(10.0),
                background: Color {
                    r: 0.10,
                    g: 0.10,
                    b: 0.12,
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
                row_hover: Color {
                    r: 0.16,
                    g: 0.17,
                    b: 0.22,
                    a: 0.95,
                },
                row_selected: Color {
                    r: 0.24,
                    g: 0.34,
                    b: 0.52,
                    a: 0.65,
                },
                text_color: Color {
                    r: 0.92,
                    g: 0.92,
                    b: 0.92,
                    a: 1.0,
                },
                text_style: TextStyle {
                    font: fret_core::FontId::default(),
                    size: Px(13.0),
                },
            });
        list.clear_selection();

        Self {
            selection,
            world,
            last_revision: None,
            last_world_revision: None,
            last_selected: None,
            list,
        }
    }

    fn maybe_refresh(&mut self, app: &App) -> bool {
        let revision = self.selection.revision(app);
        let world_revision = self.world.revision(app);

        if revision == self.last_revision && world_revision == self.last_world_revision {
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

        self.list
            .set_data(InspectorDataSource::new(app, self.world, selected));
        true
    }
}

impl Widget for InspectorPanel {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        if let Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Left,
            modifiers,
        }) = event
        {
            if modifiers.ctrl || modifiers.meta || modifiers.shift || modifiers.alt {
                self.list.event(cx, event);
                return;
            }

            let Some(window) = cx.window else {
                self.list.event(cx, event);
                return;
            };

            let Some(row_index) = self.list.row_index_at(*position) else {
                self.list.event(cx, event);
                return;
            };
            if let Some(action) = self.list.data().action_at(row_index) {
                match action {
                    InspectorRowAction::ToggleBool {
                        targets,
                        path,
                        current,
                    } => {
                        let next = current.map(|v| !v).unwrap_or(true);
                        cx.app
                            .with_global_mut(PropertyEditService::default, |s, _app| {
                                s.set(
                                    window,
                                    PropertyEditRequest {
                                        targets,
                                        path,
                                        value: crate::property::PropertyValue::Bool(next),
                                        kind: PropertyEditKind::Commit,
                                    },
                                );
                            });
                        cx.dispatch_command(fret_app::CommandId::from("property_edit.commit"));
                        cx.stop_propagation();
                        return;
                    }
                    InspectorRowAction::EditValue { request } => {
                        let mut request = request;
                        request.anchor = self.list.row_rect(row_index);
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
        self.list.event(cx, event);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        let _ = self.maybe_refresh(cx.app);
        self.list.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let _ = self.maybe_refresh(cx.app);
        self.list.paint(cx);
    }
}
