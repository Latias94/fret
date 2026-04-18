pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let control_id = ControlId::from("ui-gallery-select-label");

    let select = shadcn::Select::new_controllable(cx, None, Some("apple"), None, false)
        .control_id(control_id.clone())
        .test_id_prefix("ui-gallery-select-label")
        .trigger(shadcn::SelectTrigger::new())
        .value(shadcn::SelectValue::new())
        .content(shadcn::SelectContent::new())
        .entries([
            shadcn::SelectItem::new("apple", "Apple").into(),
            shadcn::SelectItem::new("banana", "Banana").into(),
            shadcn::SelectItem::new("blueberry", "Blueberry").into(),
            shadcn::SelectItem::new("grapes", "Grapes").into(),
            shadcn::SelectItem::new("pineapple", "Pineapple").into(),
        ])
        .into_element(cx);

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Fruit")
                        .for_control(control_id.clone())
                        .test_id("ui-gallery-select-label-label")
                        .into_element(cx),
                    shadcn::FieldDescription::new("Click the label to open the Select popup.")
                        .for_control(control_id.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
                select,
            ]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-select-label")
}
// endregion: example
