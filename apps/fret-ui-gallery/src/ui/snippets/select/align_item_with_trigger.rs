pub const SOURCE: &str = include_str!("align_item_with_trigger.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let align_item_with_trigger = cx.local_model(|| true);

    let align = cx
        .get_model_cloned(&align_item_with_trigger, Invalidation::Paint)
        .unwrap_or(true);
    let select = shadcn::Select::new_controllable(cx, None, Some("banana"), None, false)
        .trigger(shadcn::SelectTrigger::new())
        .value(shadcn::SelectValue::new())
        .content(shadcn::SelectContent::new().position(if align {
            shadcn::raw::select::SelectPosition::ItemAligned
        } else {
            shadcn::raw::select::SelectPosition::Popper
        }))
        .entries([shadcn::SelectGroup::new([
            shadcn::SelectItem::new("apple", "Apple").into(),
            shadcn::SelectItem::new("banana", "Banana").into(),
            shadcn::SelectItem::new("blueberry", "Blueberry").into(),
            shadcn::SelectItem::new("grapes", "Grapes").into(),
            shadcn::SelectItem::new("pineapple", "Pineapple").into(),
        ])
        .into()])
        .into_element(cx);

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Position")
                        .for_control("ui-gallery-select-align-item-switch")
                        .into_element(cx),
                    shadcn::FieldDescription::new("Toggle between ItemAligned and Popper positioning.")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Switch::new(align_item_with_trigger.clone())
                    .control_id("ui-gallery-select-align-item-switch")
                    .a11y_label("Select positioning toggle")
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal),
            shadcn::Field::new([select]).refine_layout(LayoutRefinement::default().w_full()),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-select-align-item")
}

// endregion: example
