use serde::Deserialize;

use super::{MenuBar, MenuItem, MenuRole};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuBarConfig {
    Replace(MenuBar),
    Patch(MenuBarPatch),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuBarPatch {
    pub ops: Vec<MenuBarPatchOp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum MenuTarget {
    /// Top-level menu title (back-compat with v1).
    Title(String),
    /// Menu path: `["File", "Recent"]` resolves to the `Recent` submenu under the `File` menu.
    Path(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum ItemAnchor {
    /// Anchor by command id (back-compat with v1).
    Command(String),
    /// Anchor by zero-based item index (useful for separators and submenus).
    Index(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum ItemSelector {
    /// Back-compat: a command id string or a numeric index.
    Anchor(ItemAnchor),
    /// Explicit selector for non-command items (e.g. submenus).
    Typed(ItemSelectorTyped),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemSelectorTyped {
    /// Select the first submenu item with the given title.
    ///
    /// Note: if multiple submenus share the same title, prefer using an index anchor instead.
    Submenu { title: String },
    /// Select the first label item with the given title.
    ///
    /// Note: if multiple labels share the same title, prefer using an index anchor instead.
    Label { title: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuBarPatchOp {
    AppendMenu {
        title: String,
        role: Option<MenuRole>,
        mnemonic: Option<char>,
        items: Vec<MenuItem>,
    },
    InsertMenuBefore {
        title: String,
        role: Option<MenuRole>,
        mnemonic: Option<char>,
        before: String,
        items: Vec<MenuItem>,
    },
    InsertMenuAfter {
        title: String,
        role: Option<MenuRole>,
        mnemonic: Option<char>,
        after: String,
        items: Vec<MenuItem>,
    },
    RemoveMenu {
        title: String,
    },
    RenameMenu {
        from: String,
        to: String,
    },
    MoveMenuBefore {
        title: String,
        before: String,
    },
    MoveMenuAfter {
        title: String,
        after: String,
    },

    RemoveAt {
        menu: MenuTarget,
        at: ItemSelector,
    },
    MoveAtBefore {
        menu: MenuTarget,
        at: ItemSelector,
        before: ItemSelector,
    },
    MoveAtAfter {
        menu: MenuTarget,
        at: ItemSelector,
        after: ItemSelector,
    },

    RemoveItem {
        menu: MenuTarget,
        command: String,
    },
    InsertItemBefore {
        menu: MenuTarget,
        before: ItemAnchor,
        item: MenuItem,
    },
    InsertItemAfter {
        menu: MenuTarget,
        after: ItemAnchor,
        item: MenuItem,
    },
    PrependItem {
        menu: MenuTarget,
        item: MenuItem,
    },
    AppendItem {
        menu: MenuTarget,
        item: MenuItem,
    },
    MoveItemBefore {
        menu: MenuTarget,
        command: String,
        before: ItemAnchor,
    },
    MoveItemAfter {
        menu: MenuTarget,
        command: String,
        after: ItemAnchor,
    },
}
