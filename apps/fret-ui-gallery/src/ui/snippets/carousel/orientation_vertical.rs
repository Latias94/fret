pub const SOURCE: &str = include_str!("orientation_vertical.rs");

// region: example
use fret_app::App;
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_xs = Px(320.0);

    let vertical_items = (1..=5)
        .map(|idx| {
            let theme = Theme::global(&*cx.app).clone();
            let number = ui::text(cx, format!("{idx}"))
                .text_size_px(Px(30.0))
                .line_height_px(Px(36.0))
                .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
                .font_semibold()
                .into_element(cx);

            let body = cx.flex(
                FlexProps {
                    layout: decl_style::layout_style(&theme, LayoutRefinement::default().w_full()),
                    direction: fret_core::Axis::Horizontal,
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    padding: Edges::all(Px(24.0)).into(),
                    ..Default::default()
                },
                move |_cx| vec![number],
            );

            let card = shadcn::Card::new([body]).into_element(cx);
            ui::container(cx, move |_cx| vec![card])
                .w_full()
                .p_1()
                .into_element(cx)
        })
        .collect::<Vec<_>>();

    shadcn::Carousel::new(vertical_items)
        .orientation(shadcn::CarouselOrientation::Vertical)
        .opts(shadcn::CarouselOptions::new().align(shadcn::CarouselAlign::Start))
        .item_basis_main_px(Px(100.0))
        .refine_viewport_layout(LayoutRefinement::default().h_px(Px(200.0)))
        .refine_track_layout(LayoutRefinement::default().h_px(Px(200.0)))
        .track_start_neg_margin(Space::N1)
        .item_padding_start(Space::N1)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_xs)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-orientation-vertical")
        .into_element(cx)
}
// endregion: example
