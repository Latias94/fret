mod apply;
mod error;
mod model;
mod normalize;
mod patch;
mod wire;

#[cfg(test)]
mod tests;

pub use error::MenuBarError;
pub use model::{
    Menu, MenuBar, MenuItem, MenuItemToggle, MenuItemToggleKind, MenuRole, SystemMenuType,
};
pub use patch::{
    ItemAnchor, ItemSelector, ItemSelectorTyped, MenuBarConfig, MenuBarPatch, MenuBarPatchOp,
    MenuTarget,
};
pub use wire::{
    MenuBarFileV1, MenuBarFileV2, MenuFileV1, MenuFileV2, MenuItemFileV1, MenuItemFileV2,
};
