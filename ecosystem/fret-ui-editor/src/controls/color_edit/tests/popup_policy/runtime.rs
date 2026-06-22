use super::super::super::state::sync_popup_runtime_options;
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

#[test]
fn popup_runtime_sync_only_bumps_revision_when_defaults_change() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let defaults = ColorEditPopupOptions::default().runtime_defaults();
    let runtime = app.models_mut().insert(defaults);

    app.models_mut()
        .update(&runtime, |runtime| {
            runtime.picker = ColorEditPopupPicker::HsvHueWheel;
            runtime.alpha_bar = false;
        })
        .unwrap();
    assert_eq!(runtime.revision(&app), Some(1));

    let runtime_for_render = runtime.clone();
    let _ = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        Rect::default(),
        "color-edit-popup-runtime-noop-sync",
        |cx| {
            sync_popup_runtime_options(cx, &runtime_for_render, defaults);
            cx.spacer(Default::default())
        },
    );

    assert_eq!(
        runtime.revision(&app),
        Some(1),
        "unchanged popup defaults should preserve local runtime overrides without bumping revision"
    );

    let changed_defaults = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::HsvHueWheel,
        alpha_bar: false,
        ..ColorEditPopupOptions::default()
    }
    .runtime_defaults();
    let runtime_for_render = runtime.clone();
    let _ = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        Rect::default(),
        "color-edit-popup-runtime-changed-sync",
        |cx| {
            sync_popup_runtime_options(cx, &runtime_for_render, changed_defaults);
            cx.spacer(Default::default())
        },
    );

    assert_eq!(runtime.revision(&app), Some(2));
    let synced = runtime.read_ref(&app, |runtime| *runtime).unwrap();
    assert_eq!(synced.default_picker, ColorEditPopupPicker::HsvHueWheel);
    assert!(!synced.default_alpha_bar);
}
