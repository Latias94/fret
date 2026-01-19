use std::sync::Arc;

use crate::{CommandId, WhenExpr};

use serde::Deserialize;

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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MenuBarPatchOp {
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

impl MenuBar {
    pub fn empty() -> Self {
        Self { menus: Vec::new() }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MenuBarError> {
        let parsed: MenuBarFileAny =
            serde_json::from_slice(bytes).map_err(|source| MenuBarError::ParseFailed { source })?;
        match parsed.menu_bar_version {
            1 => MenuBar::from_v1(parsed.try_into_v1()?),
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
                items,
            });
        }

        Ok(MenuBar { menus })
    }
}

impl MenuBarConfig {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MenuBarError> {
        let parsed: MenuBarConfigFileAny =
            serde_json::from_slice(bytes).map_err(|source| MenuBarError::ParseFailed { source })?;

        match parsed.menu_bar_version {
            1 => parsed.try_into_config_v1(),
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

    fn item_index(menu: &Menu, command: &str) -> Option<usize> {
        menu.items.iter().position(|item| match item {
            MenuItem::Command { command: c, .. } => c.as_str() == command,
            _ => false,
        })
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
        let menu = menu_bar.menus.iter_mut().find(|m| m.title.as_ref() == *first)?;
        let mut items: *mut Vec<MenuItem> = &mut menu.items;

        for title in rest {
            // Safety: we only reborrow `items` inside the loop and never keep references across
            // iterations.
            let next = unsafe { &mut *items }
                .iter_mut()
                .find_map(|item| match item {
                    MenuItem::Submenu { title: t, items, .. } if t.as_ref() == *title => {
                        Some(items as *mut Vec<MenuItem>)
                    }
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

    let fail = |error: String| Err(MenuBarError::PatchFailed { index, error });

    match op {
        MenuBarPatchOp::AppendMenu { title, items } => {
            if menu_index(menu_bar, &title).is_some() {
                return fail(format!("menu already exists: {title}"));
            }

            let mut out_items: Vec<MenuItem> = Vec::with_capacity(items.len());
            for (idx, item) in items.into_iter().enumerate() {
                out_items.push(item.into_menu_item(&format!("ops[{index}].items[{idx}]"))?);
            }

            menu_bar.menus.push(Menu {
                title: Arc::from(title),
                items: out_items,
            });
            Ok(())
        }
        MenuBarPatchOp::InsertMenuBefore {
            title,
            before,
            items,
        } => {
            if menu_index(menu_bar, &title).is_some() {
                return fail(format!("menu already exists: {title}"));
            }
            let Some(insert_at) = menu_index(menu_bar, &before) else {
                return fail(format!("target menu not found: {before}"));
            };

            let mut out_items: Vec<MenuItem> = Vec::with_capacity(items.len());
            for (idx, item) in items.into_iter().enumerate() {
                out_items.push(item.into_menu_item(&format!("ops[{index}].items[{idx}]"))?);
            }

            menu_bar.menus.insert(
                insert_at,
                Menu {
                    title: Arc::from(title),
                    items: out_items,
                },
            );
            Ok(())
        }
        MenuBarPatchOp::InsertMenuAfter {
            title,
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

            let mut out_items: Vec<MenuItem> = Vec::with_capacity(items.len());
            for (idx, item) in items.into_iter().enumerate() {
                out_items.push(item.into_menu_item(&format!("ops[{index}].items[{idx}]"))?);
            }

            menu_bar.menus.insert(
                insert_at,
                Menu {
                    title: Arc::from(title),
                    items: out_items,
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
        MenuBarPatchOp::RemoveItem { menu, command } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let Some(item_idx) = items.iter().position(|item| match item {
                MenuItem::Command { command: c, .. } => c.as_str() == command.as_str(),
                _ => false,
            }) else {
                return fail(format!("command not found in menu: {command}"));
            };
            items.remove(item_idx);
            Ok(())
        }
        MenuBarPatchOp::InsertItemBefore { menu, before, item } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let Some(item_idx) = resolve_anchor_index(items, &before) else {
                return fail("before anchor not found".to_string());
            };
            let item = item.into_menu_item(&format!("ops[{index}].item"))?;
            let insert_at = item_idx.min(items.len());
            items.insert(insert_at, item);
            Ok(())
        }
        MenuBarPatchOp::InsertItemAfter { menu, after, item } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let Some(item_idx) = resolve_anchor_index(items, &after) else {
                return fail("after anchor not found".to_string());
            };
            let item = item.into_menu_item(&format!("ops[{index}].item"))?;
            let insert_at = (item_idx + 1).min(items.len());
            items.insert(insert_at, item);
            Ok(())
        }
        MenuBarPatchOp::PrependItem { menu, item } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let item = item.into_menu_item(&format!("ops[{index}].item"))?;
            items.insert(0, item);
            Ok(())
        }
        MenuBarPatchOp::AppendItem { menu, item } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let item = item.into_menu_item(&format!("ops[{index}].item"))?;
            items.push(item);
            Ok(())
        }
        MenuBarPatchOp::MoveItemBefore {
            menu,
            command,
            before,
        } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let Some(from_idx) = items.iter().position(|item| match item {
                MenuItem::Command { command: c, .. } => c.as_str() == command.as_str(),
                _ => false,
            }) else {
                return fail(format!("command not found in menu: {command}"));
            };
            let Some(mut to_idx) = resolve_anchor_index(items, &before) else {
                return fail("before anchor not found".to_string());
            };
            let item = items.remove(from_idx);
            if from_idx < to_idx {
                to_idx = to_idx.saturating_sub(1);
            }
            items.insert(to_idx.min(items.len()), item);
            Ok(())
        }
        MenuBarPatchOp::MoveItemAfter {
            menu,
            command,
            after,
        } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let Some(from_idx) = items.iter().position(|item| match item {
                MenuItem::Command { command: c, .. } => c.as_str() == command.as_str(),
                _ => false,
            }) else {
                return fail(format!("command not found in menu: {command}"));
            };
            let Some(mut to_idx) = resolve_anchor_index(items, &after) else {
                return fail("after anchor not found".to_string());
            };
            let item = items.remove(from_idx);
            if from_idx <= to_idx {
                to_idx = to_idx.saturating_sub(1);
            }
            let insert_at = (to_idx + 1).min(items.len());
            items.insert(insert_at, item);
            Ok(())
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
struct MenuBarFileAny {
    pub menu_bar_version: u32,
    pub menus: Vec<MenuFileV1>,
}

impl MenuBarFileAny {
    fn try_into_v1(self) -> Result<MenuBarFileV1, MenuBarError> {
        Ok(MenuBarFileV1 {
            menu_bar_version: self.menu_bar_version,
            menus: self.menus,
        })
    }
}

#[derive(Debug, Deserialize)]
struct MenuBarConfigFileAny {
    pub menu_bar_version: u32,
    #[serde(default)]
    pub menus: Vec<MenuFileV1>,
    #[serde(default)]
    pub ops: Vec<MenuBarPatchOp>,
}

impl MenuBarConfigFileAny {
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
            return Ok(MenuBarConfig::Patch(MenuBarPatch { ops: self.ops }));
        }

        Ok(MenuBarConfig::Replace(MenuBar::from_v1(MenuBarFileV1 {
            menu_bar_version: 1,
            menus: self.menus,
        })?))
    }
}
