// region: example
use fret_core::Px;
use fret_ui_kit::primitives::direction::{LayoutDirection, with_direction_provider};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
            .trigger(
                shadcn::SelectTrigger::new()
                    .value(shadcn::SelectValue::new().placeholder("اختر فاكهة")),
            )
            .entries([
                shadcn::SelectGroup::new([
                    shadcn::SelectLabel::new("الفواكه").into(),
                    shadcn::SelectItem::new("apple", "تفاح").into(),
                    shadcn::SelectItem::new("banana", "موز").into(),
                    shadcn::SelectItem::new("blueberry", "توت أزرق").into(),
                ])
                .into(),
                shadcn::SelectSeparator::default().into(),
                shadcn::SelectGroup::new([
                    shadcn::SelectLabel::new("الخضروات").into(),
                    shadcn::SelectItem::new("carrot", "جزر").into(),
                    shadcn::SelectItem::new("broccoli", "بروكلي").into(),
                    shadcn::SelectItem::new("spinach", "سبانخ").into(),
                ])
                .into(),
            ])
            .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
            .into_element(cx)
    })
    .test_id("ui-gallery-select-rtl")
}

// endregion: example
