use std::sync::Arc;

use fret_core::{Color, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::input_group::derived_test_id;

use super::super::super::super::{
    ColorEditPopupNumericInputs, ColorEditPopupOptions, ColorEditPopupRuntimeOptions,
    OnColorEditEyedropper,
};
use super::super::super::eyedropper::color_eyedropper_action;
use super::super::super::numeric::color_numeric_inputs;
use super::super::super::options::color_picker_options;

pub(super) fn color_popup_picker_options_section<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    popup_options: ColorEditPopupOptions,
    runtime_options: ColorEditPopupRuntimeOptions,
    popup_runtime_options: Model<ColorEditPopupRuntimeOptions>,
    show_alpha: bool,
    enabled: bool,
    popup_test_id: Option<&Arc<str>>,
) -> Option<AnyElement> {
    popup_options.shows_picker_options(show_alpha).then(|| {
        color_picker_options(
            cx,
            current,
            popup_options,
            runtime_options,
            popup_runtime_options,
            show_alpha,
            enabled,
            derived_test_id(popup_test_id, "options"),
        )
    })
}

pub(super) fn color_popup_eyedropper_section<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    enabled: bool,
    on_eyedropper: Option<OnColorEditEyedropper>,
    eyedropper_test_id: Option<Arc<str>>,
) -> Option<AnyElement> {
    on_eyedropper.map(|on_eyedropper| {
        color_eyedropper_action(
            cx,
            current,
            model,
            draft,
            error,
            show_alpha,
            enabled,
            on_eyedropper,
            eyedropper_test_id,
        )
    })
}

pub(super) fn color_popup_numeric_section<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    rgb_draft: Model<String>,
    hsv_draft: Model<String>,
    numeric_error: Model<Option<Arc<str>>>,
    effective_popup_options: ColorEditPopupOptions,
    show_alpha: bool,
    enabled: bool,
    row_height: Px,
    text_input_chrome: fret_ui::TextInputStyle,
    text_input_text_style: TextStyle,
    error_color: Color,
    popup_test_id: Option<&Arc<str>>,
) -> Option<AnyElement> {
    (effective_popup_options.numeric_inputs != ColorEditPopupNumericInputs::Hidden).then(|| {
        color_numeric_inputs(
            cx,
            current,
            model,
            draft,
            rgb_draft,
            hsv_draft,
            numeric_error,
            effective_popup_options.numeric_inputs,
            show_alpha,
            enabled,
            row_height,
            text_input_chrome,
            text_input_text_style,
            error_color,
            derived_test_id(popup_test_id, "numbers"),
        )
    })
}
