use std::sync::Arc;

use crate::controls::color_edit::ColorEditPopupNumericInputs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::controls::color_edit) enum ColorNumericInputMode {
    Rgb,
    Hsv,
}

impl ColorNumericInputMode {
    pub(in crate::controls::color_edit) fn test_suffix(self) -> &'static str {
        match self {
            Self::Rgb => "rgb",
            Self::Hsv => "hsv",
        }
    }

    pub(in crate::controls::color_edit) fn a11y_label(self) -> Arc<str> {
        match self {
            Self::Rgb => Arc::from("RGB color channels"),
            Self::Hsv => Arc::from("HSV color channels"),
        }
    }

    pub(in crate::controls::color_edit) fn invalid_message(self) -> Arc<str> {
        match self {
            Self::Rgb => Arc::from("Invalid RGB color values"),
            Self::Hsv => Arc::from("Invalid HSV color values"),
        }
    }
}

const RGB_HSV_NUMERIC_INPUT_MODES: [ColorNumericInputMode; 2] =
    [ColorNumericInputMode::Rgb, ColorNumericInputMode::Hsv];
const RGB_NUMERIC_INPUT_MODES: [ColorNumericInputMode; 1] = [ColorNumericInputMode::Rgb];
const HSV_NUMERIC_INPUT_MODES: [ColorNumericInputMode; 1] = [ColorNumericInputMode::Hsv];

pub(in crate::controls::color_edit) fn color_numeric_input_modes(
    numeric_inputs: ColorEditPopupNumericInputs,
) -> &'static [ColorNumericInputMode] {
    match numeric_inputs {
        ColorEditPopupNumericInputs::RgbAndHsv => &RGB_HSV_NUMERIC_INPUT_MODES,
        ColorEditPopupNumericInputs::Rgb => &RGB_NUMERIC_INPUT_MODES,
        ColorEditPopupNumericInputs::Hsv => &HSV_NUMERIC_INPUT_MODES,
        ColorEditPopupNumericInputs::Hidden => &[],
    }
}
