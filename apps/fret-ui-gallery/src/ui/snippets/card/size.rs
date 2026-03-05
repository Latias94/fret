pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret_app::App;
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    let default_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Default spacing").into_element(cx),
            shadcn::CardDescription::new("CardSize::Default (py-6, px-6, gap-6).").into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![
            ui::text("CardContent text.").text_sm().into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(max_w_sm.clone())
    .into_element(cx)
    .test_id("ui-gallery-card-size-default");

    let sm_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Small spacing").into_element(cx),
            shadcn::CardDescription::new("CardSize::Sm (py-4, px-4, gap-4).").into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![
            ui::text("CardContent text.").text_sm().into_element(cx),
        ])
        .into_element(cx),
    ])
    .size(shadcn::CardSize::Sm)
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-card-size-sm");

    ui::v_flex(|_cx| vec![default_card, sm_card])
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-card-size")
}
// endregion: example
