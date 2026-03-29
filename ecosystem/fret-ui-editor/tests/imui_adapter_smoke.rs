#![cfg(feature = "imui")]

use std::sync::Arc;

use fret_authoring::UiWriter;
use fret_core::Color;
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::UiHost;
use fret_ui_kit::headless::text_assist::TextAssistItem;

use fret_ui_editor::composites::{
    InspectorPanel, InspectorPanelOptions, PropertyGrid, PropertyGridOptions,
    PropertyGridVirtualized, PropertyGridVirtualizedOptions, PropertyGroup, PropertyGroupOptions,
    PropertyRow,
};
use fret_ui_editor::controls::{
    AxisDragValue, AxisDragValueOptions, AxisDragValueOutcome, Checkbox, CheckboxOptions,
    ColorEdit, ColorEditOptions, DragValue, DragValueOptions, DragValueOutcome, EnumSelect,
    EnumSelectItem, EnumSelectOptions, IconButton, IconButtonOptions, MiniSearchBox,
    MiniSearchBoxOptions, NumericInput, NumericInputOptions, NumericPresentation,
    NumericValueConstraints, Slider, SliderOptions, TextAssistField, TextAssistFieldOptions,
    TextAssistFieldSurface, TextField, TextFieldOptions, TransformEdit, TransformEditAxisOutcome,
    TransformEditOptions, TransformEditPresentations, Vec2Edit, Vec3Edit, Vec4Edit,
    VecEditAxisOutcome, VecEditOptions,
};
use fret_ui_editor::imui;

#[allow(dead_code)]
fn editor_imui_adapters_compile<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    name_model: &Model<String>,
    color_model: &Model<Color>,
    value_model: &Model<f64>,
    enabled_model: &Model<bool>,
    mode_model: &Model<Option<Arc<str>>>,
    search_model: &Model<String>,
    search_dismissed_query_model: &Model<String>,
    search_active_item_id_model: &Model<Option<Arc<str>>>,
) {
    let value_presentation = NumericPresentation::<f64>::fixed_decimals(3);
    let blend_presentation = NumericPresentation::<f64>::percent_0_1(0);
    let items: Arc<[EnumSelectItem]> = vec![
        EnumSelectItem::new("lit", "Lit"),
        EnumSelectItem::new("unlit", "Unlit"),
    ]
    .into();
    let assist_items: Arc<[TextAssistItem]> = vec![
        TextAssistItem::new("cube", "Cube"),
        TextAssistItem::new("camera", "Camera"),
    ]
    .into();

    imui::text_field(
        ui,
        TextField::new(name_model.clone()).options(TextFieldOptions {
            clear_button: true,
            ..Default::default()
        }),
    );

    imui::color_edit(
        ui,
        ColorEdit::new(color_model.clone()).options(ColorEditOptions {
            id_source: Some(Arc::from("tests.color_edit")),
            ..Default::default()
        }),
    );

    imui::drag_value(
        ui,
        DragValue::from_presentation(value_model.clone(), value_presentation.clone()).options(
            DragValueOptions {
                id_source: Some(Arc::from("tests.drag_value")),
                ..Default::default()
            },
        ),
    );

    imui::axis_drag_value(
        ui,
        AxisDragValue::from_presentation(
            Arc::from("X"),
            Color::from_srgb_hex_rgb(0xf2_59_59),
            value_model.clone(),
            value_presentation.clone(),
        )
        .options(AxisDragValueOptions {
            id_source: Some(Arc::from("tests.axis_drag_value")),
            ..Default::default()
        }),
    );

    imui::numeric_input(
        ui,
        NumericInput::from_presentation(value_model.clone(), value_presentation.clone()).options(
            NumericInputOptions {
                id_source: Some(Arc::from("tests.numeric_input")),
                ..Default::default()
            },
        ),
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

    let _ = DragValue::from_presentation(value_model.clone(), value_presentation.clone())
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

    let _ = AxisDragValue::from_presentation(
        Arc::from("X"),
        Color::from_srgb_hex_rgb(0xf2_59_59),
        value_model.clone(),
        value_presentation.clone(),
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

    imui::mini_search_box(
        ui,
        MiniSearchBox::new(search_model.clone()).options(MiniSearchBoxOptions {
            test_id: Some(Arc::from("tests.mini_search_box")),
            ..Default::default()
        }),
    );

    imui::text_assist_field(
        ui,
        TextAssistField::new(
            search_model.clone(),
            search_dismissed_query_model.clone(),
            search_active_item_id_model.clone(),
            assist_items,
        )
        .options(TextAssistFieldOptions {
            field: TextFieldOptions {
                id_source: Some(Arc::from("tests.text_assist_field")),
                ..Default::default()
            },
            surface: TextAssistFieldSurface::AnchoredOverlay,
            ..Default::default()
        }),
    );

    imui::icon_button(
        ui,
        IconButton::new(ids::ui::SEARCH, Arc::new(|_host, _action_cx| {})).options(
            IconButtonOptions {
                test_id: Some(Arc::from("tests.icon_button")),
                ..Default::default()
            },
        ),
    );

    imui::vec2_edit(
        ui,
        Vec2Edit::from_presentation(
            value_model.clone(),
            value_model.clone(),
            value_presentation.clone(),
        )
        .options(VecEditOptions {
            id_source: Some(Arc::from("tests.vec2_edit")),
            ..Default::default()
        }),
    );

    let _ = Vec3Edit::from_presentation(
        value_model.clone(),
        value_model.clone(),
        value_model.clone(),
        value_presentation.clone(),
    )
    .on_axis_outcome(Some(on_vec_axis_outcome.clone()))
    .options(Default::default());

    imui::vec3_edit(
        ui,
        Vec3Edit::from_presentation(
            value_model.clone(),
            value_model.clone(),
            value_model.clone(),
            value_presentation.clone(),
        )
        .options(VecEditOptions {
            id_source: Some(Arc::from("tests.vec3_edit")),
            ..Default::default()
        }),
    );

    imui::vec4_edit(
        ui,
        Vec4Edit::from_presentation(
            value_model.clone(),
            value_model.clone(),
            value_model.clone(),
            value_model.clone(),
            value_presentation.clone(),
        )
        .options(VecEditOptions {
            id_source: Some(Arc::from("tests.vec4_edit")),
            ..Default::default()
        }),
    );

    let _ = TransformEdit::from_presentations(
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
        TransformEditPresentations::shared(value_presentation.clone()),
    )
    .on_axis_outcome(Some(on_transform_axis_outcome))
    .options(Default::default());

    imui::transform_edit(
        ui,
        TransformEdit::from_presentations(
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
            TransformEditPresentations::shared(value_presentation.clone()),
        )
        .options(TransformEditOptions {
            id_source: Some(Arc::from("tests.transform_edit")),
            ..Default::default()
        }),
    );

    imui::slider(
        ui,
        Slider::from_presentation(value_model.clone(), 0.0, 1.0, blend_presentation).options(
            SliderOptions {
                id_source: Some(Arc::from("tests.slider")),
                ..Default::default()
            },
        ),
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

    imui::property_group(
        ui,
        PropertyGroup::new("Metadata").options(PropertyGroupOptions {
            test_id: Some(Arc::from("tests.property_group")),
            ..Default::default()
        }),
        |_cx| None,
        move |cx| vec![cx.text("Property group body")],
    );

    imui::property_grid(
        ui,
        PropertyGrid::new().options(PropertyGridOptions {
            test_id: Some(Arc::from("tests.property_grid")),
            ..Default::default()
        }),
        move |cx, row_cx| vec![row_cx.row(cx, |cx| cx.text("Name"), |cx| cx.text("Cube"))],
    );

    imui::property_grid_virtualized(
        ui,
        PropertyGridVirtualized::new().options(PropertyGridVirtualizedOptions {
            id_source: Some(Arc::from("tests.property_grid_virtualized")),
            test_id: Some(Arc::from("tests.property_grid_virtualized")),
            ..Default::default()
        }),
        3,
        |index| index as u64,
        move |cx, index, row_cx| {
            PropertyRow::new()
                .options(row_cx.row_options.clone())
                .into_element(
                    cx,
                    |cx| cx.text(format!("Item {index}")),
                    |cx| cx.text("Value"),
                    |_cx| None,
                )
        },
    );

    imui::inspector_panel(
        ui,
        InspectorPanel::new(None).options(InspectorPanelOptions {
            test_id: Some(Arc::from("tests.inspector_panel")),
            ..Default::default()
        }),
        |_cx, _panel_cx| Vec::new(),
        |_cx, _panel_cx| vec![],
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
    let _ = ColorEditOptions::default();
    let _ = DragValueOptions::default();
    let _ = AxisDragValueOptions::default();
    let _ = NumericInputOptions::default();
    let _ = MiniSearchBoxOptions::default();
    let _ = TextAssistFieldOptions::default();
    let _ = IconButtonOptions::default();
    let _ = SliderOptions::default();
    let _ = CheckboxOptions::default();
    let _ = EnumSelectOptions::default();
    let _ = VecEditOptions::default();
    let _ = TransformEditOptions::default();
    let _ = PropertyGroupOptions::default();
    let _ = PropertyGridOptions::default();
    let _ = PropertyGridVirtualizedOptions::default();
    let _ = InspectorPanelOptions::default();
}
