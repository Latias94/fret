mod fill;
mod side;

#[cfg(test)]
pub(in crate::controls::color_edit) use fill::{checkerboard_cell_color, opaque_preview_color};
pub(in crate::controls::color_edit::popup) use fill::{checkerboard_grid, fill_preview_layout};
pub(in crate::controls::color_edit) use fill::{
    color_preview_stack, preview_color_for_alpha_visibility,
};
pub(super) use side::color_side_preview;
#[cfg(test)]
pub(in crate::controls::color_edit) use side::{
    SIDE_PREVIEW_SWATCH_HEIGHT, SIDE_PREVIEW_SWATCH_WIDTH, restore_reference_color,
};
