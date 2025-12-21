use crate::widget::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};
use fret_app::{CommandId, InputContext, Menu, MenuItem};
use fret_core::{Event, KeyCode, Modifiers, MouseButton, Px};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

use super::context_menu::{ContextMenuRequest, ContextMenuService};
use super::virtual_list::{VirtualList, VirtualListDataSource, VirtualListRow};

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
    pub indent_width: Px,
    pub disclosure_width: Px,
}

impl Default for TreeViewStyle {
    fn default() -> Self {
        Self {
            indent_width: Px(16.0),
            disclosure_width: Px(14.0),
        }
    }
}

#[derive(Debug, Clone)]
struct FlatRow {
    id: u64,
    depth: usize,
    text: String,
    has_children: bool,
}

#[derive(Debug, Clone)]
struct TreeViewDataSource {
    rows: Rc<Vec<FlatRow>>,
    indent_width: Px,
}

impl VirtualListDataSource for TreeViewDataSource {
    type Key = u64;

    fn len(&self) -> usize {
        self.rows.len()
    }

    fn key_at(&self, index: usize) -> Self::Key {
        self.rows[index].id
    }

    fn row_at(&self, index: usize) -> VirtualListRow<'_> {
        let row = &self.rows[index];
        VirtualListRow::new(Cow::Borrowed(row.text.as_str()))
            .with_indent_x(Px(row.depth as f32 * self.indent_width.0))
    }
}

#[derive(Debug)]
pub struct TreeView {
    roots: Vec<TreeNode>,
    expanded: HashSet<u64>,
    selected: Option<u64>,
    context_menu_target: Option<u64>,

    parent_by_id: HashMap<u64, Option<u64>>,
    first_child_by_id: HashMap<u64, u64>,

    flat: Rc<Vec<FlatRow>>,
    id_to_index: HashMap<u64, usize>,

    list: VirtualList<TreeViewDataSource>,
    style: TreeViewStyle,
    dirty: bool,
}

impl TreeView {
    pub fn new(roots: Vec<TreeNode>) -> Self {
        let style = TreeViewStyle::default();
        let flat = Rc::new(Vec::new());
        let mut view = Self {
            roots,
            expanded: HashSet::new(),
            selected: None,
            context_menu_target: None,
            parent_by_id: HashMap::new(),
            first_child_by_id: HashMap::new(),
            flat: flat.clone(),
            id_to_index: HashMap::new(),
            list: VirtualList::new(TreeViewDataSource {
                rows: flat,
                indent_width: style.indent_width,
            }),
            style,
            dirty: true,
        };
        view.rebuild();
        view
    }

    pub fn set_roots(&mut self, roots: Vec<TreeNode>) {
        self.roots = roots;
        self.dirty = true;
        self.rebuild();
    }

    pub fn row_id_at(&mut self, position: fret_core::Point) -> Option<u64> {
        if self.dirty {
            self.rebuild();
        }
        let index = self.list.row_index_at(position)?;
        self.flat.get(index).map(|r| r.id)
    }

    pub fn row_rect(&mut self, id: u64) -> Option<fret_core::Rect> {
        if self.dirty {
            self.rebuild();
        }
        let index = *self.id_to_index.get(&id)?;
        self.list.row_rect(index)
    }

    pub fn last_row_rect(&mut self) -> Option<fret_core::Rect> {
        if self.dirty {
            self.rebuild();
        }
        let n = self.list.row_count();
        if n == 0 {
            return None;
        }
        self.list.row_rect(n - 1)
    }

    pub fn content_bounds(&self) -> fret_core::Rect {
        self.list.content_bounds()
    }

    pub fn parent_of(&mut self, id: u64) -> Option<u64> {
        if self.dirty {
            self.rebuild();
        }
        self.parent_by_id.get(&id).copied().flatten()
    }

    pub fn with_expanded(mut self, ids: impl IntoIterator<Item = u64>) -> Self {
        for id in ids {
            self.expanded.insert(id);
        }
        self.dirty = true;
        self.rebuild();
        self
    }

    pub fn set_expanded(&mut self, ids: impl IntoIterator<Item = u64>) {
        self.expanded.clear();
        for id in ids {
            self.expanded.insert(id);
        }
        self.dirty = true;
        self.rebuild();
    }

    pub fn selected(&self) -> Option<u64> {
        self.selected
    }

    pub fn selected_keys(&self) -> &HashSet<u64> {
        self.list.selected_keys()
    }

    pub fn set_selected_keys(&mut self, keys: impl IntoIterator<Item = u64>, lead: Option<u64>) {
        if self.dirty {
            self.rebuild();
        }

        let mut visible: Vec<u64> = keys
            .into_iter()
            .filter(|id| self.id_to_index.contains_key(id))
            .collect();
        visible.sort_unstable();
        visible.dedup();

        let lead = lead.filter(|id| self.id_to_index.contains_key(id));
        let lead = lead.or_else(|| visible.first().copied());
        self.selected = lead;

        self.list.set_selected_keys(visible, lead);

        if let Some(lead) = lead {
            if let Some(index) = self.id_to_index.get(&lead).copied() {
                self.list.ensure_visible(index);
            }
        }
    }

    pub fn reveal(&mut self, id: u64) {
        if self.dirty {
            self.rebuild();
        }

        let mut changed = false;
        let mut cur = Some(id);
        while let Some(node) = cur {
            let parent = self.parent_by_id.get(&node).copied().flatten();
            let Some(parent) = parent else {
                break;
            };
            if self.expanded.insert(parent) {
                changed = true;
            }
            cur = Some(parent);
        }

        if changed {
            self.dirty = true;
            self.rebuild();
        }
    }

    fn rebuild(&mut self) {
        self.parent_by_id.clear();
        self.first_child_by_id.clear();
        self.build_index_maps();

        let mut flat: Vec<FlatRow> = Vec::new();
        for root in &self.roots {
            push_flat(&mut flat, &self.expanded, root, 0);
        }
        let flat = Rc::new(flat);
        self.flat = flat.clone();

        self.id_to_index.clear();
        for (i, row) in self.flat.iter().enumerate() {
            self.id_to_index.insert(row.id, i);
        }

        self.list.set_data(TreeViewDataSource {
            rows: flat,
            indent_width: self.style.indent_width,
        });

        self.sync_selected_from_list();
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
        self.selected = self.list.selected_lead_key();
    }

    fn sync_list_selection_from_selected(&mut self) {
        let Some(mut id) = self.selected else {
            self.list.set_selected_key(None);
            return;
        };

        loop {
            if let Some(&index) = self.id_to_index.get(&id) {
                self.list.set_selected_key(Some(id));
                self.list.ensure_visible(index);
                self.selected = Some(id);
                return;
            }

            let parent = self.parent_by_id.get(&id).copied().flatten();
            let Some(next) = parent else {
                self.selected = None;
                self.list.set_selected_key(None);
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
        if self.toggle_expanded(row.id) {
            self.dirty = true;
            self.rebuild();
            cx.invalidate_self(Invalidation::Layout);
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
        }
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

        self.context_menu_target = Some(row_id);

        let inv_ctx = InputContext {
            platform: cx.input_ctx.platform,
            ui_has_modal: cx.input_ctx.ui_has_modal,
            focus_is_text_input: false,
        };

        let menu = Menu {
            title: Arc::from("Hierarchy"),
            items: vec![
                MenuItem::Command {
                    command: CommandId::from("tree_view.expand"),
                    when: None,
                },
                MenuItem::Command {
                    command: CommandId::from("tree_view.collapse"),
                    when: None,
                },
                MenuItem::Separator,
                MenuItem::Command {
                    command: CommandId::from("tree_view.expand_all"),
                    when: None,
                },
                MenuItem::Command {
                    command: CommandId::from("tree_view.collapse_all"),
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

    fn expand_target(&mut self, id: u64) -> bool {
        if !self.first_child_by_id.contains_key(&id) {
            return false;
        }
        if self.expanded.insert(id) {
            self.dirty = true;
            self.rebuild();
        }
        true
    }

    fn collapse_target(&mut self, id: u64) -> bool {
        if self.expanded.remove(&id) {
            self.dirty = true;
            self.rebuild();
            return true;
        }
        false
    }

    fn expand_all(&mut self) {
        fn visit(expanded: &mut HashSet<u64>, node: &TreeNode) {
            if !node.children.is_empty() {
                expanded.insert(node.id);
                for child in &node.children {
                    visit(expanded, child);
                }
            }
        }

        for root in &self.roots {
            visit(&mut self.expanded, root);
        }
        self.dirty = true;
        self.rebuild();
    }

    fn collapse_all(&mut self) {
        if self.expanded.is_empty() {
            return;
        }
        self.expanded.clear();
        self.dirty = true;
        self.rebuild();
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

fn push_flat(out: &mut Vec<FlatRow>, expanded: &HashSet<u64>, node: &TreeNode, depth: usize) {
    let has_children = !node.children.is_empty();
    let is_expanded = has_children && expanded.contains(&node.id);
    let glyph = if has_children {
        if is_expanded { "▾ " } else { "▸ " }
    } else {
        "  "
    };
    out.push(FlatRow {
        id: node.id,
        depth,
        text: format!("{glyph}{}", node.label),
        has_children,
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
                position,
                button,
                modifiers: _,
            }) => {
                if *button == MouseButton::Left {
                    self.maybe_handle_disclosure_click(cx, *position);
                    if cx.stop_propagation {
                        return;
                    }
                } else if *button == MouseButton::Right {
                    if self.dirty {
                        self.rebuild();
                    }
                    let Some(index) = self.list.row_index_at(*position) else {
                        return;
                    };
                    let Some(row) = self.flat.get(index) else {
                        return;
                    };
                    let row_id = row.id;

                    cx.request_focus(cx.node);
                    self.selected = Some(row_id);
                    self.sync_list_selection_from_selected();
                    self.open_context_menu_at(cx, *position, row_id);
                    cx.stop_propagation();
                    return;
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

    fn command(&mut self, cx: &mut crate::widget::CommandCx<'_>, command: &CommandId) -> bool {
        let Some(target) = self.context_menu_target.or(self.selected) else {
            return false;
        };

        let did = match command.as_str() {
            "tree_view.expand" => self.expand_target(target),
            "tree_view.collapse" => self.collapse_target(target),
            "tree_view.expand_all" => {
                self.expand_all();
                true
            }
            "tree_view.collapse_all" => {
                self.collapse_all();
                true
            }
            _ => return false,
        };

        if did {
            cx.invalidate_self(Invalidation::Layout);
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
        }
        did
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
