use super::config_v1::MenuBarConfigFileV1;
use super::config_v2::MenuBarConfigFileV2;
use super::shared::MenuBarVersionOnly;
use super::v1::MenuBarFileV1;
use super::v2::MenuBarFileV2;

use super::super::{Menu, MenuBar, MenuBarConfig, MenuBarError, MenuItem};
use std::sync::Arc;

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
