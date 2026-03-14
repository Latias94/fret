pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
            .trigger_test_id("ui-gallery-select-rtl-trigger")
            .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
            .into_element_parts(
                cx,
                |_cx| shadcn::SelectTrigger::new(),
                |_cx| shadcn::SelectValue::new().placeholder("اختر فاكهة"),
                |_cx| {
                    shadcn::SelectContent::new().with_entries([
                        shadcn::SelectGroup::new([
                            shadcn::SelectLabel::new("الفواكه").into(),
                            shadcn::SelectItem::new("apple", "تفاح")
                                .test_id("ui-gallery-select-rtl-item-apple")
                                .into(),
                            shadcn::SelectItem::new("banana", "موز")
                                .test_id("ui-gallery-select-rtl-item-banana")
                                .into(),
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
                },
            )
    })
    .test_id("ui-gallery-select-rtl")
}

// endregion: example
