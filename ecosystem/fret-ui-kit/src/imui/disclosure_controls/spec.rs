use std::sync::Arc;

use super::super::{CollapsingHeaderOptions, TreeNodeOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DisclosureKind {
    CollapsingHeader,
    TreeNode,
}

#[derive(Debug, Clone)]
pub(super) struct DisclosureSpec {
    pub(super) kind: DisclosureKind,
    pub(super) label: Arc<str>,
    pub(super) enabled: bool,
    pub(super) open: Option<fret_runtime::Model<bool>>,
    pub(super) default_open: bool,
    pub(super) activate_shortcut: Option<fret_runtime::KeyChord>,
    pub(super) shortcut_repeat: bool,
    pub(super) selected: bool,
    pub(super) leaf: bool,
    pub(super) level: u32,
    pub(super) pos_in_set: Option<u32>,
    pub(super) set_size: Option<u32>,
    pub(super) root_test_id: Option<Arc<str>>,
    pub(super) header_test_id: Option<Arc<str>>,
    pub(super) content_test_id: Option<Arc<str>>,
}

impl DisclosureSpec {
    pub(super) fn collapsing_header(label: Arc<str>, options: CollapsingHeaderOptions) -> Self {
        Self {
            kind: DisclosureKind::CollapsingHeader,
            label,
            enabled: options.enabled,
            open: options.open,
            default_open: options.default_open,
            activate_shortcut: options.activate_shortcut,
            shortcut_repeat: options.shortcut_repeat,
            selected: false,
            leaf: false,
            level: 1,
            pos_in_set: None,
            set_size: None,
            root_test_id: options.test_id,
            header_test_id: options.header_test_id,
            content_test_id: options.content_test_id,
        }
    }

    pub(super) fn tree_node(label: Arc<str>, options: TreeNodeOptions) -> Self {
        let level = options.level.max(1);
        Self {
            kind: DisclosureKind::TreeNode,
            label,
            enabled: options.enabled,
            open: options.open,
            default_open: options.default_open,
            activate_shortcut: options.activate_shortcut,
            shortcut_repeat: options.shortcut_repeat,
            selected: options.selected,
            leaf: options.leaf,
            level,
            pos_in_set: options.pos_in_set,
            set_size: options.set_size,
            root_test_id: None,
            header_test_id: options.test_id,
            content_test_id: options.content_test_id,
        }
    }

    pub(super) fn has_children(&self) -> bool {
        !self.leaf
    }
}
