pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    let card = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Small Card"),
                    shadcn::card_description("This card uses the small size variant."),
                ]
            }),
            shadcn::card_content(|cx| {
                ui::children![
                    cx;
                    ui::text(
                        "The card component supports a size prop that can be set to \"sm\" for a more compact appearance.",
                    )
                    .text_sm(),
                ]
            }),
            shadcn::card_footer(|cx| {
                ui::children![
                    cx;
                    shadcn::Button::new("Action")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .refine_layout(LayoutRefinement::default().w_full())
                        .ui()
                        .test_id("ui-gallery-card-size-sm-action"),
                ]
            }),
        ]
    })
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
