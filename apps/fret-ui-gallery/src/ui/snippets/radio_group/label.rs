pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || Some(Arc::<str>::from("free")));

    let control_id = ControlId::from("ui-gallery-radio-group-label");
    let radio_group = shadcn::radio_group(
        value,
        vec![
            shadcn::RadioGroupItem::new("free", "Free"),
            shadcn::RadioGroupItem::new("pro", "Pro"),
            shadcn::RadioGroupItem::new("enterprise", "Enterprise"),
        ],
    )
    .control_id(control_id.clone())
    .test_id_prefix("ui-gallery-radio-group-label")
    .into_element(cx);

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Plan")
                        .for_control(control_id.clone())
                        .test_id("ui-gallery-radio-group-label-label")
                        .into_element(cx),
                    shadcn::FieldDescription::new("Click the label to focus the active radio item.")
                        .for_control(control_id.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
                radio_group,
            ]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-radio-group-label")
}
// endregion: example
