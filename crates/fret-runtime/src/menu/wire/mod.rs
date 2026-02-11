mod config_v1;
mod config_v2;
mod load;
mod patch_v1;
mod patch_v2;
mod shared;
mod v1;
mod v2;

pub use v1::{MenuBarFileV1, MenuFileV1, MenuItemFileV1};
pub use v2::{MenuBarFileV2, MenuFileV2, MenuItemFileV2};
