pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let control_id = ControlId::from("ui-gallery-select-label");

    let select = shadcn::Select::new_controllable(cx, None, Some("apple"), None, false)
        .control_id(control_id.clone())
        .test_id_prefix("ui-gallery-select-label")
        .into_element_parts(
            cx,
            |_cx| shadcn::SelectTrigger::new(),
            |_cx| shadcn::SelectValue::new(),
            |_cx| {
                shadcn::SelectContent::new().with_entries([
                    shadcn::SelectItem::new("apple", "Apple").into(),
                    shadcn::SelectItem::new("banana", "Banana").into(),
                    shadcn::SelectItem::new("blueberry", "Blueberry").into(),
                    shadcn::SelectItem::new("grapes", "Grapes").into(),
                    shadcn::SelectItem::new("pineapple", "Pineapple").into(),
                ])
            },
        );

    shadcn::FieldGroup::new([shadcn::Field::new([
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
    ])
    .into_element(cx)])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-select-label")
}
// endregion: example
