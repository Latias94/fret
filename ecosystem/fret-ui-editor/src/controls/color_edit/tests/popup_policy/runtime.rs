use super::super::*;

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
