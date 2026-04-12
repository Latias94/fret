pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("open", || false);

    shadcn::Select::new(value, open)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(180.0))
                .min_w_0(),
        )
        .trigger_test_id("ui-gallery-select-shadcn-demo-trigger")
        .trigger(shadcn::SelectTrigger::new())
        .value(shadcn::SelectValue::new().placeholder("Select a fruit"))
        .content(shadcn::SelectContent::new())
        .entries([shadcn::SelectGroup::new([
            shadcn::SelectLabel::new("Fruits").into(),
            shadcn::SelectItem::new("apple", "Apple").into(),
            shadcn::SelectItem::new("banana", "Banana").into(),
            shadcn::SelectItem::new("blueberry", "Blueberry").into(),
            shadcn::SelectItem::new("grapes", "Grapes").into(),
            shadcn::SelectItem::new("pineapple", "Pineapple").into(),
        ])
        .into()])
        .into_element(cx)
}
// endregion: example
