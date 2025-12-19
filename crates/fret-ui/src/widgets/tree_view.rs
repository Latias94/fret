use crate::widget::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};
use fret_core::{Event, KeyCode, Modifiers, MouseButton, Px};
use std::collections::{HashMap, HashSet};

use super::virtual_list::{VirtualList, VirtualListRow, VirtualListStyle};

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: u64,
    pub label: String,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(id: u64, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<TreeNode>) -> Self {
        self.children = children;
        self
    }
}

#[derive(Debug, Clone)]
pub struct TreeViewStyle {
    pub list: VirtualListStyle,
    pub indent_width: Px,
    pub disclosure_width: Px,
}

impl Default for TreeViewStyle {
    fn default() -> Self {
        Self {
            list: VirtualListStyle::default(),
            indent_width: Px(16.0),
            disclosure_width: Px(14.0),
        }
    }
}

#[derive(Debug, Clone)]
struct FlatRow {
    id: u64,
    depth: usize,
    label: String,
    has_children: bool,
    expanded: bool,
}

#[derive(Debug)]
pub struct TreeView {
    roots: Vec<TreeNode>,
    expanded: HashSet<u64>,
    selected: Option<u64>,

    parent_by_id: HashMap<u64, Option<u64>>,
    first_child_by_id: HashMap<u64, u64>,

    flat: Vec<FlatRow>,
    id_to_index: HashMap<u64, usize>,

    list: VirtualList,
    style: TreeViewStyle,
    dirty: bool,
}

impl TreeView {
    pub fn new(roots: Vec<TreeNode>) -> Self {
        let style = TreeViewStyle::default();
        let mut view = Self {
            roots,
            expanded: HashSet::new(),
            selected: None,
            parent_by_id: HashMap::new(),
            first_child_by_id: HashMap::new(),
            flat: Vec::new(),
            id_to_index: HashMap::new(),
            list: VirtualList::new(Vec::new()).with_style(style.list.clone()),
            style,
            dirty: true,
        };
        view.rebuild();
        view
    }

    pub fn with_expanded(mut self, ids: impl IntoIterator<Item = u64>) -> Self {
        for id in ids {
            self.expanded.insert(id);
        }
        self.dirty = true;
        self.rebuild();
        self
    }

    pub fn selected(&self) -> Option<u64> {
        self.selected
    }

    fn rebuild(&mut self) {
        self.parent_by_id.clear();
        self.first_child_by_id.clear();
        self.build_index_maps();

        let mut flat: Vec<FlatRow> = Vec::new();
        for root in &self.roots {
            push_flat(&mut flat, &self.expanded, root, 0);
        }
        self.flat = flat;

        self.id_to_index.clear();
        for (i, row) in self.flat.iter().enumerate() {
            self.id_to_index.insert(row.id, i);
        }

        let rows: Vec<VirtualListRow> = self
            .flat
            .iter()
            .map(|row| {
                let glyph = if row.has_children {
                    if row.expanded { "▾ " } else { "▸ " }
                } else {
                    "  "
                };
                let text = format!("{glyph}{}", row.label);
                VirtualListRow::new(text)
                    .with_indent_x(Px(row.depth as f32 * self.style.indent_width.0))
            })
            .collect();
        self.list.set_rows(rows);

        self.sync_list_selection_from_selected();
        self.dirty = false;
    }

    fn build_index_maps(&mut self) {
        fn visit(
            out_parent: &mut HashMap<u64, Option<u64>>,
            out_first_child: &mut HashMap<u64, u64>,
            node: &TreeNode,
            parent: Option<u64>,
        ) {
            out_parent.insert(node.id, parent);
            if let Some(first) = node.children.first() {
                out_first_child.insert(node.id, first.id);
            }
            for child in &node.children {
                visit(out_parent, out_first_child, child, Some(node.id));
            }
        }

        for root in &self.roots {
            visit(
                &mut self.parent_by_id,
                &mut self.first_child_by_id,
                root,
                None,
            );
        }
    }

    fn sync_selected_from_list(&mut self) {
        let Some(index) = self.list.selected() else {
            return;
        };
        let Some(row) = self.flat.get(index) else {
            return;
        };
        self.selected = Some(row.id);
    }

    fn sync_list_selection_from_selected(&mut self) {
        let Some(mut id) = self.selected else {
            self.list.set_selected(None);
            return;
        };

        loop {
            if let Some(&index) = self.id_to_index.get(&id) {
                self.list.set_selected(Some(index));
                self.list.ensure_visible(index);
                self.selected = Some(id);
                return;
            }

            let parent = self.parent_by_id.get(&id).copied().flatten();
            let Some(next) = parent else {
                self.selected = None;
                self.list.set_selected(None);
                return;
            };
            id = next;
        }
    }

    fn toggle_expanded(&mut self, id: u64) -> bool {
        if !self.first_child_by_id.contains_key(&id) {
            return false;
        }
        if self.expanded.contains(&id) {
            self.expanded.remove(&id);
        } else {
            self.expanded.insert(id);
        }
        true
    }

    fn maybe_handle_disclosure_click(&mut self, cx: &mut EventCx<'_>, position: fret_core::Point) {
        if self.dirty {
            self.rebuild();
        }

        let Some(index) = self.list.row_index_at(position) else {
            return;
        };
        let Some(row) = self.flat.get(index) else {
            return;
        };
        if !row.has_children {
            return;
        }

        let content = self.list.content_bounds();
        let local_x = Px(position.x.0 - content.origin.x.0);
        let indent_x = Px(row.depth as f32 * self.style.indent_width.0);
        let disclosure_left = Px(self.list.style().padding_x.0 + indent_x.0);
        let disclosure_right = Px(disclosure_left.0 + self.style.disclosure_width.0);
        if local_x.0 < disclosure_left.0 || local_x.0 > disclosure_right.0 {
            return;
        }

        cx.request_focus(cx.node);
        self.selected = Some(row.id);
        if self.toggle_expanded(row.id) {
            self.dirty = true;
            self.rebuild();
            cx.invalidate_self(Invalidation::Layout);
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
        }
    }

    fn maybe_handle_tree_keys(&mut self, cx: &mut EventCx<'_>, key: KeyCode, modifiers: Modifiers) {
        if modifiers.ctrl || modifiers.meta || modifiers.alt {
            return;
        }
        if cx.focus != Some(cx.node) {
            return;
        }
        if self.dirty {
            self.rebuild();
        }

        let Some(selected_id) = self.selected.or_else(|| self.flat.first().map(|r| r.id)) else {
            return;
        };

        match key {
            KeyCode::ArrowRight => {
                if self.first_child_by_id.contains_key(&selected_id)
                    && !self.expanded.contains(&selected_id)
                {
                    self.selected = Some(selected_id);
                    self.expanded.insert(selected_id);
                    self.dirty = true;
                    self.rebuild();
                } else if let Some(&child_id) = self.first_child_by_id.get(&selected_id) {
                    self.selected = Some(child_id);
                    self.sync_list_selection_from_selected();
                } else {
                    return;
                }
            }
            KeyCode::ArrowLeft => {
                if self.expanded.contains(&selected_id)
                    && self.first_child_by_id.contains_key(&selected_id)
                {
                    self.selected = Some(selected_id);
                    self.expanded.remove(&selected_id);
                    self.dirty = true;
                    self.rebuild();
                } else if let Some(parent_id) =
                    self.parent_by_id.get(&selected_id).copied().flatten()
                {
                    self.selected = Some(parent_id);
                    self.sync_list_selection_from_selected();
                } else {
                    return;
                }
            }
            _ => return,
        }

        cx.invalidate_self(Invalidation::Layout);
        cx.invalidate_self(Invalidation::Paint);
        cx.request_redraw();
        cx.stop_propagation();
    }
}

fn push_flat(
    out: &mut Vec<FlatRow>,
    expanded: &HashSet<u64>,
    node: &TreeNode,
    depth: usize,
) {
    let has_children = !node.children.is_empty();
    let is_expanded = has_children && expanded.contains(&node.id);
    out.push(FlatRow {
        id: node.id,
        depth,
        label: node.label.clone(),
        has_children,
        expanded: is_expanded,
    });

    if is_expanded {
        for child in &node.children {
            push_flat(out, expanded, child, depth + 1);
        }
    }
}

impl Widget for TreeView {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        match event {
            Event::Pointer(fret_core::PointerEvent::Down {
                position, button, ..
            }) => {
                if *button == MouseButton::Left {
                    self.maybe_handle_disclosure_click(cx, *position);
                    if cx.stop_propagation {
                        return;
                    }
                }
            }
            Event::KeyDown { key, modifiers, .. } => {
                self.maybe_handle_tree_keys(cx, *key, *modifiers);
                if cx.stop_propagation {
                    return;
                }
            }
            _ => {}
        }

        self.list.event(cx, event);
        self.sync_selected_from_list();
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> fret_core::Size {
        if self.dirty {
            self.rebuild();
        }
        self.list.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.list.paint(cx);
    }
}
