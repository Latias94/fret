use serde::Deserialize;

use super::super::{MenuBar, MenuBarConfig, MenuBarError, MenuBarPatch};
use super::patch_v1::MenuBarPatchOpFileV1;
use super::v1::{MenuBarFileV1, MenuFileV1};

#[derive(Debug, Deserialize)]
pub(super) struct MenuBarConfigFileV1 {
    pub menu_bar_version: u32,
    #[serde(default)]
    pub menus: Vec<MenuFileV1>,
    #[serde(default)]
    pub ops: Vec<MenuBarPatchOpFileV1>,
}

impl MenuBarConfigFileV1 {
    pub(super) fn try_into_config_v1(self) -> Result<MenuBarConfig, MenuBarError> {
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
