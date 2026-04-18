pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(192.0)))
        .disabled(true)
        .trigger(shadcn::SelectTrigger::new())
        .value(shadcn::SelectValue::new().placeholder("Select a fruit"))
        .content(shadcn::SelectContent::new())
        .entries([shadcn::SelectGroup::new([
            shadcn::SelectItem::new("apple", "Apple").into(),
            shadcn::SelectItem::new("banana", "Banana").into(),
            shadcn::SelectItem::new("blueberry", "Blueberry").into(),
            shadcn::SelectItem::new("grapes", "Grapes")
                .disabled(true)
                .into(),
            shadcn::SelectItem::new("pineapple", "Pineapple").into(),
        ])
        .into()])
        .into_element(cx)
        .test_id("ui-gallery-select-disabled")
}

// endregion: example
