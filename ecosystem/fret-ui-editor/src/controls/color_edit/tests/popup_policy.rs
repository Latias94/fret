use super::*;

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

#[test]
fn popup_runtime_options_are_local_overrides_until_defaults_change() {
    let options = ColorEditPopupOptions::default();
    let mut runtime = options.runtime_defaults();

    runtime.picker = ColorEditPopupPicker::HsvHueWheel;
    runtime.alpha_bar = false;
    runtime.sync_defaults(options.runtime_defaults());

    assert_eq!(runtime.picker, ColorEditPopupPicker::HsvHueWheel);
    assert!(!runtime.alpha_bar);

    let next_defaults = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::HsvHueBar,
        numeric_inputs: ColorEditPopupNumericInputs::Rgb,
        side_preview: ColorEditPopupSidePreview::Hidden,
        presets: false,
        alpha_bar: false,
        picker_options: true,
    };
    runtime.sync_defaults(next_defaults.runtime_defaults());

    assert_eq!(runtime.picker, ColorEditPopupPicker::HsvHueWheel);
    assert!(!runtime.alpha_bar);

    let changed_picker_defaults = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::HsvHueWheel,
        ..next_defaults
    };
    runtime.sync_defaults(changed_picker_defaults.runtime_defaults());

    assert_eq!(runtime.picker, ColorEditPopupPicker::HsvHueWheel);
    assert!(!runtime.alpha_bar);
}

#[test]
fn popup_runtime_options_do_not_re_enable_hidden_picker_policy() {
    let options = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::Hidden,
        numeric_inputs: ColorEditPopupNumericInputs::Hidden,
        side_preview: ColorEditPopupSidePreview::Hidden,
        presets: false,
        alpha_bar: true,
        picker_options: true,
    };
    let mut runtime = options.runtime_defaults();
    runtime.picker = ColorEditPopupPicker::HsvHueWheel;

    let effective = options.with_runtime_options(runtime);

    assert_eq!(effective.picker, ColorEditPopupPicker::Hidden);
    assert!(effective.shows_picker_options(true));
    assert!(!effective.shows_picker_options(false));
}

#[test]
fn popup_runtime_options_are_ignored_when_options_surface_is_disabled() {
    let options = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::HsvHueBar,
        numeric_inputs: ColorEditPopupNumericInputs::Hidden,
        side_preview: ColorEditPopupSidePreview::Hidden,
        presets: false,
        alpha_bar: true,
        picker_options: false,
    };
    let mut runtime = options.runtime_defaults();
    runtime.picker = ColorEditPopupPicker::HsvHueWheel;
    runtime.alpha_bar = false;

    let effective = options.with_runtime_options(runtime);

    assert_eq!(effective.picker, ColorEditPopupPicker::HsvHueBar);
    assert!(effective.alpha_bar);
    assert!(!effective.shows_picker_options(true));
}
