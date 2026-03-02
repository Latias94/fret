pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_app::App;
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Card Title").into_element(cx),
            shadcn::CardDescription::new("Card Description").into_element(cx),
            shadcn::CardAction::new([ui::text(cx, "Card Action").text_sm().into_element(cx)])
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![
            ui::text(cx, "Card Content").text_sm().into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardFooter::new(vec![ui::text(cx, "Card Footer").text_sm().into_element(cx)])
            .into_element(cx),
    ])
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-card-usage")
}
// endregion: example
