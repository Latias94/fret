use super::super::*;

#[test]
fn popup_numeric_input_modes_are_explicit_and_ordered() {
    assert_eq!(
        color_numeric_input_modes(ColorEditPopupNumericInputs::RgbAndHsv),
        &[ColorNumericInputMode::Rgb, ColorNumericInputMode::Hsv]
    );
    assert_eq!(
        color_numeric_input_modes(ColorEditPopupNumericInputs::Rgb),
        &[ColorNumericInputMode::Rgb]
    );
    assert_eq!(
        color_numeric_input_modes(ColorEditPopupNumericInputs::Hsv),
        &[ColorNumericInputMode::Hsv]
    );
    assert!(color_numeric_input_modes(ColorEditPopupNumericInputs::Hidden).is_empty());
}
