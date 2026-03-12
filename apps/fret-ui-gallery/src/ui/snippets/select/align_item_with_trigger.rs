pub const SOURCE: &str = include_str!("align_item_with_trigger.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let align_item_with_trigger = cx.local_model(|| true);

    let align = cx
        .get_model_cloned(&align_item_with_trigger, Invalidation::Paint)
        .unwrap_or(true);
    let select = shadcn::Select::new_controllable(cx, None, Some("banana"), None, false)
        .into_element_parts(
            cx,
            |_cx| shadcn::SelectTrigger::new(),
            |_cx| shadcn::SelectValue::new(),
            |_cx| {
                shadcn::SelectContent::new()
                    .position(if align {
                        fret_ui_shadcn::select::SelectPosition::ItemAligned
                    } else {
                        fret_ui_shadcn::select::SelectPosition::Popper
                    })
                    .with_entries([shadcn::SelectGroup::new([
                        shadcn::SelectItem::new("apple", "Apple").into(),
                        shadcn::SelectItem::new("banana", "Banana").into(),
                        shadcn::SelectItem::new("blueberry", "Blueberry").into(),
                        shadcn::SelectItem::new("grapes", "Grapes").into(),
                        shadcn::SelectItem::new("pineapple", "Pineapple").into(),
                    ])
                    .into()])
            },
        );

    shadcn::FieldGroup::new([
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
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx),
        shadcn::Field::new([select])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-select-align-item")
}

// endregion: example
