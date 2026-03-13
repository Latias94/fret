#![cfg(feature = "imui")]

use std::sync::Arc;

use fret_authoring::UiWriter;
use fret_core::Color;
use fret_runtime::Model;
use fret_ui::UiHost;

use fret_ui_editor::controls::{
    AxisDragValue, AxisDragValueOptions, AxisDragValueOutcome, Checkbox, CheckboxOptions,
    DragValue, DragValueOptions, DragValueOutcome, EnumSelect, EnumSelectItem, EnumSelectOptions,
    NumericFormatFn, NumericParseFn, NumericValueConstraints, Slider, SliderOptions, TextField,
    TextFieldOptions, TransformEdit, TransformEditAxisOutcome, Vec3Edit, VecEditAxisOutcome,
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

    let on_drag_outcome = Arc::new(
        |_host: &mut dyn fret_ui::action::UiActionHost,
         _action_cx: fret_ui::action::ActionCx,
         _outcome: DragValueOutcome| {},
    );
    let on_axis_outcome = Arc::new(
        |_host: &mut dyn fret_ui::action::UiActionHost,
         _action_cx: fret_ui::action::ActionCx,
         _outcome: AxisDragValueOutcome| {},
    );
    let on_vec_axis_outcome = Arc::new(
        |_host: &mut dyn fret_ui::action::UiActionHost,
         _action_cx: fret_ui::action::ActionCx,
         _outcome: VecEditAxisOutcome| {},
    );
    let on_transform_axis_outcome = Arc::new(
        |_host: &mut dyn fret_ui::action::UiActionHost,
         _action_cx: fret_ui::action::ActionCx,
         _outcome: TransformEditAxisOutcome| {},
    );

    let _ = DragValue::new(value_model.clone(), fmt.clone(), parse.clone())
        .on_outcome(Some(on_drag_outcome))
        .options(DragValueOptions {
            id_source: Some(Arc::from("tests.drag_value.outcome")),
            constraints: NumericValueConstraints {
                min: Some(0.0),
                max: Some(1.0),
                clamp: true,
                step: Some(0.125),
            },
            ..Default::default()
        });

    let _ = AxisDragValue::new(
        Arc::from("X"),
        Color::from_srgb_hex_rgb(0xf2_59_59),
        value_model.clone(),
        fmt.clone(),
        parse.clone(),
    )
    .on_outcome(Some(on_axis_outcome))
    .options(AxisDragValueOptions {
        id_source: Some(Arc::from("tests.axis_drag_value.outcome")),
        constraints: NumericValueConstraints {
            min: Some(-1.0),
            max: Some(1.0),
            clamp: true,
            step: Some(0.25),
        },
        ..Default::default()
    });

    let _ = Vec3Edit::new(
        value_model.clone(),
        value_model.clone(),
        value_model.clone(),
        fmt.clone(),
        parse.clone(),
    )
    .on_axis_outcome(Some(on_vec_axis_outcome))
    .options(Default::default());

    let _ = TransformEdit::new(
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
        fmt.clone(),
        parse.clone(),
    )
    .on_axis_outcome(Some(on_transform_axis_outcome))
    .options(Default::default());

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
