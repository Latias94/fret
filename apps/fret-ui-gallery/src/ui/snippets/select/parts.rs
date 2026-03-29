pub const SOURCE: &str = include_str!("parts.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
        .test_id_prefix("ui-gallery-select-composable-parts")
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
