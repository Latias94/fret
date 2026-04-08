pub const SOURCE: &str = include_str!("card_content.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("CardContent inline children"),
                    shadcn::card_description(
                        "CardContent should not stretch inline-sized children (e.g. buttons).",
                    ),
                ]
            }),
            shadcn::card_content(|cx| {
                ui::children![
                    cx;
                    shadcn::Button::new("Inline Button")
                        .test_id("ui-gallery-card-content-inline-button"),
                ]
            }),
            shadcn::card_footer(|cx| ui::children![cx; ui::text("Footer").text_sm()]),
        ]
    })
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-card-content-inline-button-demo")
}
// endregion: example
