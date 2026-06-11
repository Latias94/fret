#![cfg(feature = "imui")]

use std::sync::Arc;

use fret::component::prelude::{Px, UiHost};
use fret::imui::{
    editor::{
        self,
        composites::{PropertyRow, PropertyRowOptions},
    },
    kit::ListBoxOptions,
    prelude::UiWriter,
};

#[allow(dead_code)]
fn root_imui_editor_composites_compile<H: UiHost + 'static>(ui: &mut impl UiWriter<H>) {
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
    let _ = PropertyRowOptions::default();
    let _ = ListBoxOptions::new()
        .width(Px(180.0))
        .height(Px(72.0))
        .size(Px(220.0), Px(88.0));
}
