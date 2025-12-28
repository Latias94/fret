use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret_core::{Event, KeyCode, Modifiers, Size};
use fret_runtime::{CommandId, Model};
use fret_ui::declarative;
use fret_ui::widget::{CommandCx, EventCx, LayoutCx, PaintCx, Widget};
use fret_ui::{ElementCx, element::AnyElement};
use fret_ui::{UiHost, tree::UiTree};

use crate::Size as ComponentSize;

pub type TreeItemId = u64;

#[derive(Debug, Clone, Copy)]
pub struct TreeRowState {
    pub selected: bool,
    pub expanded: bool,
    pub disabled: bool,
    pub depth: usize,
    pub has_children: bool,
}

pub trait TreeRowRenderer<H: UiHost> {
    fn render_row(
        &mut self,
        cx: &mut ElementCx<'_, H>,
        entry: &TreeEntry,
        state: TreeRowState,
    ) -> Vec<AnyElement>;

    fn render_trailing(
        &mut self,
        _cx: &mut ElementCx<'_, H>,
        _entry: &TreeEntry,
        _state: TreeRowState,
    ) -> Vec<AnyElement> {
        Vec::new()
    }
}

impl<H: UiHost, F> TreeRowRenderer<H> for F
where
    F: FnMut(&mut ElementCx<'_, H>, &TreeEntry, TreeRowState) -> Vec<AnyElement>,
{
    fn render_row(
        &mut self,
        cx: &mut ElementCx<'_, H>,
        entry: &TreeEntry,
        state: TreeRowState,
    ) -> Vec<AnyElement> {
        (self)(cx, entry, state)
    }
}

#[derive(Debug, Clone)]
pub struct TreeItem {
    pub id: TreeItemId,
    pub label: Arc<str>,
    pub children: Vec<TreeItem>,
    pub disabled: bool,
}

impl TreeItem {
    pub fn new(id: TreeItemId, label: impl Into<Arc<str>>) -> Self {
        Self {
            id,
            label: label.into(),
            children: Vec::new(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn child(mut self, child: TreeItem) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: Vec<TreeItem>) -> Self {
        self.children = children;
        self
    }

    pub fn is_folder(&self) -> bool {
        !self.children.is_empty()
    }
}

#[derive(Debug, Default, Clone)]
pub struct TreeState {
    pub selected: Option<TreeItemId>,
    pub expanded: HashSet<TreeItemId>,
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub id: TreeItemId,
    pub label: Arc<str>,
    pub depth: usize,
    pub parent: Option<TreeItemId>,
    pub has_children: bool,
    pub disabled: bool,
}

pub fn flatten_tree(items: &[TreeItem], expanded: &HashSet<TreeItemId>) -> Vec<TreeEntry> {
    fn walk(
        out: &mut Vec<TreeEntry>,
        items: &[TreeItem],
        expanded: &HashSet<TreeItemId>,
        depth: usize,
        parent: Option<TreeItemId>,
    ) {
        for item in items {
            let has_children = item.is_folder();
            out.push(TreeEntry {
                id: item.id,
                label: item.label.clone(),
                depth,
                parent,
                has_children,
                disabled: item.disabled,
            });

            if has_children && expanded.contains(&item.id) {
                walk(out, &item.children, expanded, depth + 1, Some(item.id));
            }
        }
    }

    let mut out = Vec::new();
    walk(&mut out, items, expanded, 0, None);
    out
}

fn is_plain(mods: Modifiers) -> bool {
    !mods.shift && !mods.ctrl && !mods.alt && !mods.alt_gr && !mods.meta
}

fn first_selectable(entries: &[TreeEntry]) -> Option<TreeItemId> {
    entries.iter().find(|e| !e.disabled).map(|e| e.id)
}

fn last_selectable(entries: &[TreeEntry]) -> Option<TreeItemId> {
    entries.iter().rev().find(|e| !e.disabled).map(|e| e.id)
}

fn index_by_id(entries: &[TreeEntry]) -> HashMap<TreeItemId, usize> {
    entries.iter().enumerate().map(|(i, e)| (e.id, i)).collect()
}

fn next_selectable(entries: &[TreeEntry], start: usize, dir: i32) -> Option<TreeItemId> {
    if entries.is_empty() {
        return None;
    }

    if dir == 0 {
        return entries.get(start).filter(|e| !e.disabled).map(|e| e.id);
    }

    let mut i = start as i32;
    loop {
        i += dir;
        if i < 0 || i as usize >= entries.len() {
            return None;
        }
        let e = &entries[i as usize];
        if !e.disabled {
            return Some(e.id);
        }
    }
}

fn first_child_of(entries: &[TreeEntry], index: usize) -> Option<TreeItemId> {
    let cur = entries.get(index)?;
    let want_depth = cur.depth + 1;
    for e in entries.iter().skip(index + 1) {
        if e.depth < want_depth {
            return None;
        }
        if e.depth == want_depth && e.parent == Some(cur.id) && !e.disabled {
            return Some(e.id);
        }
    }
    None
}

pub struct TreeView {
    items: Model<Vec<TreeItem>>,
    state: Model<TreeState>,
}

impl TreeView {
    pub fn new(items: Model<Vec<TreeItem>>, state: Model<TreeState>) -> Self {
        Self { items, state }
    }

    fn entries_snapshot<H: UiHost>(
        &self,
        cx: &mut EventCx<'_, H>,
    ) -> (TreeState, Vec<TreeEntry>, HashMap<TreeItemId, usize>) {
        let state_value = cx
            .app
            .models()
            .get(self.state)
            .cloned()
            .unwrap_or_else(TreeState::default);
        let items_value = cx.app.models().get(self.items).cloned().unwrap_or_default();
        let entries = flatten_tree(&items_value, &state_value.expanded);
        let by_id = index_by_id(&entries);
        (state_value, entries, by_id)
    }

    fn set_selected<H: UiHost>(&self, cx: &mut EventCx<'_, H>, id: Option<TreeItemId>) {
        let _ = cx.app.models_mut().update(self.state, |s| s.selected = id);
        cx.stop_propagation();
    }

    fn toggle_expanded<H: UiHost>(&self, cx: &mut EventCx<'_, H>, id: TreeItemId) {
        let _ = cx.app.models_mut().update(self.state, |s| {
            if s.expanded.contains(&id) {
                s.expanded.remove(&id);
            } else {
                s.expanded.insert(id);
            }
            s.selected = Some(id);
        });
        cx.stop_propagation();
    }
}

impl<H: UiHost> Widget<H> for TreeView {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Event::KeyDown { key, modifiers, .. } = event else {
            return;
        };
        if !is_plain(*modifiers) {
            return;
        }

        enum Action {
            None,
            Select(Option<TreeItemId>),
            Toggle(TreeItemId),
        }

        let action = {
            let (state, entries, by_id) = self.entries_snapshot(cx);
            let cur_idx = state.selected.and_then(|id| by_id.get(&id).copied());

            match key {
                KeyCode::ArrowDown => {
                    let next = if let Some(i) = cur_idx {
                        next_selectable(&entries, i, 1)
                    } else {
                        first_selectable(&entries)
                    };
                    Action::Select(next)
                }
                KeyCode::ArrowUp => {
                    let next = if let Some(i) = cur_idx {
                        next_selectable(&entries, i, -1)
                    } else {
                        last_selectable(&entries)
                    };
                    Action::Select(next)
                }
                KeyCode::Home => Action::Select(first_selectable(&entries)),
                KeyCode::End => Action::Select(last_selectable(&entries)),
                KeyCode::ArrowRight => {
                    let Some(i) = cur_idx else {
                        return;
                    };
                    let cur = &entries[i];
                    if cur.has_children && !state.expanded.contains(&cur.id) {
                        Action::Toggle(cur.id)
                    } else if cur.has_children && state.expanded.contains(&cur.id) {
                        Action::Select(first_child_of(&entries, i))
                    } else {
                        Action::None
                    }
                }
                KeyCode::ArrowLeft => {
                    let Some(i) = cur_idx else {
                        return;
                    };
                    let cur = &entries[i];
                    if cur.has_children && state.expanded.contains(&cur.id) {
                        Action::Toggle(cur.id)
                    } else if let Some(parent) = cur.parent {
                        Action::Select(Some(parent))
                    } else {
                        Action::None
                    }
                }
                _ => Action::None,
            }
        };

        match action {
            Action::None => {}
            Action::Select(id) => {
                if id.is_some() {
                    self.set_selected(cx, id);
                }
            }
            Action::Toggle(id) => self.toggle_expanded(cx, id),
        }
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        if let Some(id) = command.as_str().strip_prefix("tree.select.") {
            let Ok(id) = id.parse::<u64>() else {
                return false;
            };
            let _ = cx
                .app
                .models_mut()
                .update(self.state, |s| s.selected = Some(id));
            cx.stop_propagation();
            return true;
        }
        if let Some(id) = command.as_str().strip_prefix("tree.toggle.") {
            let Ok(id) = id.parse::<u64>() else {
                return false;
            };
            let _ = cx.app.models_mut().update(self.state, |s| {
                if s.expanded.contains(&id) {
                    s.expanded.remove(&id);
                } else {
                    s.expanded.insert(id);
                }
                s.selected = Some(id);
            });
            cx.stop_propagation();
            return true;
        }
        false
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        if let Some(&child) = cx.children.first() {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        if let Some(&child) = cx.children.first() {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}

#[derive(Clone)]
pub struct TreeViewHandles {
    pub items: Model<Vec<TreeItem>>,
    pub state: Model<TreeState>,
    pub tree_root: fret_core::NodeId,
}

pub fn create_tree_view<H: UiHost>(
    ui: &mut UiTree<H>,
    parent: fret_core::NodeId,
    items: Model<Vec<TreeItem>>,
    state: Model<TreeState>,
) -> TreeViewHandles {
    let tree_root = ui.create_node(TreeView::new(items, state));
    ui.add_child(parent, tree_root);

    TreeViewHandles {
        items,
        state,
        tree_root,
    }
}

/// Renders the tree virtualized list subtree (declarative element composition) into the provided
/// mount node.
///
/// Call this once per frame before `UiTree::layout_all` / `paint_all` for the relevant window.
pub fn render_tree_view_list<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn fret_core::UiServices,
    window: fret_core::AppWindowId,
    handles: &TreeViewHandles,
    size: ComponentSize,
) {
    let bounds = ui.debug_node_bounds(handles.tree_root).unwrap_or_default();

    let root = declarative::render_root(ui, app, services, window, bounds, "tree-view", |cx| {
        vec![crate::declarative::tree::tree_view(
            cx,
            handles.items,
            handles.state,
            size,
        )]
    });

    ui.set_children(handles.tree_root, vec![root]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::PlatformCapabilities;
    use fret_core::{
        Event, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, Px, Rect,
        Scene, Size, TextService, UiServices, geometry::Point,
    };
    use fret_ui::widget::{LayoutCx, PaintCx};

    #[derive(Default)]
    struct FakeServices(());

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: fret_core::TextStyle,
            _constraints: fret_core::TextConstraints,
        ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                fret_core::TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl fret_core::SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            false
        }
    }

    #[derive(Default)]
    struct TestContainer;

    impl<H: UiHost> Widget<H> for TestContainer {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            for &child in cx.children {
                cx.layout_in(child, cx.bounds);
            }
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            for &child in cx.children {
                cx.paint(child, cx.bounds);
            }
        }
    }

    struct Focusable;

    impl<H: UiHost> Widget<H> for Focusable {
        fn is_focusable(&self) -> bool {
            true
        }

        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            Size::new(Px(10.0), Px(10.0))
        }
    }

    fn run_frame(ui: &mut UiTree<App>, host: &mut App, services: &mut dyn UiServices) {
        let mut scene = Scene::default();
        ui.layout_all(
            host,
            services,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(400.0), Px(300.0)),
            ),
            1.0,
        );
        ui.paint_all(
            host,
            services,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(400.0), Px(300.0)),
            ),
            &mut scene,
            1.0,
        );
    }

    fn key_down(key: KeyCode) -> Event {
        Event::KeyDown {
            key,
            repeat: false,
            modifiers: Modifiers::default(),
        }
    }

    #[test]
    fn tree_navigation_and_expand_collapse_follow_apg_like_expectations() {
        let mut host = App::new();
        host.set_global(PlatformCapabilities::default());
        let mut services = FakeServices::default();

        let window = fret_core::AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
        ui.set_root(root);

        let items = host.models_mut().insert(vec![
            TreeItem::new(1, "root")
                .child(TreeItem::new(2, "a").disabled(true))
                .child(TreeItem::new(3, "b")),
            TreeItem::new(4, "c"),
        ]);
        let state = host.models_mut().insert(TreeState::default());

        let handles = create_tree_view(&mut ui, root, items, state);

        let focusable = ui.create_node(Focusable);
        ui.add_child(handles.tree_root, focusable);
        ui.set_focus(Some(focusable));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(&mut host, &mut services, &key_down(KeyCode::ArrowDown));
        assert_eq!(host.models().get(state).and_then(|s| s.selected), Some(1));

        ui.dispatch_event(&mut host, &mut services, &key_down(KeyCode::ArrowRight));
        let s = host.models().get(state).cloned().unwrap_or_default();
        assert!(s.expanded.contains(&1));
        assert_eq!(s.selected, Some(1));

        ui.dispatch_event(&mut host, &mut services, &key_down(KeyCode::ArrowDown));
        // Skips disabled child `a` and selects `b`.
        assert_eq!(host.models().get(state).and_then(|s| s.selected), Some(3));

        ui.dispatch_event(&mut host, &mut services, &key_down(KeyCode::ArrowLeft));
        assert_eq!(host.models().get(state).and_then(|s| s.selected), Some(1));

        ui.dispatch_event(&mut host, &mut services, &key_down(KeyCode::ArrowLeft));
        let s = host.models().get(state).cloned().unwrap_or_default();
        assert!(!s.expanded.contains(&1));
        assert_eq!(s.selected, Some(1));

        ui.dispatch_event(&mut host, &mut services, &key_down(KeyCode::End));
        assert_eq!(host.models().get(state).and_then(|s| s.selected), Some(4));
    }
}
