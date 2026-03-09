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

    shadcn::Card::build(|cx, out| {
        out.extend([
            shadcn::card_header(cx, |cx| {
                vec![
                    shadcn::card_title(cx, "Card Title"),
                    shadcn::card_description(cx, "Card Description"),
                    shadcn::card_action(cx, |cx| {
                        vec![ui::text("Card Action").text_sm().into_element(cx)]
                    }),
                ]
            }),
            shadcn::card_content(cx, |cx| {
                vec![ui::text("Card Content").text_sm().into_element(cx)]
            }),
            shadcn::card_footer(cx, |cx| {
                vec![ui::text("Card Footer").text_sm().into_element(cx)]
            }),
        ]);
    })
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-card-usage")
}
// endregion: example
