use std::sync::Arc;

use crate::{CommandId, WhenExpr};

use serde::Deserialize;

use super::{
    ItemAnchor, ItemSelector, Menu, MenuBar, MenuBarConfig, MenuBarError, MenuBarPatch,
    MenuBarPatchOp, MenuItem, MenuRole, MenuTarget, SystemMenuType,
};

impl MenuBar {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MenuBarError> {
        let version: MenuBarVersionOnly =
            serde_json::from_slice(bytes).map_err(|source| MenuBarError::ParseFailed { source })?;
        match version.menu_bar_version {
            1 => {
                let file: MenuBarFileV1 = serde_json::from_slice(bytes)
                    .map_err(|source| MenuBarError::ParseFailed { source })?;
                MenuBar::from_v1(file)
            }
            2 => {
                let file: MenuBarFileV2 = serde_json::from_slice(bytes)
                    .map_err(|source| MenuBarError::ParseFailed { source })?;
                MenuBar::from_v2(file)
            }
            other => Err(MenuBarError::UnsupportedVersion(other)),
        }
    }

    pub fn from_v1(file: MenuBarFileV1) -> Result<Self, MenuBarError> {
        if file.menu_bar_version != 1 {
            return Err(MenuBarError::UnsupportedVersion(file.menu_bar_version));
        }

        let mut menus: Vec<Menu> = Vec::with_capacity(file.menus.len());
        for (menu_index, menu) in file.menus.into_iter().enumerate() {
            let mut items: Vec<MenuItem> = Vec::with_capacity(menu.items.len());
            for (item_index, item) in menu.items.into_iter().enumerate() {
                items.push(
                    item.into_menu_item(&format!("menus[{menu_index}].items[{item_index}]"))?,
                );
            }

            menus.push(Menu {
                title: Arc::from(menu.title),
                role: None,
                mnemonic: None,
                items,
            });
        }

        Ok(MenuBar { menus })
    }

    pub fn from_v2(file: MenuBarFileV2) -> Result<Self, MenuBarError> {
        if file.menu_bar_version != 2 {
            return Err(MenuBarError::UnsupportedVersion(file.menu_bar_version));
        }

        let mut menus: Vec<Menu> = Vec::with_capacity(file.menus.len());
        for (menu_index, menu) in file.menus.into_iter().enumerate() {
            let mut items: Vec<MenuItem> = Vec::with_capacity(menu.items.len());
            for (item_index, item) in menu.items.into_iter().enumerate() {
                items.push(
                    item.into_menu_item(&format!("menus[{menu_index}].items[{item_index}]"))?,
                );
            }

            menus.push(Menu {
                title: Arc::from(menu.title),
                role: menu.role,
                mnemonic: menu.mnemonic,
                items,
            });
        }

        Ok(MenuBar { menus })
    }
}

impl MenuBarConfig {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MenuBarError> {
        let version: MenuBarVersionOnly =
            serde_json::from_slice(bytes).map_err(|source| MenuBarError::ParseFailed { source })?;

        match version.menu_bar_version {
            1 => {
                let file: MenuBarConfigFileV1 = serde_json::from_slice(bytes)
                    .map_err(|source| MenuBarError::ParseFailed { source })?;
                file.try_into_config_v1()
            }
            2 => {
                let file: MenuBarConfigFileV2 = serde_json::from_slice(bytes)
                    .map_err(|source| MenuBarError::ParseFailed { source })?;
                file.try_into_config_v2()
            }
            other => Err(MenuBarError::UnsupportedVersion(other)),
        }
    }
}

fn parse_when(path: &str, when: &str) -> Result<WhenExpr, MenuBarError> {
    let expr = WhenExpr::parse(when).map_err(|error| MenuBarError::WhenParseFailed {
        path: path.to_string(),
        error,
    })?;
    expr.validate()
        .map_err(|error| MenuBarError::WhenValidationFailed {
            path: path.to_string(),
            error: error.to_string(),
        })?;
    Ok(expr)
}

#[derive(Debug, Deserialize)]
struct MenuBarVersionOnly {
    pub menu_bar_version: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MenuBarFileV1 {
    pub menu_bar_version: u32,
    pub menus: Vec<MenuFileV1>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MenuFileV1 {
    pub title: String,
    pub items: Vec<MenuItemFileV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MenuItemFileV1 {
    Command {
        command: String,
        #[serde(default)]
        when: Option<String>,
    },
    Separator,
    Submenu {
        title: String,
        #[serde(default)]
        when: Option<String>,
        items: Vec<MenuItemFileV1>,
    },
}

impl MenuItemFileV1 {
    fn into_menu_item(self, path: &str) -> Result<MenuItem, MenuBarError> {
        match self {
            Self::Separator => Ok(MenuItem::Separator),
            Self::Command { command, when } => {
                let when = when
                    .as_deref()
                    .map(|w| parse_when(&format!("{path}.when"), w))
                    .transpose()?;
                Ok(MenuItem::Command {
                    command: CommandId::new(command),
                    when,
                    toggle: None,
                })
            }
            Self::Submenu { title, when, items } => {
                let when = when
                    .as_deref()
                    .map(|w| parse_when(&format!("{path}.when"), w))
                    .transpose()?;

                let mut out_items: Vec<MenuItem> = Vec::with_capacity(items.len());
                for (idx, item) in items.into_iter().enumerate() {
                    out_items.push(item.into_menu_item(&format!("{path}.items[{idx}]"))?);
                }

                Ok(MenuItem::Submenu {
                    title: Arc::from(title),
                    when,
                    items: out_items,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MenuBarFileV2 {
    pub menu_bar_version: u32,
    pub menus: Vec<MenuFileV2>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MenuFileV2 {
    pub title: String,
    #[serde(default)]
    pub role: Option<MenuRole>,
    #[serde(default)]
    pub mnemonic: Option<char>,
    pub items: Vec<MenuItemFileV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MenuItemFileV2 {
    Command {
        command: String,
        #[serde(default)]
        when: Option<String>,
    },
    Label {
        title: String,
    },
    Separator,
    Submenu {
        title: String,
        #[serde(default)]
        when: Option<String>,
        items: Vec<MenuItemFileV2>,
    },
    SystemMenu {
        title: String,
        #[serde(rename = "system")]
        menu_type: SystemMenuType,
    },
}

impl MenuItemFileV2 {
    fn into_menu_item(self, path: &str) -> Result<MenuItem, MenuBarError> {
        match self {
            Self::Separator => Ok(MenuItem::Separator),
            Self::Command { command, when } => {
                let when = when
                    .as_deref()
                    .map(|w| parse_when(&format!("{path}.when"), w))
                    .transpose()?;
                Ok(MenuItem::Command {
                    command: CommandId::new(command),
                    when,
                    toggle: None,
                })
            }
            Self::Label { title } => Ok(MenuItem::Label {
                title: Arc::from(title),
            }),
            Self::Submenu { title, when, items } => {
                let when = when
                    .as_deref()
                    .map(|w| parse_when(&format!("{path}.when"), w))
                    .transpose()?;

                let mut out_items: Vec<MenuItem> = Vec::with_capacity(items.len());
                for (idx, item) in items.into_iter().enumerate() {
                    out_items.push(item.into_menu_item(&format!("{path}.items[{idx}]"))?);
                }

                Ok(MenuItem::Submenu {
                    title: Arc::from(title),
                    when,
                    items: out_items,
                })
            }
            Self::SystemMenu { title, menu_type } => Ok(MenuItem::SystemMenu {
                title: Arc::from(title),
                menu_type,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum MenuBarPatchOpFileV1 {
    AppendMenu {
        title: String,
        #[serde(default)]
        items: Vec<MenuItemFileV1>,
    },
    InsertMenuBefore {
        title: String,
        before: String,
        #[serde(default)]
        items: Vec<MenuItemFileV1>,
    },
    InsertMenuAfter {
        title: String,
        after: String,
        #[serde(default)]
        items: Vec<MenuItemFileV1>,
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
        item: MenuItemFileV1,
    },
    InsertItemAfter {
        menu: MenuTarget,
        after: ItemAnchor,
        item: MenuItemFileV1,
    },
    PrependItem {
        menu: MenuTarget,
        item: MenuItemFileV1,
    },
    AppendItem {
        menu: MenuTarget,
        item: MenuItemFileV1,
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

impl MenuBarPatchOpFileV1 {
    fn into_op(self, index: usize) -> Result<MenuBarPatchOp, MenuBarError> {
        match self {
            Self::AppendMenu { title, items } => {
                let mut out_items = Vec::with_capacity(items.len());
                for (idx, item) in items.into_iter().enumerate() {
                    out_items.push(item.into_menu_item(&format!("ops[{index}].items[{idx}]"))?);
                }
                Ok(MenuBarPatchOp::AppendMenu {
                    title,
                    role: None,
                    mnemonic: None,
                    items: out_items,
                })
            }
            Self::InsertMenuBefore {
                title,
                before,
                items,
            } => {
                let mut out_items = Vec::with_capacity(items.len());
                for (idx, item) in items.into_iter().enumerate() {
                    out_items.push(item.into_menu_item(&format!("ops[{index}].items[{idx}]"))?);
                }
                Ok(MenuBarPatchOp::InsertMenuBefore {
                    title,
                    role: None,
                    mnemonic: None,
                    before,
                    items: out_items,
                })
            }
            Self::InsertMenuAfter {
                title,
                after,
                items,
            } => {
                let mut out_items = Vec::with_capacity(items.len());
                for (idx, item) in items.into_iter().enumerate() {
                    out_items.push(item.into_menu_item(&format!("ops[{index}].items[{idx}]"))?);
                }
                Ok(MenuBarPatchOp::InsertMenuAfter {
                    title,
                    role: None,
                    mnemonic: None,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum MenuBarPatchOpFileV2 {
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
    fn into_op(self, index: usize) -> Result<MenuBarPatchOp, MenuBarError> {
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

#[derive(Debug, Deserialize)]
struct MenuBarConfigFileV1 {
    pub menu_bar_version: u32,
    #[serde(default)]
    pub menus: Vec<MenuFileV1>,
    #[serde(default)]
    pub ops: Vec<MenuBarPatchOpFileV1>,
}

impl MenuBarConfigFileV1 {
    fn try_into_config_v1(self) -> Result<MenuBarConfig, MenuBarError> {
        let has_menus = !self.menus.is_empty();
        let has_ops = !self.ops.is_empty();

        if has_menus && has_ops {
            return Err(MenuBarError::PatchFailed {
                index: 0,
                error: "menubar config cannot contain both `menus` and `ops`".to_string(),
            });
        }

        if has_ops {
            let mut ops = Vec::with_capacity(self.ops.len());
            for (idx, op) in self.ops.into_iter().enumerate() {
                ops.push(op.into_op(idx)?);
            }
            return Ok(MenuBarConfig::Patch(MenuBarPatch { ops }));
        }

        Ok(MenuBarConfig::Replace(MenuBar::from_v1(MenuBarFileV1 {
            menu_bar_version: self.menu_bar_version,
            menus: self.menus,
        })?))
    }
}

#[derive(Debug, Deserialize)]
struct MenuBarConfigFileV2 {
    pub menu_bar_version: u32,
    #[serde(default)]
    pub menus: Vec<MenuFileV2>,
    #[serde(default)]
    pub ops: Vec<MenuBarPatchOpFileV2>,
}

impl MenuBarConfigFileV2 {
    fn try_into_config_v2(self) -> Result<MenuBarConfig, MenuBarError> {
        let has_menus = !self.menus.is_empty();
        let has_ops = !self.ops.is_empty();

        if has_menus && has_ops {
            return Err(MenuBarError::PatchFailed {
                index: 0,
                error: "menubar config cannot contain both `menus` and `ops`".to_string(),
            });
        }

        if has_ops {
            let mut ops = Vec::with_capacity(self.ops.len());
            for (idx, op) in self.ops.into_iter().enumerate() {
                ops.push(op.into_op(idx)?);
            }
            return Ok(MenuBarConfig::Patch(MenuBarPatch { ops }));
        }

        Ok(MenuBarConfig::Replace(MenuBar::from_v2(MenuBarFileV2 {
            menu_bar_version: self.menu_bar_version,
            menus: self.menus,
        })?))
    }
}
