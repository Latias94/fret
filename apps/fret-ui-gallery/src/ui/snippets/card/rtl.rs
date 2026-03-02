// region: example
use fret_app::App;
use fret_ui_kit::primitives::direction::{LayoutDirection, with_direction_provider};
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("نموذج بطاقة").into_element(cx),
                shadcn::CardDescription::new("تأكد من أن Card يحترم اتجاه RTL.").into_element(cx),
                shadcn::CardAction::new([shadcn::Button::new("إجراء")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .into_element(cx)])
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![ui::text(cx, "محتوى البطاقة").text_sm().into_element(cx)])
                .into_element(cx),
        ])
        .refine_layout(max_w_sm)
        .into_element(cx)
    })
    .test_id("ui-gallery-card-rtl")
}
// endregion: example

