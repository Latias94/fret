use std::sync::Arc;

use fret_runtime::Model;

use fret_ui_editor::controls::{
    DragValue, DragValueOptions, NumericFormatFn, NumericInput, NumericInputOptions, NumericParseFn,
};

#[allow(dead_code)]
fn numeric_controls_accept_affixes(value_model: &Model<f64>) {
    let format: NumericFormatFn<f64> = Arc::new(|value| Arc::from(format!("{value:.2}")));
    let parse: NumericParseFn<f64> = Arc::new(|text| text.trim().parse::<f64>().ok());

    let _numeric = NumericInput::new(value_model.clone(), format.clone(), parse.clone()).options(
        NumericInputOptions {
            prefix: Some(Arc::from("$")),
            suffix: Some(Arc::from("px")),
            id_source: Some(Arc::from("tests.numeric_affix")),
            ..Default::default()
        },
    );

    let _drag = DragValue::new(value_model.clone(), format, parse).options(DragValueOptions {
        prefix: Some(Arc::from("$")),
        suffix: Some(Arc::from("px")),
        id_source: Some(Arc::from("tests.drag_affix")),
        ..Default::default()
    });
}

#[test]
fn numeric_affix_option_defaults_are_empty() {
    let numeric = NumericInputOptions::default();
    assert!(numeric.prefix.is_none());
    assert!(numeric.suffix.is_none());

    let drag = DragValueOptions::default();
    assert!(drag.prefix.is_none());
    assert!(drag.suffix.is_none());
}
