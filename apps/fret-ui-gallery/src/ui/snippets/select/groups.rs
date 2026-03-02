pub const SOURCE: &str = include_str!("groups.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
        .trigger(
            shadcn::SelectTrigger::new()
                .value(shadcn::SelectValue::new().placeholder("Select a fruit")),
        )
        .entries([
            shadcn::SelectGroup::new([
                shadcn::SelectLabel::new("Fruits").into(),
                shadcn::SelectItem::new("apple", "Apple").into(),
                shadcn::SelectItem::new("banana", "Banana").into(),
                shadcn::SelectItem::new("blueberry", "Blueberry").into(),
            ])
            .into(),
            shadcn::SelectSeparator::default().into(),
            shadcn::SelectGroup::new([
                shadcn::SelectLabel::new("Vegetables").into(),
                shadcn::SelectItem::new("carrot", "Carrot").into(),
                shadcn::SelectItem::new("broccoli", "Broccoli").into(),
                shadcn::SelectItem::new("spinach", "Spinach").into(),
            ])
            .into(),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(192.0)))
        .into_element(cx)
        .test_id("ui-gallery-select-groups")
}

// endregion: example
