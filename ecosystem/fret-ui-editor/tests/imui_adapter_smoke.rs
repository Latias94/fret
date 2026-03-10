#![cfg(feature = "imui")]

use std::sync::Arc;

use fret_authoring::UiWriter;
use fret_runtime::Model;
use fret_ui::UiHost;

use fret_ui_editor::controls::{
    Checkbox, CheckboxOptions, DragValue, DragValueOptions, EnumSelect, EnumSelectItem,
    EnumSelectOptions, NumericFormatFn, NumericParseFn, Slider, SliderOptions, TextField,
    TextFieldOptions,
};
use fret_ui_editor::imui;

#[allow(dead_code)]
fn editor_imui_adapters_compile<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    name_model: &Model<String>,
    value_model: &Model<f64>,
    enabled_model: &Model<bool>,
    mode_model: &Model<Option<Arc<str>>>,
) {
    let fmt: NumericFormatFn<f64> = Arc::new(|v| Arc::from(format!("{v:.3}")));
    let parse: NumericParseFn<f64> = Arc::new(|s| s.trim().parse::<f64>().ok());
    let items: Arc<[EnumSelectItem]> = vec![
        EnumSelectItem::new("lit", "Lit"),
        EnumSelectItem::new("unlit", "Unlit"),
    ]
    .into();

    imui::text_field(
        ui,
        TextField::new(name_model.clone()).options(TextFieldOptions {
            clear_button: true,
            ..Default::default()
        }),
    );

    imui::drag_value(
        ui,
        DragValue::new(value_model.clone(), fmt.clone(), parse.clone()).options(DragValueOptions {
            id_source: Some(Arc::from("tests.drag_value")),
            ..Default::default()
        }),
    );

    imui::slider(
        ui,
        Slider::new(value_model.clone(), 0.0, 1.0)
            .format(fmt)
            .parse(parse)
            .options(SliderOptions {
                id_source: Some(Arc::from("tests.slider")),
                ..Default::default()
            }),
    );

    imui::checkbox(
        ui,
        Checkbox::new(enabled_model.clone()).options(CheckboxOptions::default()),
    );

    imui::enum_select(
        ui,
        EnumSelect::new(mode_model.clone(), items).options(EnumSelectOptions {
            id_source: Some(Arc::from("tests.enum_select")),
            ..Default::default()
        }),
    );
}

#[test]
fn editor_imui_adapter_option_defaults_compile() {
    let items: Arc<[EnumSelectItem]> = vec![
        EnumSelectItem::new("lit", "Lit"),
        EnumSelectItem::new("unlit", "Unlit"),
    ]
    .into();

    assert_eq!(items.len(), 2);
    let _ = TextFieldOptions::default();
    let _ = DragValueOptions::default();
    let _ = SliderOptions::default();
    let _ = CheckboxOptions::default();
    let _ = EnumSelectOptions::default();
}
