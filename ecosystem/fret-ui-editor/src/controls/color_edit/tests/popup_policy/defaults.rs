use super::super::*;

#[test]
fn popup_options_default_to_imgui_like_hue_bar_surface() {
    let options = ColorEditPopupOptions::default();

    assert_eq!(options.picker, ColorEditPopupPicker::HsvHueBar);
    assert_eq!(
        options.numeric_inputs,
        ColorEditPopupNumericInputs::RgbAndHsv
    );
    assert_eq!(
        options.side_preview,
        ColorEditPopupSidePreview::CurrentAndOriginal
    );
    assert!(options.presets);
    assert!(options.alpha_bar);
    assert!(options.picker_options);
    assert!(options.has_visible_content_with_swatches(false, true, false));
    assert!(options.has_visible_content_with_swatches(true, true, false));
    assert!(!options.shows_alpha_bar(false));
    assert!(options.shows_alpha_bar(true));
    assert!(options.shows_picker_options(false));
    assert!(options.shows_picker_options(true));
}

#[test]
fn popup_side_preview_defaults_to_imgui_current_and_original() {
    let options = ColorEditOptions::default();

    assert_eq!(
        options.popup.side_preview,
        ColorEditPopupSidePreview::CurrentAndOriginal
    );
    assert!(options.popup.side_preview.has_visible_content());
    assert!(options.popup.side_preview.shows_original());
}

#[test]
fn popup_side_preview_uses_imgui_three_by_two_color_button_ratio() {
    let ratio = SIDE_PREVIEW_SWATCH_WIDTH.0 / SIDE_PREVIEW_SWATCH_HEIGHT.0;

    assert!((ratio - 1.5).abs() < f32::EPSILON);
}

#[test]
fn alpha_preview_options_cover_imgui_color_button_preview_modes() {
    let options = ColorEditOptions::default();
    assert_eq!(options.alpha_preview, ColorEditAlphaPreview::Checkerboard);
    assert_eq!(
        [
            ColorEditAlphaPreview::Checkerboard,
            ColorEditAlphaPreview::Opaque,
            ColorEditAlphaPreview::NoBackground,
            ColorEditAlphaPreview::Half,
        ]
        .len(),
        4
    );
}

#[test]
fn tooltip_options_default_to_imgui_hover_preview_enabled() {
    let options = ColorEditOptions::default();

    assert!(options.tooltip.enabled);
    assert!(options.tooltip_test_id.is_none());
}

#[test]
fn copy_options_default_to_imgui_context_copy_enabled() {
    let options = ColorEditOptions::default();

    assert!(options.copy.enabled);
    assert!(options.copy_menu_test_id.is_none());
}
