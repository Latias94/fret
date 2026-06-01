//! Minimal color edit control (swatch + hex input + picker popup).
//!
//! v1 scope:
//! - hex input for `#RRGGBB` (and optionally `#RRGGBBAA`)
//! - swatch button that opens HSV picker controls plus app-owned palette/history swatches
//! - RGB-only edits preserve alpha; `show_alpha` only controls explicit alpha editing
//! - per-control alpha preview policy mirroring Dear ImGui's ColorButton preview modes

mod drag_drop;
mod element;
mod input;
mod layout;
mod model;
mod options;
mod popup;
mod records;
mod state;
mod swatch;

#[cfg(test)]
mod tests;

pub use self::element::ColorEdit;
pub(in crate::controls::color_edit) use self::options::ColorEditPopupRuntimeOptions;
pub use self::options::{
    ColorEditAlphaPreview, ColorEditCopyOptions, ColorEditDragDropOptions, ColorEditOptions,
    ColorEditPopupNumericInputs, ColorEditPopupOptions, ColorEditPopupPicker,
    ColorEditPopupSidePreview, ColorEditTooltipOptions,
};
pub use self::records::{
    ColorEditDragDropComponents, ColorEditDragDropPayload, ColorEditEyedropperRequest,
    ColorEditPaletteEntry, ColorEditPaletteSlotDrop, OnColorEditEyedropper,
    OnColorEditPaletteSlotDrop, default_color_edit_palette,
};
const CHECKERBOARD_LIGHT_RGB: u32 = 0xd8_de_e8;
const CHECKERBOARD_DARK_RGB: u32 = 0x8b_95_a5;
const ALPHA_BAR_STEPS: usize = 8;
const HUE_BAR_STEPS: usize = 12;
const SV_PICKER_STEPS: usize = 8;
