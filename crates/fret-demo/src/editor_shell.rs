use crate::inspector_edit::{InspectorEditKind, InspectorEditRequest, InspectorEditService};
use crate::property::PropertyPath;
use crate::world::DemoWorld;
use fret_app::{App, Model};
use fret_core::{Color, Corners, Edges, Event, Px, Size, TextStyle};
use fret_ui::{EventCx, LayoutCx, PaintCx, TreeNode, TreeView, VirtualList, Widget};
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

        #[derive(Default)]
        struct InspectorEditorRegistry {
            vec3_overrides: Vec<PropertyPath>,
        }

        impl InspectorEditorRegistry {
            fn register_vec3(&mut self, path: PropertyPath) {
                self.vec3_overrides.push(path);
            }

            fn is_vec3(&self, path: &PropertyPath) -> bool {
                self.vec3_overrides.iter().any(|p| p == path)
            }
        }

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
        registry.register_vec3(path_pos.clone());

        rows.push(InspectorRow::Header {
            label: format!("GameObject ({})", targets.len()),
        });

        let name_val = mixed_value(world, &targets, &path_name);
        rows.push(InspectorRow::Property {
            label: "Name".to_string(),
            value: name_val.as_display_string(),
            action: Some(InspectorRowAction::EditValue {
                request: InspectorEditRequest {
                    targets: targets.clone(),
                    path: path_name.clone(),
                    kind: InspectorEditKind::String,
                    initial_text: match name_val {
                        crate::property::PropertyValue::String(v) => v,
                        _ => String::new(),
                    },
                },
            }),
        });

        let active_val = mixed_value(world, &targets, &path_active);
        let active_action = match active_val {
            crate::property::PropertyValue::Bool(_) => Some(InspectorRowAction::ToggleBool {
                targets: targets.clone(),
                path: path_active.clone(),
            }),
            _ => None,
        };
        rows.push(InspectorRow::Property {
            label: "Active".to_string(),
            value: active_val.as_display_string(),
            action: active_action,
        });

        rows.push(InspectorRow::Header {
            label: "Transform".to_string(),
        });

        let pos_val = mixed_value(world, &targets, &path_pos);
        let pos_kind = if registry.is_vec3(&path_pos) {
            InspectorEditKind::Vec3
        } else {
            InspectorEditKind::F32
        };
        rows.push(InspectorRow::Property {
            label: "Position".to_string(),
            value: pos_val.as_display_string(),
            action: Some(InspectorRowAction::EditValue {
                request: InspectorEditRequest {
                    targets: targets.clone(),
                    path: path_pos.clone(),
                    kind: pos_kind,
                    initial_text: match pos_val {
                        crate::property::PropertyValue::Vec3([x, y, z]) => {
                            format!("{x:.3}, {y:.3}, {z:.3}")
                        }
                        _ => String::new(),
                    },
                },
            }),
        });

        let rot_val = mixed_value(world, &targets, &path_rot);
        rows.push(InspectorRow::Property {
            label: "Rotation Y".to_string(),
            value: rot_val.as_display_string(),
            action: Some(InspectorRowAction::EditValue {
                request: InspectorEditRequest {
                    targets: targets.clone(),
                    path: path_rot.clone(),
                    kind: InspectorEditKind::F32,
                    initial_text: match rot_val {
                        crate::property::PropertyValue::F32(v) => format!("{v:.3}"),
                        _ => String::new(),
                    },
                },
            }),
        });

        let scale_val = mixed_value(world, &targets, &path_scale);
        rows.push(InspectorRow::Property {
            label: "Scale".to_string(),
            value: scale_val.as_display_string(),
            action: Some(InspectorRowAction::EditValue {
                request: InspectorEditRequest {
                    targets,
                    path: path_scale,
                    kind: InspectorEditKind::F32,
                    initial_text: match scale_val {
                        crate::property::PropertyValue::F32(v) => format!("{v:.3}"),
                        _ => String::new(),
                    },
                },
            }),
        });

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
    last_selected: Option<u64>,
    last_selected_keys: Vec<u64>,
}

impl HierarchyPanel {
    pub fn new(selection: Model<DemoSelection>, roots: Vec<TreeNode>, expanded: Vec<u64>) -> Self {
        Self {
            tree: TreeView::new(roots).with_expanded(expanded),
            selection,
            last_selected: None,
            last_selected_keys: Vec::new(),
        }
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

        cx.request_redraw();
    }
}

impl Widget for HierarchyPanel {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        self.tree.event(cx, event);
        self.sync_selection_model(cx);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        self.tree.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.tree.paint(cx);
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
                    InspectorRowAction::ToggleBool { targets, path } => {
                        let _ = self.world.update(cx.app, |w, _cx| {
                            for id in targets {
                                let cur = w.get_property(id, &path);
                                if let Some(crate::property::PropertyValue::Bool(v)) = cur {
                                    let _ = w.set_property(
                                        id,
                                        &path,
                                        crate::property::PropertyValue::Bool(!v),
                                    );
                                }
                            }
                        });
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
                    InspectorRowAction::EditValue { request } => {
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
