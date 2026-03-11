pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret_app::App;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    let card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Small Card").into_element(cx),
            shadcn::CardDescription::new("This card uses the small size variant.")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![ui::text(
            "The card component supports a size prop that can be set to \"sm\" for a more compact appearance.",
        )
        .text_sm()
        .into_element(cx)])
        .into_element(cx),
        shadcn::CardFooter::new(vec![
            shadcn::Button::new("Action")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
                .test_id("ui-gallery-card-size-sm-action"),
        ])
        .into_element(cx),
    ])
    .size(shadcn::CardSize::Sm)
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-card-size-sm");

    ui::v_flex(move |_cx| vec![card])
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-card-size")
}
// endregion: example
