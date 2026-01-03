//! shadcn/ui `button-group` facade.
//!
//! In the current Fret component model this is a thin naming wrapper over `toggle_group`.

pub use crate::toggle_group::{
    ToggleGroup as ButtonGroup, ToggleGroupItem as ButtonGroupItem,
    ToggleGroupKind as ButtonGroupKind, toggle_group_multiple as button_group_multiple,
    toggle_group_single as button_group_single,
};
