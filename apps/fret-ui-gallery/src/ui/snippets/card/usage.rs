pub const SOURCE: &str = include_str!("usage.rs");

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
                    shadcn::card_title("Card Title"),
                    shadcn::card_description("Card Description"),
                    shadcn::card_action(
                        |cx| ui::children![cx; ui::text("Card Action")]
                    ),
                ]
            }),
            shadcn::card_content(|cx| ui::children![cx; ui::text("Card Content")]),
            shadcn::card_footer(|cx| ui::children![cx; ui::text("Card Footer")]),
        ]
    })
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-card-usage")
}
// endregion: example
