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
        .trigger_test_id("ui-gallery-select-shadcn-demo-trigger")
        .into_element_parts(
            cx,
            |_cx| {
                shadcn::SelectTrigger::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
            },
            |_cx| shadcn::SelectValue::new().placeholder("Select a fruit"),
            |_cx| {
                shadcn::SelectContent::new().with_entries([shadcn::SelectGroup::new([
                    shadcn::SelectLabel::new("Fruits").into(),
                    shadcn::SelectItem::new("apple", "Apple").into(),
                    shadcn::SelectItem::new("banana", "Banana").into(),
                    shadcn::SelectItem::new("blueberry", "Blueberry").into(),
                    shadcn::SelectItem::new("grapes", "Grapes").into(),
                    shadcn::SelectItem::new("pineapple", "Pineapple").into(),
                ])
                .into()])
            },
        )
}
// endregion: example
