use serde::Deserialize;

use super::super::{ItemAnchor, ItemSelector, MenuBarError, MenuBarPatchOp, MenuRole, MenuTarget};
use super::v2::MenuItemFileV2;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum MenuBarPatchOpFileV2 {
    AppendMenu {
        title: String,
        #[serde(default)]
        role: Option<MenuRole>,
        #[serde(default)]
        mnemonic: Option<char>,
        #[serde(default)]
        items: Vec<MenuItemFileV2>,
    },
    InsertMenuBefore {
        title: String,
        #[serde(default)]
        role: Option<MenuRole>,
        #[serde(default)]
        mnemonic: Option<char>,
        before: String,
        #[serde(default)]
        items: Vec<MenuItemFileV2>,
    },
    InsertMenuAfter {
        title: String,
        #[serde(default)]
        role: Option<MenuRole>,
        #[serde(default)]
        mnemonic: Option<char>,
        after: String,
        #[serde(default)]
        items: Vec<MenuItemFileV2>,
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
        item: MenuItemFileV2,
    },
    InsertItemAfter {
        menu: MenuTarget,
        after: ItemAnchor,
        item: MenuItemFileV2,
    },
    PrependItem {
        menu: MenuTarget,
        item: MenuItemFileV2,
    },
    AppendItem {
        menu: MenuTarget,
        item: MenuItemFileV2,
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

impl MenuBarPatchOpFileV2 {
    pub(super) fn into_op(self, index: usize) -> Result<MenuBarPatchOp, MenuBarError> {
        match self {
            Self::AppendMenu {
                title,
                role,
                mnemonic,
                items,
            } => {
                let mut out_items = Vec::with_capacity(items.len());
                for (idx, item) in items.into_iter().enumerate() {
                    out_items.push(item.into_menu_item(&format!("ops[{index}].items[{idx}]"))?);
                }
                Ok(MenuBarPatchOp::AppendMenu {
                    title,
                    role,
                    mnemonic,
                    items: out_items,
                })
            }
            Self::InsertMenuBefore {
                title,
                role,
                mnemonic,
                before,
                items,
            } => {
                let mut out_items = Vec::with_capacity(items.len());
                for (idx, item) in items.into_iter().enumerate() {
                    out_items.push(item.into_menu_item(&format!("ops[{index}].items[{idx}]"))?);
                }
                Ok(MenuBarPatchOp::InsertMenuBefore {
                    title,
                    role,
                    mnemonic,
                    before,
                    items: out_items,
                })
            }
            Self::InsertMenuAfter {
                title,
                role,
                mnemonic,
                after,
                items,
            } => {
                let mut out_items = Vec::with_capacity(items.len());
                for (idx, item) in items.into_iter().enumerate() {
                    out_items.push(item.into_menu_item(&format!("ops[{index}].items[{idx}]"))?);
                }
                Ok(MenuBarPatchOp::InsertMenuAfter {
                    title,
                    role,
                    mnemonic,
                    after,
                    items: out_items,
                })
            }
            Self::RemoveMenu { title } => Ok(MenuBarPatchOp::RemoveMenu { title }),
            Self::RenameMenu { from, to } => Ok(MenuBarPatchOp::RenameMenu { from, to }),
            Self::MoveMenuBefore { title, before } => {
                Ok(MenuBarPatchOp::MoveMenuBefore { title, before })
            }
            Self::MoveMenuAfter { title, after } => {
                Ok(MenuBarPatchOp::MoveMenuAfter { title, after })
            }
            Self::RemoveAt { menu, at } => Ok(MenuBarPatchOp::RemoveAt { menu, at }),
            Self::MoveAtBefore { menu, at, before } => {
                Ok(MenuBarPatchOp::MoveAtBefore { menu, at, before })
            }
            Self::MoveAtAfter { menu, at, after } => {
                Ok(MenuBarPatchOp::MoveAtAfter { menu, at, after })
            }
            Self::RemoveItem { menu, command } => Ok(MenuBarPatchOp::RemoveItem { menu, command }),
            Self::InsertItemBefore { menu, before, item } => Ok(MenuBarPatchOp::InsertItemBefore {
                menu,
                before,
                item: item.into_menu_item(&format!("ops[{index}].item"))?,
            }),
            Self::InsertItemAfter { menu, after, item } => Ok(MenuBarPatchOp::InsertItemAfter {
                menu,
                after,
                item: item.into_menu_item(&format!("ops[{index}].item"))?,
            }),
            Self::PrependItem { menu, item } => Ok(MenuBarPatchOp::PrependItem {
                menu,
                item: item.into_menu_item(&format!("ops[{index}].item"))?,
            }),
            Self::AppendItem { menu, item } => Ok(MenuBarPatchOp::AppendItem {
                menu,
                item: item.into_menu_item(&format!("ops[{index}].item"))?,
            }),
            Self::MoveItemBefore {
                menu,
                command,
                before,
            } => Ok(MenuBarPatchOp::MoveItemBefore {
                menu,
                command,
                before,
            }),
            Self::MoveItemAfter {
                menu,
                command,
                after,
            } => Ok(MenuBarPatchOp::MoveItemAfter {
                menu,
                command,
                after,
            }),
        }
    }
}
