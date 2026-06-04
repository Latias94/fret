use super::super::*;

#[test]
fn popup_options_can_hide_every_popup_affordance() {
    let options = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::Hidden,
        numeric_inputs: ColorEditPopupNumericInputs::Hidden,
        side_preview: ColorEditPopupSidePreview::Hidden,
        presets: false,
        alpha_bar: false,
        picker_options: false,
    };

    assert!(!options.has_visible_content_with_swatches(false, false, false));
    assert!(!options.has_visible_content_with_swatches(true, false, false));
}

#[test]
fn empty_palette_does_not_count_as_visible_popup_content() {
    let options = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::Hidden,
        numeric_inputs: ColorEditPopupNumericInputs::Hidden,
        side_preview: ColorEditPopupSidePreview::Hidden,
        presets: true,
        alpha_bar: false,
        picker_options: false,
    };

    assert!(!options.has_visible_content_with_swatches(false, false, false));
    assert!(options.has_visible_content_with_swatches(false, true, false));
}

#[test]
fn non_empty_history_counts_as_visible_popup_content_without_palette() {
    let options = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::Hidden,
        numeric_inputs: ColorEditPopupNumericInputs::Hidden,
        side_preview: ColorEditPopupSidePreview::Hidden,
        presets: false,
        alpha_bar: false,
        picker_options: false,
    };

    assert!(options.has_visible_content_with_swatches(false, false, true));
}
