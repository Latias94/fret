use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Color, Rect};
use fret_ui::element::ElementKind;
use fret_ui::elements::with_element_cx;

use super::super::super::ColorEditPopupRuntimeOptions;
use super::color_picker_options;
use crate::controls::{
    ColorEditPopupNumericInputs, ColorEditPopupOptions, ColorEditPopupPicker,
    ColorEditPopupSidePreview,
};

fn mount_picker_options(
    popup_options: ColorEditPopupOptions,
    show_alpha: bool,
    test_id: Option<Arc<str>>,
) -> fret_ui::element::AnyElement {
    let mut app = App::new();
    let window = AppWindowId::default();
    let current = Color::from_srgb_hex_rgb(0x33_66_99);
    let runtime_model = app.models_mut().insert(ColorEditPopupRuntimeOptions {
        default_picker: popup_options.picker,
        picker: popup_options.picker,
        default_alpha_bar: popup_options.alpha_bar,
        alpha_bar: popup_options.alpha_bar,
    });
    let runtime = popup_options.runtime_defaults();

    with_element_cx(
        &mut app,
        window,
        Rect::default(),
        "color-edit-popup-options",
        |cx| {
            color_picker_options(
                cx,
                current,
                popup_options,
                runtime,
                runtime_model,
                show_alpha,
                true,
                test_id,
            )
        },
    )
}

#[test]
fn single_visible_picker_option_returns_the_option_directly() {
    let element = mount_picker_options(
        ColorEditPopupOptions {
            picker: ColorEditPopupPicker::HsvHueBar,
            numeric_inputs: ColorEditPopupNumericInputs::Hidden,
            side_preview: ColorEditPopupSidePreview::Hidden,
            presets: false,
            alpha_bar: false,
            picker_options: true,
        },
        false,
        None,
    );

    assert!(
        matches!(element.kind, ElementKind::Flex(_)),
        "single visible picker option should still be the picker row"
    );
    assert_eq!(element.children.len(), 2);
    assert!(matches!(
        element.children[0].kind,
        ElementKind::Pressable(_)
    ));
    assert!(matches!(
        element.children[1].kind,
        ElementKind::Pressable(_)
    ));
}

#[test]
fn picker_options_with_test_id_keeps_the_single_option_directly() {
    let element = mount_picker_options(
        ColorEditPopupOptions {
            picker: ColorEditPopupPicker::HsvHueBar,
            numeric_inputs: ColorEditPopupNumericInputs::Hidden,
            side_preview: ColorEditPopupSidePreview::Hidden,
            presets: false,
            alpha_bar: false,
            picker_options: true,
        },
        false,
        Some(Arc::from("color-edit.popup.options")),
    );

    assert!(
        matches!(element.kind, ElementKind::Flex(_)),
        "diagnostic test ids should not force a vertical shell when only one option is visible"
    );
    assert_eq!(element.children.len(), 2);
    assert!(find_test_id(&element, "color-edit.popup.options").is_some());
}

fn find_test_id<'a>(
    element: &'a fret_ui::element::AnyElement,
    test_id: &str,
) -> Option<&'a fret_ui::element::AnyElement> {
    if element
        .semantics_decoration
        .as_ref()
        .and_then(|d| d.test_id.as_deref())
        == Some(test_id)
    {
        return Some(element);
    }

    element
        .children
        .iter()
        .find_map(|child| find_test_id(child, test_id))
}
