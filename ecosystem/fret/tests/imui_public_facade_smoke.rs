#![cfg(feature = "imui")]

use std::sync::Arc;

use fret::component::prelude::{Model, Px, UiHost};
use fret::imui::{
    editor::{
        self,
        composites::{PropertyRow, PropertyRowOptions},
        controls::{EditorThemePresetPicker, EditorThemePresetPickerOptions},
        theme::EditorThemePresetV1,
    },
    kit::ListBoxOptions,
    prelude::UiWriter,
};

#[allow(dead_code)]
fn root_imui_editor_composites_compile<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    theme_preset_model: &Model<EditorThemePresetV1>,
) {
    editor::editor_theme_preset_picker(
        ui,
        EditorThemePresetPicker::new(theme_preset_model.clone()).options(
            EditorThemePresetPickerOptions::new()
                .test_id("tests.fret.imui.editor_theme")
                .item_test_id_prefix("tests.fret.imui.editor_theme.item"),
        ),
    );

    editor::property_row(
        ui,
        PropertyRow::new().options(PropertyRowOptions {
            test_id: Some(Arc::from("tests.fret.imui.property_row")),
            ..Default::default()
        }),
        |cx| cx.text("Name"),
        |cx| cx.text("Cube"),
        |_cx| None,
    );
}

#[test]
fn root_imui_facade_exposes_editor_composites_and_kit_sugar() {
    assert_eq!(
        EditorThemePresetV1::from_key("imgui-like-dense"),
        Some(EditorThemePresetV1::ImguiLikeDense)
    );

    let _ = EditorThemePresetPickerOptions::new()
        .disabled()
        .without_label()
        .test_id("tests.fret.imui.editor_theme")
        .item_test_id_prefix("tests.fret.imui.editor_theme.item");
    let _ = PropertyRowOptions::default();
    let _ = ListBoxOptions::new()
        .width(Px(180.0))
        .height(Px(72.0))
        .size(Px(220.0), Px(88.0));
}
