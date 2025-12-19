use fret_app::{App, Model};
use fret_core::{Color, Corners, Edges, Event, Px, Size, TextStyle};
use fret_ui::{EventCx, LayoutCx, PaintCx, TreeNode, TreeView, VirtualList, Widget};
use std::borrow::Cow;

#[derive(Debug, Default, Clone)]
pub struct DemoSelection {
    pub selected_entity: Option<u64>,
}

#[derive(Debug, Clone)]
struct InspectorRow {
    label: String,
}

#[derive(Debug, Clone)]
struct InspectorDataSource {
    rows: Vec<InspectorRow>,
}

impl InspectorDataSource {
    fn new(selected: Option<u64>) -> Self {
        let mut rows: Vec<InspectorRow> = Vec::new();
        match selected {
            Some(id) => {
                rows.push(InspectorRow {
                    label: format!("Selected Entity: {id}"),
                });
                rows.push(InspectorRow {
                    label: format!("Name: Entity {id:06}"),
                });
                rows.push(InspectorRow {
                    label: format!("Active: {}", if id % 3 != 0 { "true" } else { "false" }),
                });
                rows.push(InspectorRow {
                    label: format!(
                        "Transform.Position: ({:.2}, {:.2}, {:.2})",
                        (id % 97) as f32 * 0.1,
                        (id % 53) as f32 * 0.1,
                        (id % 31) as f32 * 0.1
                    ),
                });
                rows.push(InspectorRow {
                    label: format!("Transform.RotationY: {:.1}°", (id % 360) as f32),
                });
                rows.push(InspectorRow {
                    label: format!("Transform.Scale: {:.2}", 0.5 + (id % 100) as f32 * 0.01),
                });
                rows.push(InspectorRow {
                    label: "Components:".to_string(),
                });
                for i in 0..12u64 {
                    rows.push(InspectorRow {
                        label: format!("  - Component {i} (hash={:x})", (id ^ (i * 31)) as u64),
                    });
                }
            }
            None => {
                rows.push(InspectorRow {
                    label: "No selection (click an item in Hierarchy)".to_string(),
                });
            }
        }

        Self { rows }
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
        fret_ui::VirtualListRow::new(Cow::Borrowed(self.rows[index].label.as_str()))
    }
}

pub struct HierarchyPanel {
    tree: TreeView,
    selection: Model<DemoSelection>,
    last_selected: Option<u64>,
}

impl HierarchyPanel {
    pub fn new(selection: Model<DemoSelection>, roots: Vec<TreeNode>, expanded: Vec<u64>) -> Self {
        Self {
            tree: TreeView::new(roots).with_expanded(expanded),
            selection,
            last_selected: None,
        }
    }

    fn sync_selection_model(&mut self, cx: &mut EventCx<'_>) {
        let next = self.tree.selected();
        if next == self.last_selected {
            return;
        }
        self.last_selected = next;

        let _ = self.selection.update(cx.app, |state, _cx| {
            state.selected_entity = next;
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
    last_revision: Option<u64>,
    last_selected: Option<u64>,
    list: VirtualList<InspectorDataSource>,
}

impl InspectorPanel {
    pub fn new(selection: Model<DemoSelection>) -> Self {
        let mut list = VirtualList::new(InspectorDataSource::new(None))
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
            last_revision: None,
            last_selected: None,
            list,
        }
    }

    fn maybe_refresh(&mut self, app: &App) -> bool {
        let revision = self.selection.revision(app);
        if revision == self.last_revision {
            return false;
        }
        self.last_revision = revision;

        let selected = self.selection.get(app).and_then(|s| s.selected_entity);
        if selected == self.last_selected {
            return true;
        }
        self.last_selected = selected;
        self.list.set_data(InspectorDataSource::new(selected));
        true
    }
}

impl Widget for InspectorPanel {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
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
