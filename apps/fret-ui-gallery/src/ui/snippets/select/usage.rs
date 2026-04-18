pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
        .test_id_prefix("ui-gallery-select-usage")
        .trigger(
            shadcn::SelectTrigger::new().refine_layout(LayoutRefinement::default().w_px(Px(180.0))),
        )
        .value(shadcn::SelectValue::new().placeholder("Select a fruit"))
        .content(shadcn::SelectContent::new())
        .entries([shadcn::SelectGroup::new([
            shadcn::SelectLabel::new("Fruits").into(),
            shadcn::SelectItem::new("apple", "Apple").into(),
            shadcn::SelectItem::new("banana", "Banana").into(),
            shadcn::SelectItem::new("blueberry", "Blueberry").into(),
        ])
        .into()])
        .into_element(cx)
}
// endregion: example
