pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Field::build(|cx, out| {
            out.push_ui(cx, shadcn::FieldLabel::new("التعليقات"));
            out.push_ui(
                cx,
                shadcn::Textarea::new(value)
                    .placeholder("تعليقاتك تساعدنا على التحسين...")
                    .rows(4),
            );
            out.push_ui(
                cx,
                shadcn::FieldDescription::new("شاركنا أفكارك حول خدمتنا."),
            );
        })
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
    })
    .test_id("ui-gallery-textarea-rtl")
}
// endregion: example
