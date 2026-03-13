use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;

use fret_ui_editor::controls::{
    AxisDragValue, AxisDragValueOptions, DragValue, DragValueOptions, NumericFormatFn,
    NumericInput, NumericInputOptions, NumericParseFn, NumericPresentation, Slider, SliderOptions,
    TransformEdit, TransformEditOptions, Vec3Edit, VecEditOptions,
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

    let _axis_drag = AxisDragValue::new(
        Arc::from("X"),
        Color::from_srgb_hex_rgb(0xf2_59_59),
        value_model.clone(),
        Arc::new(|value| Arc::from(format!("{value:.2}"))),
        Arc::new(|text| text.trim().parse::<f64>().ok()),
    )
    .options(AxisDragValueOptions {
        prefix: Some(Arc::from("$")),
        suffix: Some(Arc::from("m")),
        id_source: Some(Arc::from("tests.axis_drag_affix")),
        ..Default::default()
    });

    let _slider = Slider::new(value_model.clone(), 0.0, 100.0).options(SliderOptions {
        prefix: Some(Arc::from("$")),
        suffix: Some(Arc::from("%")),
        id_source: Some(Arc::from("tests.slider_affix")),
        ..Default::default()
    });
}

#[allow(dead_code)]
fn composite_controls_accept_affixes(value_model: &Model<f64>) {
    let format: NumericFormatFn<f64> = Arc::new(|value| Arc::from(format!("{value:.2}")));
    let parse: NumericParseFn<f64> = Arc::new(|text| text.trim().parse::<f64>().ok());

    let _vec3 = Vec3Edit::new(
        value_model.clone(),
        value_model.clone(),
        value_model.clone(),
        format.clone(),
        parse.clone(),
    )
    .options(VecEditOptions {
        prefix: Some(Arc::from("$")),
        suffix: Some(Arc::from("mm")),
        id_source: Some(Arc::from("tests.vec_affix")),
        ..Default::default()
    });

    let _transform = TransformEdit::new(
        (
            value_model.clone(),
            value_model.clone(),
            value_model.clone(),
        ),
        (
            value_model.clone(),
            value_model.clone(),
            value_model.clone(),
        ),
        (
            value_model.clone(),
            value_model.clone(),
            value_model.clone(),
        ),
        format,
        parse,
    )
    .options(TransformEditOptions {
        position_suffix: Some(Arc::from("m")),
        rotation_suffix: Some(Arc::from("deg")),
        scale_suffix: Some(Arc::from("%")),
        id_source: Some(Arc::from("tests.transform_affix")),
        ..Default::default()
    });
}

#[allow(dead_code)]
fn numeric_controls_accept_presentation_bundle(value_model: &Model<f64>) {
    let (value_format, value_parse, value_affixes) = NumericPresentation::<f64>::fixed_decimals(2)
        .with_chrome_prefix("$")
        .with_chrome_suffix("px")
        .parts();
    let (blend_format, blend_parse, blend_affixes) =
        NumericPresentation::<f64>::percent_0_1(0).parts();

    let _numeric = NumericInput::new(
        value_model.clone(),
        value_format.clone(),
        value_parse.clone(),
    )
    .options(NumericInputOptions {
        prefix: value_affixes.prefix.clone(),
        suffix: value_affixes.suffix.clone(),
        id_source: Some(Arc::from("tests.numeric_presentation.input")),
        ..Default::default()
    });

    let _drag =
        DragValue::new(value_model.clone(), value_format, value_parse).options(DragValueOptions {
            prefix: value_affixes.prefix.clone(),
            suffix: value_affixes.suffix.clone(),
            id_source: Some(Arc::from("tests.numeric_presentation.drag")),
            ..Default::default()
        });

    let _slider = Slider::new(value_model.clone(), 0.0, 1.0)
        .format(blend_format)
        .parse(blend_parse)
        .options(SliderOptions {
            id_source: Some(Arc::from("tests.numeric_presentation.slider")),
            suffix: blend_affixes.suffix.clone(),
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

    let axis_drag = AxisDragValueOptions::default();
    assert!(axis_drag.prefix.is_none());
    assert!(axis_drag.suffix.is_none());

    let slider = SliderOptions::default();
    assert!(slider.prefix.is_none());
    assert!(slider.suffix.is_none());

    let vec_edit = VecEditOptions::default();
    assert!(vec_edit.prefix.is_none());
    assert!(vec_edit.suffix.is_none());

    let transform = TransformEditOptions::default();
    assert!(transform.position_prefix.is_none());
    assert!(transform.position_suffix.is_none());
    assert!(transform.rotation_prefix.is_none());
    assert!(transform.rotation_suffix.is_none());
    assert!(transform.scale_prefix.is_none());
    assert!(transform.scale_suffix.is_none());
}

#[test]
fn numeric_presentation_percent_keeps_slider_chrome_suffix_empty() {
    let presentation = NumericPresentation::<f64>::percent_0_1(0);
    assert_eq!(presentation.format()(0.25).as_ref(), "25%");
    assert!(presentation.chrome_suffix().is_none());
}
