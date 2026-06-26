use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Color, Rect};
use fret_ui::Theme;
use fret_ui::element::ElementKind;
use fret_ui::elements::with_element_cx;

use super::super::super::model::format_hex;
use super::*;
use crate::primitives::EditorDensity;

fn mount_numeric_inputs(
    numeric_inputs: ColorEditPopupNumericInputs,
    error: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
) -> fret_ui::element::AnyElement {
    let mut app = App::new();
    let window = AppWindowId::default();
    let current = Color::from_srgb_hex_rgb(0x33_66_99);
    let theme = Theme::global(&app);
    let density = EditorDensity::resolve(theme);
    let row_height = density.row_height;
    let (text_input_chrome, text_input_text_style) =
        crate::primitives::chrome::resolve_editor_text_field_style(
            theme,
            fret_ui_kit::Size::default(),
            &fret_ui_kit::ChromeRefinement::default(),
        );
    let text_input_text_style = fret_ui_kit::typography::as_control_text(fret_core::TextStyle {
        size: fret_core::Px(10.0),
        line_height: Some(row_height),
        ..text_input_text_style
    });
    let error_color = theme.color_token("destructive");
    let model = app.models_mut().insert(current);
    let hex_draft = app
        .models_mut()
        .insert(format_hex(current, true).as_ref().to_string());
    let rgb_draft = app
        .models_mut()
        .insert(rgb_numeric_text(current, true).as_ref().to_string());
    let hsv_draft = app
        .models_mut()
        .insert(hsv_numeric_text(current).as_ref().to_string());
    let error = app.models_mut().insert(error);

    with_element_cx(
        &mut app,
        window,
        Rect::default(),
        "color-edit-popup-numeric",
        |cx| {
            color_numeric_inputs(
                cx,
                current,
                model,
                hex_draft,
                rgb_draft,
                hsv_draft,
                error,
                numeric_inputs,
                true,
                true,
                row_height,
                text_input_chrome,
                text_input_text_style,
                error_color,
                test_id,
            )
        },
    )
}

#[test]
fn single_numeric_input_without_error_returns_the_input_root_directly() {
    let element = mount_numeric_inputs(
        ColorEditPopupNumericInputs::Rgb,
        None,
        Some(Arc::from("popup.numbers")),
    );

    assert!(
        matches!(element.kind, ElementKind::TextInput(_)),
        "single numeric input should mount the text input directly"
    );
    assert!(
        element.children.is_empty(),
        "direct numeric input root should not add a flex shell"
    );
}

#[test]
fn multi_numeric_inputs_keep_the_flex_shell() {
    let element = mount_numeric_inputs(ColorEditPopupNumericInputs::RgbAndHsv, None, None);

    assert!(
        matches!(element.kind, ElementKind::Flex(_)),
        "multiple numeric inputs should keep the list shell"
    );
    assert_eq!(element.children.len(), 2);
    assert!(matches!(
        element.children[0].kind,
        ElementKind::TextInput(_)
    ));
    assert!(matches!(
        element.children[1].kind,
        ElementKind::TextInput(_)
    ));
}

#[test]
fn single_numeric_input_with_error_keeps_the_flex_shell() {
    let element = mount_numeric_inputs(
        ColorEditPopupNumericInputs::Rgb,
        Some(Arc::from("Invalid color")),
        None,
    );

    assert!(
        matches!(element.kind, ElementKind::Flex(_)),
        "inline error should keep the list shell even for a single numeric input"
    );
    assert_eq!(element.children.len(), 2);
    assert!(matches!(
        element.children[0].kind,
        ElementKind::TextInput(_)
    ));
}

#[test]
fn numeric_inputs_do_not_resync_unchanged_unfocused_drafts() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let current = Color::from_srgb_hex_rgb(0x33_66_99);
    let theme = Theme::global(&app);
    let density = EditorDensity::resolve(theme);
    let row_height = density.row_height;
    let (text_input_chrome, text_input_text_style) =
        crate::primitives::chrome::resolve_editor_text_field_style(
            theme,
            fret_ui_kit::Size::default(),
            &fret_ui_kit::ChromeRefinement::default(),
        );
    let text_input_text_style = fret_ui_kit::typography::as_control_text(fret_core::TextStyle {
        size: fret_core::Px(10.0),
        line_height: Some(row_height),
        ..text_input_text_style
    });
    let error_color = theme.color_token("destructive");
    let model = app.models_mut().insert(current);
    let hex_draft = app
        .models_mut()
        .insert(format_hex(current, true).as_ref().to_string());
    let rgb_draft = app
        .models_mut()
        .insert(rgb_numeric_text(current, true).as_ref().to_string());
    let hsv_draft = app
        .models_mut()
        .insert(hsv_numeric_text(current).as_ref().to_string());
    let error = app.models_mut().insert(None::<Arc<str>>);
    let rgb_revision = rgb_draft.revision(&app);
    let hsv_revision = hsv_draft.revision(&app);

    let model_for_render = model.clone();
    let hex_draft_for_render = hex_draft.clone();
    let rgb_draft_for_render = rgb_draft.clone();
    let hsv_draft_for_render = hsv_draft.clone();
    let error_for_render = error.clone();
    let element = with_element_cx(
        &mut app,
        window,
        Rect::default(),
        "color-edit-popup-numeric-noop-sync",
        |cx| {
            color_numeric_inputs(
                cx,
                current,
                model_for_render,
                hex_draft_for_render,
                rgb_draft_for_render,
                hsv_draft_for_render,
                error_for_render,
                ColorEditPopupNumericInputs::RgbAndHsv,
                true,
                true,
                row_height,
                text_input_chrome,
                text_input_text_style,
                error_color,
                Some(Arc::from("color-edit-popup-numeric-noop-sync")),
            )
        },
    );

    assert!(matches!(element.kind, ElementKind::Flex(_)));
    assert_eq!(rgb_draft.revision(&app), rgb_revision);
    assert_eq!(hsv_draft.revision(&app), hsv_revision);
}
