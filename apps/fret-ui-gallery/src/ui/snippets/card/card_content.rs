pub const SOURCE: &str = include_str!("card_content.rs");

// region: example
use fret_app::App;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("CardContent inline children").into_element(cx),
            shadcn::CardDescription::new(
                "CardContent should not stretch inline-sized children (e.g. buttons).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![
            shadcn::Button::new("Inline Button")
                .into_element(cx)
                .test_id("ui-gallery-card-content-inline-button"),
        ])
        .into_element(cx),
        shadcn::CardFooter::new(vec![ui::text("Footer").text_sm().into_element(cx)])
            .into_element(cx),
    ])
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-card-content-inline-button-demo")
}
// endregion: example
