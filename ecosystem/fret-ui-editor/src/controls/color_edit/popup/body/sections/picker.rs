use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::input_group::derived_test_id;

use super::super::super::super::{ColorEditPopupOptions, ColorEditPopupPicker};
use super::super::super::picker::{alpha_bar, hsv_hue_wheel_picker, hsv_picker};

pub(super) fn color_popup_picker_section<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    effective_popup_options: ColorEditPopupOptions,
    enabled: bool,
    popup_test_id: Option<&Arc<str>>,
) -> Option<AnyElement> {
    match effective_popup_options.picker {
        ColorEditPopupPicker::HsvHueBar => Some(hsv_picker(
            cx,
            current,
            model,
            draft,
            error,
            show_alpha,
            effective_popup_options.shows_alpha_bar(show_alpha),
            enabled,
            derived_test_id(popup_test_id, "hsv"),
        )),
        ColorEditPopupPicker::HsvHueWheel => Some(hsv_hue_wheel_picker(
            cx,
            current,
            model,
            draft,
            error,
            show_alpha,
            effective_popup_options.shows_alpha_bar(show_alpha),
            enabled,
            derived_test_id(popup_test_id, "hsv-wheel"),
        )),
        ColorEditPopupPicker::Hidden => None,
    }
}

pub(super) fn color_popup_standalone_alpha_bar_section<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    effective_popup_options: ColorEditPopupOptions,
    show_alpha: bool,
    enabled: bool,
    popup_test_id: Option<&Arc<str>>,
) -> Option<AnyElement> {
    if effective_popup_options.picker != ColorEditPopupPicker::Hidden
        || !effective_popup_options.shows_alpha_bar(show_alpha)
    {
        return None;
    }

    Some(alpha_bar(
        cx,
        current,
        model,
        draft,
        error,
        enabled,
        derived_test_id(popup_test_id, "alpha"),
    ))
}
