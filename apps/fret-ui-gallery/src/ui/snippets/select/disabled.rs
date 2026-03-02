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
        .disabled(true)
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
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(192.0)))
        .into_element(cx)
        .test_id("ui-gallery-select-disabled")
}

// endregion: example
