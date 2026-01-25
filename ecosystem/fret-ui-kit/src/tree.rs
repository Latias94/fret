use std::collections::HashSet;
use std::sync::Arc;

use fret_ui::element::Elements;
use fret_ui::{ElementContext, UiHost};

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
        cx: &mut ElementContext<'_, H>,
        entry: &TreeEntry,
        state: TreeRowState,
    ) -> Elements;

    fn render_trailing(
        &mut self,
        _cx: &mut ElementContext<'_, H>,
        _entry: &TreeEntry,
        _state: TreeRowState,
    ) -> Elements {
        Elements::default()
    }
}

impl<H: UiHost, F, R> TreeRowRenderer<H> for F
where
    F: FnMut(&mut ElementContext<'_, H>, &TreeEntry, TreeRowState) -> R,
    R: Into<Elements>,
{
    fn render_row(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        entry: &TreeEntry,
        state: TreeRowState,
    ) -> Elements {
        (self)(cx, entry, state).into()
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
                label: Arc::clone(&item.label),
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
