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

    pub const TEXT_FIELD_PADDING_X: &'static str = "editor.text_field.padding_x";
    pub const TEXT_FIELD_PADDING_Y: &'static str = "editor.text_field.padding_y";
    pub const TEXT_FIELD_MIN_HEIGHT: &'static str = "editor.text_field.min_height";
    pub const TEXT_FIELD_RADIUS: &'static str = "editor.text_field.radius";
    pub const TEXT_FIELD_BORDER_WIDTH: &'static str = "editor.text_field.border_width";
    pub const TEXT_FIELD_BG: &'static str = "editor.text_field.bg";
    pub const TEXT_FIELD_BORDER: &'static str = "editor.text_field.border";
    pub const TEXT_FIELD_BORDER_FOCUS: &'static str = "editor.text_field.border_focus";
    pub const TEXT_FIELD_FG: &'static str = "editor.text_field.fg";
    pub const TEXT_FIELD_TEXT_PX: &'static str = "editor.text_field.text_px";
    pub const TEXT_FIELD_SELECTION: &'static str = "editor.text_field.selection";
    pub const CHROME_MUTED_FG: &'static str = "editor.chrome.muted_fg";
    pub const CHROME_ACCENT: &'static str = "editor.chrome.accent";
    pub const CHROME_RING: &'static str = "editor.chrome.ring";

    pub const CONTROL_INVALID_FG: &'static str = "editor.control.invalid_fg";
    pub const CONTROL_INVALID_BORDER: &'static str = "editor.control.invalid_border";
    pub const CONTROL_INVALID_BG: &'static str = "editor.control.invalid_bg";

    pub const NUMERIC_SCRUB_SPEED: &'static str = "editor.numeric.scrub_speed";
    pub const NUMERIC_SCRUB_SLOW_MULTIPLIER: &'static str = "editor.numeric.scrub_slow_multiplier";
    pub const NUMERIC_SCRUB_FAST_MULTIPLIER: &'static str = "editor.numeric.scrub_fast_multiplier";
    pub const NUMERIC_SCRUB_DRAG_THRESHOLD: &'static str = "editor.numeric.scrub_drag_threshold";
    pub const NUMERIC_ERROR_FG: &'static str = "editor.numeric.error_fg";
    pub const NUMERIC_ERROR_BORDER: &'static str = "editor.numeric.error_border";
    pub const NUMERIC_ERROR_BG: &'static str = "editor.numeric.error_bg";

    pub const PROPERTY_COLUMN_GAP: &'static str = "editor.property.column_gap";
    pub const PROPERTY_TRAILING_GAP: &'static str = "editor.property.trailing_gap";
    pub const PROPERTY_ROW_GAP: &'static str = "editor.property.row_gap";
    pub const PROPERTY_LABEL_WIDTH: &'static str = "editor.property.label_width";
    pub const PROPERTY_VALUE_MAX_WIDTH: &'static str = "editor.property.value_max_width";
    pub const PROPERTY_STATUS_SLOT_WIDTH: &'static str = "editor.property.status_slot_width";
    pub const PROPERTY_RESET_SLOT_WIDTH: &'static str = "editor.property.reset_slot_width";
    pub const PROPERTY_GROUP_HEADER_HEIGHT: &'static str = "editor.property.group_header_height";
    pub const PROPERTY_GROUP_CONTENT_GAP: &'static str = "editor.property.group_content_gap";
    pub const PROPERTY_AUTO_STACK_BELOW: &'static str = "editor.property.auto_stack_below";
    pub const PROPERTY_PANEL_GAP: &'static str = "editor.property.panel_gap";
    pub const PROPERTY_PANEL_HEADER_GAP: &'static str = "editor.property.panel_header_gap";
    pub const PROPERTY_PANEL_BG: &'static str = "editor.property.panel_bg";
    pub const PROPERTY_PANEL_BORDER: &'static str = "editor.property.panel_border";
    pub const PROPERTY_PANEL_HEADER_BG: &'static str = "editor.property.panel_header_bg";
    pub const PROPERTY_PANEL_HEADER_BORDER: &'static str = "editor.property.panel_header_border";
    pub const PROPERTY_PANEL_RADIUS: &'static str = "editor.property.panel_radius";
    pub const PROPERTY_GROUP_BORDER: &'static str = "editor.property.group_border";
    pub const PROPERTY_HEADER_BG: &'static str = "editor.property.header_bg";
    pub const PROPERTY_HEADER_BORDER: &'static str = "editor.property.header_border";
    pub const PROPERTY_HEADER_FG: &'static str = "editor.property.header_fg";

    pub const CHECKBOX_SIZE: &'static str = "editor.checkbox.size";
    pub const CHECKBOX_RADIUS: &'static str = "editor.checkbox.radius";

    pub const ENUM_SELECT_MAX_LIST_HEIGHT: &'static str = "editor.enum_select.max_list_height";
    pub const POPUP_BG: &'static str = "editor.popup.bg";
    pub const POPUP_BORDER: &'static str = "editor.popup.border";
    pub const POPUP_RADIUS: &'static str = "editor.popup.radius";
    pub const POPUP_SHADOW_OFFSET_Y: &'static str = "editor.popup.shadow_offset_y";
    pub const POPUP_SHADOW_BLUR: &'static str = "editor.popup.shadow.blur";
    pub const POPUP_SHADOW_SPREAD: &'static str = "editor.popup.shadow.spread";

    pub const AXIS_X_COLOR: &'static str = "editor.axis.x_color";
    pub const AXIS_Y_COLOR: &'static str = "editor.axis.y_color";
    pub const AXIS_Z_COLOR: &'static str = "editor.axis.z_color";
    pub const AXIS_W_COLOR: &'static str = "editor.axis.w_color";

    pub const VEC_AUTO_STACK_BELOW: &'static str = "editor.vec.auto_stack_below";
    pub const VEC_AXIS_MIN_WIDTH: &'static str = "editor.vec.axis_min_width";

    pub const COLOR_SWATCH_SIZE: &'static str = "editor.color.swatch_size";
    pub const COLOR_POPUP_PADDING: &'static str = "editor.color.popup_padding";

    pub const SLIDER_TRACK_HEIGHT: &'static str = "editor.slider.track_height";
    pub const SLIDER_THUMB_DIAMETER: &'static str = "editor.slider.thumb_diameter";
}
