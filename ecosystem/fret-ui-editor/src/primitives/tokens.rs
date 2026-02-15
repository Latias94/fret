//! Editor token keys (namespaced under `editor.*`).
//!
//! This module intentionally keeps tokens stringly-typed and small. The theme system already
//! provides typed primitives (`Color`, `Px`), while editor policy decides how to interpret them.

#[derive(Debug, Clone, Copy)]
pub struct EditorTokenKeys;

impl EditorTokenKeys {
    pub const DENSITY_ROW_HEIGHT: &'static str = "editor.density.row_height";
    pub const DENSITY_PADDING_X: &'static str = "editor.density.padding_x";
    pub const DENSITY_PADDING_Y: &'static str = "editor.density.padding_y";
    pub const DENSITY_HIT_THICKNESS: &'static str = "editor.density.hit_thickness";
    pub const DENSITY_ICON_SIZE: &'static str = "editor.density.icon_size";

    pub const NUMERIC_SCRUB_SPEED: &'static str = "editor.numeric.scrub_speed";
    pub const NUMERIC_SCRUB_SLOW_MULTIPLIER: &'static str = "editor.numeric.scrub_slow_multiplier";
    pub const NUMERIC_SCRUB_FAST_MULTIPLIER: &'static str = "editor.numeric.scrub_fast_multiplier";
    pub const NUMERIC_SCRUB_DRAG_THRESHOLD: &'static str = "editor.numeric.scrub_drag_threshold";
    pub const NUMERIC_ERROR_FG: &'static str = "editor.numeric.error_fg";

    pub const PROPERTY_COLUMN_GAP: &'static str = "editor.property.column_gap";
    pub const PROPERTY_GROUP_HEADER_HEIGHT: &'static str = "editor.property.group_header_height";

    pub const CHECKBOX_SIZE: &'static str = "editor.checkbox.size";
    pub const CHECKBOX_RADIUS: &'static str = "editor.checkbox.radius";

    pub const ENUM_SELECT_MAX_LIST_HEIGHT: &'static str = "editor.enum_select.max_list_height";

    pub const AXIS_X_COLOR: &'static str = "editor.axis.x_color";
    pub const AXIS_Y_COLOR: &'static str = "editor.axis.y_color";
    pub const AXIS_Z_COLOR: &'static str = "editor.axis.z_color";
    pub const AXIS_W_COLOR: &'static str = "editor.axis.w_color";

    pub const COLOR_SWATCH_SIZE: &'static str = "editor.color.swatch_size";
    pub const COLOR_POPUP_PADDING: &'static str = "editor.color.popup_padding";
}
