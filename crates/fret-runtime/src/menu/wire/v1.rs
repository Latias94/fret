use std::sync::Arc;

use crate::CommandId;

use serde::Deserialize;

use super::super::{MenuBarError, MenuItem};
use super::shared::parse_when;

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
    pub(super) fn into_menu_item(self, path: &str) -> Result<MenuItem, MenuBarError> {
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
