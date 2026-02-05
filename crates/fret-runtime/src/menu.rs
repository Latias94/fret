use std::sync::Arc;

use crate::{CommandId, WhenExpr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MenuRole {
    App,
    File,
    Edit,
    View,
    Window,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemMenuType {
    Services,
}

/// A minimal, data-only menu model intended to power:
/// - future menubar rendering,
/// - context menus,
/// - command palette “breadcrumbs”.
///
/// This keeps menu structures derived from commands (ADR 0023) and avoids duplicating enablement
/// logic in widget code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuBar {
    pub menus: Vec<Menu>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    pub title: Arc<str>,
    pub role: Option<MenuRole>,
    pub items: Vec<MenuItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    Command {
        command: CommandId,
        when: Option<WhenExpr>,
    },
    Separator,
    Submenu {
        title: Arc<str>,
        when: Option<WhenExpr>,
        items: Vec<MenuItem>,
    },
    /// A menu managed by the OS (e.g. the macOS Services menu).
    ///
    /// In-window menu surfaces should ignore or render a disabled placeholder for these entries.
    SystemMenu {
        title: Arc<str>,
        menu_type: SystemMenuType,
    },
}

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuBarPatchOp {
    AppendMenu {
        title: String,
        role: Option<MenuRole>,
        items: Vec<MenuItem>,
    },
    InsertMenuBefore {
        title: String,
        role: Option<MenuRole>,
        before: String,
        items: Vec<MenuItem>,
    },
    InsertMenuAfter {
        title: String,
        role: Option<MenuRole>,
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

impl MenuBar {
    pub fn empty() -> Self {
        Self { menus: Vec::new() }
    }

    /// Normalize the menu structure for display across menu surfaces.
    ///
    /// This is a best-effort "shape cleanup" pass intended to prevent drift between:
    /// - OS menubars (runner mappings),
    /// - in-window menubars (overlay renderers),
    /// - other menu-like surfaces that derive from `MenuBar`.
    ///
    /// Current normalization rules:
    /// - remove leading separators,
    /// - collapse duplicate separators,
    /// - remove trailing separators,
    /// - recursively drop empty submenus (after normalizing their children).
    ///
    /// Note: this does **not** apply enable/disable gating; that is handled by
    /// `WindowCommandGatingSnapshot` and surface-specific policies.
    pub fn normalize(&mut self) {
        for menu in &mut self.menus {
            normalize_menu_items(&mut menu.items);
        }
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

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
                items,
            });
        }

        Ok(MenuBar { menus })
    }
}

fn normalize_menu_items(items: &mut Vec<MenuItem>) {
    let mut out: Vec<MenuItem> = Vec::with_capacity(items.len());
    let mut last_was_separator = false;

    for item in std::mem::take(items) {
        match item {
            MenuItem::Separator => {
                if out.is_empty() || last_was_separator {
                    continue;
                }
                out.push(MenuItem::Separator);
                last_was_separator = true;
            }
            MenuItem::Submenu {
                title,
                when,
                mut items,
            } => {
                normalize_menu_items(&mut items);
                if items.is_empty() {
                    continue;
                }
                out.push(MenuItem::Submenu { title, when, items });
                last_was_separator = false;
            }
            other => {
                out.push(other);
                last_was_separator = false;
            }
        }
    }

    while matches!(out.last(), Some(MenuItem::Separator)) {
        out.pop();
    }

    *items = out;
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

impl MenuBarPatch {
    pub fn apply_to(&self, base: &mut MenuBar) -> Result<(), MenuBarError> {
        for (idx, op) in self.ops.iter().cloned().enumerate() {
            apply_patch_op(base, idx, op)?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MenuBarError {
    #[error("failed to parse menubar json")]
    ParseFailed { source: serde_json::Error },
    #[error("unsupported menu_bar_version {0}")]
    UnsupportedVersion(u32),
    #[error("invalid when expression at {path}: {error}")]
    WhenParseFailed { path: String, error: String },
    #[error("invalid when expression at {path}: {error}")]
    WhenValidationFailed { path: String, error: String },
    #[error("menubar patch failed at ops[{index}]: {error}")]
    PatchFailed { index: usize, error: String },
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
        items: Vec<MenuItemFileV2>,
    },
    InsertMenuBefore {
        title: String,
        #[serde(default)]
        role: Option<MenuRole>,
        before: String,
        #[serde(default)]
        items: Vec<MenuItemFileV2>,
    },
    InsertMenuAfter {
        title: String,
        #[serde(default)]
        role: Option<MenuRole>,
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
            Self::AppendMenu { title, role, items } => {
                let mut out_items = Vec::with_capacity(items.len());
                for (idx, item) in items.into_iter().enumerate() {
                    out_items.push(item.into_menu_item(&format!("ops[{index}].items[{idx}]"))?);
                }
                Ok(MenuBarPatchOp::AppendMenu {
                    title,
                    role,
                    items: out_items,
                })
            }
            Self::InsertMenuBefore {
                title,
                role,
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
                    before,
                    items: out_items,
                })
            }
            Self::InsertMenuAfter {
                title,
                role,
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

fn apply_patch_op(
    menu_bar: &mut MenuBar,
    index: usize,
    op: MenuBarPatchOp,
) -> Result<(), MenuBarError> {
    fn menu_index(menu_bar: &MenuBar, title: &str) -> Option<usize> {
        menu_bar
            .menus
            .iter()
            .position(|m| m.title.as_ref() == title)
    }

    fn resolve_menu_items_mut<'a>(
        menu_bar: &'a mut MenuBar,
        target: &MenuTarget,
    ) -> Option<&'a mut Vec<MenuItem>> {
        let path: Vec<&str> = match target {
            MenuTarget::Title(title) => vec![title.as_str()],
            MenuTarget::Path(parts) => parts.iter().map(|s| s.as_str()).collect(),
        };
        let (first, rest) = path.split_first()?;
        let menu = menu_bar
            .menus
            .iter_mut()
            .find(|m| m.title.as_ref() == *first)?;
        let mut items: *mut Vec<MenuItem> = &mut menu.items;

        for title in rest {
            // Safety: we only reborrow `items` inside the loop and never keep references across
            // iterations.
            let next = unsafe { &mut *items }
                .iter_mut()
                .find_map(|item| match item {
                    MenuItem::Submenu {
                        title: t, items, ..
                    } if t.as_ref() == *title => Some(items as *mut Vec<MenuItem>),
                    _ => None,
                })?;
            items = next;
        }

        // Safety: `items` points to the last submenu `items` vec (or top-level menu items).
        Some(unsafe { &mut *items })
    }

    fn resolve_anchor_index(items: &[MenuItem], anchor: &ItemAnchor) -> Option<usize> {
        match anchor {
            ItemAnchor::Index(i) => Some(*i),
            ItemAnchor::Command(cmd) => items.iter().position(|item| match item {
                MenuItem::Command { command, .. } => command.as_str() == cmd.as_str(),
                _ => false,
            }),
        }
    }

    fn resolve_selector_index(items: &[MenuItem], selector: &ItemSelector) -> Option<usize> {
        match selector {
            ItemSelector::Anchor(anchor) => resolve_anchor_index(items, anchor),
            ItemSelector::Typed(ItemSelectorTyped::Submenu { title }) => items.iter().position(
                |item| matches!(item, MenuItem::Submenu { title: t, .. } if t.as_ref() == title),
            ),
        }
    }

    let fail = |error: String| Err(MenuBarError::PatchFailed { index, error });

    match op {
        MenuBarPatchOp::AppendMenu { title, role, items } => {
            if menu_index(menu_bar, &title).is_some() {
                return fail(format!("menu already exists: {title}"));
            }

            menu_bar.menus.push(Menu {
                title: Arc::from(title),
                role,
                items,
            });
            Ok(())
        }
        MenuBarPatchOp::InsertMenuBefore {
            title,
            role,
            before,
            items,
        } => {
            if menu_index(menu_bar, &title).is_some() {
                return fail(format!("menu already exists: {title}"));
            }
            let Some(insert_at) = menu_index(menu_bar, &before) else {
                return fail(format!("target menu not found: {before}"));
            };

            menu_bar.menus.insert(
                insert_at,
                Menu {
                    title: Arc::from(title),
                    role,
                    items,
                },
            );
            Ok(())
        }
        MenuBarPatchOp::InsertMenuAfter {
            title,
            role,
            after,
            items,
        } => {
            if menu_index(menu_bar, &title).is_some() {
                return fail(format!("menu already exists: {title}"));
            }
            let Some(after_idx) = menu_index(menu_bar, &after) else {
                return fail(format!("target menu not found: {after}"));
            };
            let insert_at = (after_idx + 1).min(menu_bar.menus.len());

            menu_bar.menus.insert(
                insert_at,
                Menu {
                    title: Arc::from(title),
                    role,
                    items,
                },
            );
            Ok(())
        }
        MenuBarPatchOp::RemoveMenu { title } => {
            let Some(idx) = menu_index(menu_bar, &title) else {
                return fail(format!("menu not found: {title}"));
            };
            menu_bar.menus.remove(idx);
            Ok(())
        }
        MenuBarPatchOp::RenameMenu { from, to } => {
            let Some(idx) = menu_index(menu_bar, &from) else {
                return fail(format!("menu not found: {from}"));
            };
            menu_bar.menus[idx].title = Arc::from(to);
            Ok(())
        }
        MenuBarPatchOp::MoveMenuBefore { title, before } => {
            let Some(from_idx) = menu_index(menu_bar, &title) else {
                return fail(format!("menu not found: {title}"));
            };
            let Some(mut to_idx) = menu_index(menu_bar, &before) else {
                return fail(format!("target menu not found: {before}"));
            };
            let menu = menu_bar.menus.remove(from_idx);
            if from_idx < to_idx {
                to_idx = to_idx.saturating_sub(1);
            }
            menu_bar.menus.insert(to_idx, menu);
            Ok(())
        }
        MenuBarPatchOp::MoveMenuAfter { title, after } => {
            let Some(from_idx) = menu_index(menu_bar, &title) else {
                return fail(format!("menu not found: {title}"));
            };
            let Some(mut to_idx) = menu_index(menu_bar, &after) else {
                return fail(format!("target menu not found: {after}"));
            };
            let menu = menu_bar.menus.remove(from_idx);
            if from_idx <= to_idx {
                to_idx = to_idx.saturating_sub(1);
            }
            let insert_at = (to_idx + 1).min(menu_bar.menus.len());
            menu_bar.menus.insert(insert_at, menu);
            Ok(())
        }
        MenuBarPatchOp::RemoveAt { menu, at } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let Some(item_idx) = resolve_selector_index(items, &at) else {
                return fail("item selector not found".to_string());
            };
            if item_idx >= items.len() {
                return fail(format!("index out of bounds: {item_idx}"));
            }
            items.remove(item_idx);
            Ok(())
        }
        MenuBarPatchOp::MoveAtBefore { menu, at, before } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let Some(from_idx) = resolve_selector_index(items, &at) else {
                return fail("item selector not found".to_string());
            };
            let Some(mut to_idx) = resolve_selector_index(items, &before) else {
                return fail("before selector not found".to_string());
            };
            if from_idx >= items.len() {
                return fail(format!("index out of bounds: {from_idx}"));
            }
            if to_idx >= items.len() {
                return fail(format!("index out of bounds: {to_idx}"));
            }

            let item = items.remove(from_idx);
            if from_idx < to_idx {
                to_idx = to_idx.saturating_sub(1);
            }
            items.insert(to_idx.min(items.len()), item);
            Ok(())
        }
        MenuBarPatchOp::MoveAtAfter { menu, at, after } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let Some(from_idx) = resolve_selector_index(items, &at) else {
                return fail("item selector not found".to_string());
            };
            let Some(mut to_idx) = resolve_selector_index(items, &after) else {
                return fail("after selector not found".to_string());
            };
            if from_idx >= items.len() {
                return fail(format!("index out of bounds: {from_idx}"));
            }
            if to_idx >= items.len() {
                return fail(format!("index out of bounds: {to_idx}"));
            }

            let item = items.remove(from_idx);
            if from_idx <= to_idx {
                to_idx = to_idx.saturating_sub(1);
            }
            let insert_at = (to_idx + 1).min(items.len());
            items.insert(insert_at, item);
            Ok(())
        }
        MenuBarPatchOp::RemoveItem { menu, command } => apply_patch_op(
            menu_bar,
            index,
            MenuBarPatchOp::RemoveAt {
                menu,
                at: ItemSelector::Anchor(ItemAnchor::Command(command)),
            },
        ),
        MenuBarPatchOp::InsertItemBefore { menu, before, item } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let before = ItemSelector::Anchor(before);
            let Some(item_idx) = resolve_selector_index(items, &before) else {
                return fail("before selector not found".to_string());
            };
            let insert_at = item_idx.min(items.len());
            items.insert(insert_at, item);
            Ok(())
        }
        MenuBarPatchOp::InsertItemAfter { menu, after, item } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let after = ItemSelector::Anchor(after);
            let Some(item_idx) = resolve_selector_index(items, &after) else {
                return fail("after selector not found".to_string());
            };
            let insert_at = (item_idx + 1).min(items.len());
            items.insert(insert_at, item);
            Ok(())
        }
        MenuBarPatchOp::PrependItem { menu, item } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            items.insert(0, item);
            Ok(())
        }
        MenuBarPatchOp::AppendItem { menu, item } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            items.push(item);
            Ok(())
        }
        MenuBarPatchOp::MoveItemBefore {
            menu,
            command,
            before,
        } => apply_patch_op(
            menu_bar,
            index,
            MenuBarPatchOp::MoveAtBefore {
                menu,
                at: ItemSelector::Anchor(ItemAnchor::Command(command)),
                before: ItemSelector::Anchor(before),
            },
        ),
        MenuBarPatchOp::MoveItemAfter {
            menu,
            command,
            after,
        } => apply_patch_op(
            menu_bar,
            index,
            MenuBarPatchOp::MoveAtAfter {
                menu,
                at: ItemSelector::Anchor(ItemAnchor::Command(command)),
                after: ItemSelector::Anchor(after),
            },
        ),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn base_menu_bar() -> MenuBar {
        MenuBar {
            menus: vec![Menu {
                title: Arc::from("File"),
                role: None,
                items: vec![
                    MenuItem::Command {
                        command: CommandId::new("app.open"),
                        when: None,
                    },
                    MenuItem::Submenu {
                        title: Arc::from("Recent"),
                        when: None,
                        items: vec![MenuItem::Separator],
                    },
                ],
            }],
        }
    }

    #[test]
    fn normalize_removes_leading_trailing_and_duplicate_separators() {
        let mut bar = MenuBar {
            menus: vec![Menu {
                title: Arc::from("File"),
                role: None,
                items: vec![
                    MenuItem::Separator,
                    MenuItem::Separator,
                    MenuItem::Command {
                        command: CommandId::new("app.open"),
                        when: None,
                    },
                    MenuItem::Separator,
                    MenuItem::Separator,
                    MenuItem::SystemMenu {
                        title: Arc::from("Services"),
                        menu_type: SystemMenuType::Services,
                    },
                    MenuItem::Separator,
                ],
            }],
        };

        bar.normalize();

        assert_eq!(bar.menus.len(), 1);
        assert!(matches!(
            bar.menus[0].items.as_slice(),
            [
                MenuItem::Command { .. },
                MenuItem::Separator,
                MenuItem::SystemMenu { .. }
            ]
        ));
    }

    #[test]
    fn normalize_drops_empty_submenus_recursively() {
        let mut bar = MenuBar {
            menus: vec![Menu {
                title: Arc::from("File"),
                role: None,
                items: vec![
                    MenuItem::Submenu {
                        title: Arc::from("Empty"),
                        when: None,
                        items: vec![MenuItem::Separator, MenuItem::Separator],
                    },
                    MenuItem::Submenu {
                        title: Arc::from("NonEmpty"),
                        when: None,
                        items: vec![
                            MenuItem::Submenu {
                                title: Arc::from("NestedEmpty"),
                                when: None,
                                items: vec![MenuItem::Separator],
                            },
                            MenuItem::Command {
                                command: CommandId::new("app.open"),
                                when: None,
                            },
                        ],
                    },
                ],
            }],
        };

        bar.normalize();

        let items = &bar.menus[0].items;
        assert_eq!(items.len(), 1);
        let MenuItem::Submenu { title, items, .. } = &items[0] else {
            panic!("expected submenu");
        };
        assert_eq!(title.as_ref(), "NonEmpty");
        assert!(items.iter().all(|i| !matches!(
            i,
            MenuItem::Submenu { title, .. } if title.as_ref() == "NestedEmpty"
        )));
    }

    #[test]
    fn patch_can_target_submenu_by_path() {
        let mut base = base_menu_bar();
        let patch = MenuBarPatch {
            ops: vec![MenuBarPatchOp::AppendItem {
                menu: MenuTarget::Path(vec!["File".into(), "Recent".into()]),
                item: MenuItem::Command {
                    command: CommandId::new("app.open_recent"),
                    when: None,
                },
            }],
        };

        patch.apply_to(&mut base).unwrap();

        let recent_items = match &base.menus[0].items[1] {
            MenuItem::Submenu { items, .. } => items,
            other => panic!("expected submenu, got {other:?}"),
        };

        assert!(recent_items.iter().any(|item| match item {
            MenuItem::Command { command, .. } => command.as_str() == "app.open_recent",
            _ => false,
        }));
    }

    #[test]
    fn insert_item_before_can_use_index_anchor() {
        let mut base = base_menu_bar();
        let patch = MenuBarPatch {
            ops: vec![MenuBarPatchOp::InsertItemBefore {
                menu: MenuTarget::Title("File".into()),
                before: ItemAnchor::Index(0),
                item: MenuItem::Command {
                    command: CommandId::new("app.new"),
                    when: None,
                },
            }],
        };
        patch.apply_to(&mut base).unwrap();

        match &base.menus[0].items[0] {
            MenuItem::Command { command, .. } => assert_eq!(command.as_str(), "app.new"),
            other => panic!("expected command, got {other:?}"),
        }
    }

    #[test]
    fn config_file_allows_menu_as_string_or_path_array() {
        let json = serde_json::json!({
            "menu_bar_version": 1,
            "ops": [
                { "type": "append_item", "menu": "File", "item": { "type": "separator" } },
                { "type": "append_item", "menu": ["File", "Recent"], "item": { "type": "separator" } }
            ]
        });

        let bytes = serde_json::to_vec(&json).unwrap();
        let cfg = MenuBarConfig::from_bytes(&bytes).unwrap();
        match cfg {
            MenuBarConfig::Patch(patch) => assert_eq!(patch.ops.len(), 2),
            other => panic!("expected patch, got {other:?}"),
        }
    }

    #[test]
    fn remove_at_can_remove_submenu_by_title() {
        let mut base = base_menu_bar();
        let patch = MenuBarPatch {
            ops: vec![MenuBarPatchOp::RemoveAt {
                menu: MenuTarget::Title("File".into()),
                at: ItemSelector::Typed(ItemSelectorTyped::Submenu {
                    title: "Recent".into(),
                }),
            }],
        };

        patch.apply_to(&mut base).unwrap();
        assert!(!base.menus[0].items.iter().any(|item| matches!(
            item,
            MenuItem::Submenu { title, .. } if title.as_ref() == "Recent"
        )));
    }

    #[test]
    fn v2_replace_parses_roles_and_system_menus() {
        let json = serde_json::json!({
            "menu_bar_version": 2,
            "menus": [
                {
                    "title": "Window",
                    "role": "window",
                    "items": [
                        { "type": "system_menu", "title": "Services", "system": "services" }
                    ]
                }
            ]
        });

        let bytes = serde_json::to_vec(&json).unwrap();
        let bar = MenuBar::from_bytes(&bytes).unwrap();
        assert_eq!(bar.menus.len(), 1);
        assert_eq!(bar.menus[0].role, Some(MenuRole::Window));
        assert!(matches!(
            &bar.menus[0].items[0],
            MenuItem::SystemMenu {
                title,
                menu_type: SystemMenuType::Services
            } if title.as_ref() == "Services"
        ));
    }

    #[test]
    fn v2_patch_ops_can_set_menu_role() {
        let json = serde_json::json!({
            "menu_bar_version": 2,
            "ops": [
                {
                    "type": "append_menu",
                    "title": "Help",
                    "role": "help",
                    "items": [
                        { "type": "command", "command": "app.command_palette" }
                    ]
                }
            ]
        });

        let bytes = serde_json::to_vec(&json).unwrap();
        let cfg = MenuBarConfig::from_bytes(&bytes).unwrap();
        match cfg {
            MenuBarConfig::Patch(patch) => assert!(matches!(
                &patch.ops[0],
                MenuBarPatchOp::AppendMenu {
                    role: Some(MenuRole::Help),
                    ..
                }
            )),
            other => panic!("expected patch, got {other:?}"),
        }
    }
}
