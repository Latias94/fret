use super::*;

use crate::ui::previews::material3 as legacy;

pub(in crate::ui) fn preview_material3_gallery(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_list_value: Model<Arc<str>>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_gallery(
        cx,
        material3_checkbox,
        material3_switch,
        material3_radio_value,
        material3_tabs_value,
        material3_list_value,
        material3_navigation_bar_value,
        material3_text_field_value,
        material3_text_field_disabled,
        material3_text_field_error,
        last_action,
    )
}

pub(in crate::ui) fn preview_material3_state_matrix(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    material3_menu_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_state_matrix(
        cx,
        material3_checkbox,
        material3_switch,
        material3_radio_value,
        material3_tabs_value,
        material3_navigation_bar_value,
        material3_text_field_value,
        material3_text_field_disabled,
        material3_text_field_error,
        material3_menu_open,
        last_action,
    )
}

pub(in crate::ui) fn preview_material3_touch_targets(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_touch_targets(
        cx,
        material3_checkbox,
        material3_switch,
        material3_radio_value,
        material3_tabs_value,
    )
}

pub(in crate::ui) fn preview_material3_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    legacy::preview_material3_button(cx)
}

pub(in crate::ui) fn preview_material3_icon_button(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    legacy::preview_material3_icon_button(cx)
}

pub(in crate::ui) fn preview_material3_checkbox(
    cx: &mut ElementContext<'_, App>,
    checked: Model<bool>,
) -> Vec<AnyElement> {
    legacy::preview_material3_checkbox(cx, checked)
}

pub(in crate::ui) fn preview_material3_switch(
    cx: &mut ElementContext<'_, App>,
    selected: Model<bool>,
) -> Vec<AnyElement> {
    legacy::preview_material3_switch(cx, selected)
}

pub(in crate::ui) fn preview_material3_slider(
    cx: &mut ElementContext<'_, App>,
    value: Model<f32>,
) -> Vec<AnyElement> {
    legacy::preview_material3_slider(cx, value)
}

pub(in crate::ui) fn preview_material3_radio(
    cx: &mut ElementContext<'_, App>,
    group_value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_radio(cx, group_value)
}

pub(in crate::ui) fn preview_material3_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    legacy::preview_material3_badge(cx)
}

pub(in crate::ui) fn preview_material3_top_app_bar(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    legacy::preview_material3_top_app_bar(cx)
}

pub(in crate::ui) fn preview_material3_bottom_sheet(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    legacy::preview_material3_bottom_sheet(cx, open)
}

pub(in crate::ui) fn preview_material3_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<time::Date>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_date_picker(cx, open, month, selected)
}

pub(in crate::ui) fn preview_material3_time_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    selected: Model<time::Time>,
) -> Vec<AnyElement> {
    legacy::preview_material3_time_picker(cx, open, selected)
}

pub(in crate::ui) fn preview_material3_segmented_button(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    legacy::preview_material3_segmented_button(cx)
}

pub(in crate::ui) fn preview_material3_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    legacy::preview_material3_select(cx)
}

pub(in crate::ui) fn preview_material3_autocomplete(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
    disabled: Model<bool>,
    error: Model<bool>,
    dialog_open: Model<bool>,
) -> Vec<AnyElement> {
    legacy::preview_material3_autocomplete(cx, value, disabled, error, dialog_open)
}

pub(in crate::ui) fn preview_material3_text_field(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
    disabled: Model<bool>,
    error: Model<bool>,
) -> Vec<AnyElement> {
    legacy::preview_material3_text_field(cx, value, disabled, error)
}

pub(in crate::ui) fn preview_material3_tabs(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_tabs(cx, value)
}

pub(in crate::ui) fn preview_material3_list(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_list(cx, value)
}

pub(in crate::ui) fn preview_material3_navigation_bar(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_navigation_bar(cx, value)
}

pub(in crate::ui) fn preview_material3_navigation_rail(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_navigation_rail(cx, value)
}

pub(in crate::ui) fn preview_material3_navigation_drawer(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_navigation_drawer(cx, value)
}

pub(in crate::ui) fn preview_material3_modal_navigation_drawer(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_modal_navigation_drawer(cx, open, value)
}

pub(in crate::ui) fn preview_material3_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_dialog(cx, open, last_action)
}

pub(in crate::ui) fn preview_material3_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_menu(cx, open, last_action)
}

pub(in crate::ui) fn preview_material3_snackbar(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    legacy::preview_material3_snackbar(cx, last_action)
}

pub(in crate::ui) fn preview_material3_tooltip(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    legacy::preview_material3_tooltip(cx)
}
