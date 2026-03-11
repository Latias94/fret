pub const SOURCE: &str = include_str!("focus_watch.rs");

// region: example
use fret_app::App;
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let items = (1..=5)
        .map(|idx| {
            let theme = Theme::global(&*cx.app).snapshot();

            let button = shadcn::Button::new(format!("Button {idx}"))
                .test_id(format!("ui-gallery-carousel-focus-button-{idx}"))
                .into_element(cx);

            let body = cx.flex(
                FlexProps {
                    layout: decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default().w_full().h_full(),
                    ),
                    direction: fret_core::Axis::Horizontal,
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    padding: Edges::all(Px(24.0)).into(),
                    ..Default::default()
                },
                move |_cx| vec![button],
            );

            let card = shadcn::Card::new([body]).into_element(cx);
            ui::container(move |_cx| vec![card])
                .w_full()
                .p_1()
                .into_element(cx)
        })
        .map(shadcn::CarouselItem::new)
        .collect::<Vec<_>>();

    shadcn::Carousel::default()
        .opts(
            shadcn::CarouselOptions::new()
                .watch_focus(true)
                .embla_engine(true)
                .ignore_reduced_motion(true),
        )
        .track_start_neg_margin(Space::N0)
        .item_padding_start(Space::N0)
        .item_basis_main_px(Px(200.0))
        .refine_layout(
            LayoutRefinement::default()
                .w_px(Px(200.0))
                .h_px(Px(120.0))
                .mx_auto(),
        )
        .refine_viewport_layout(LayoutRefinement::default().h_px(Px(120.0)))
        .refine_track_layout(LayoutRefinement::default().h_px(Px(120.0)))
        .test_id("ui-gallery-carousel-focus")
        .into_element_parts(
            cx,
            |_cx| shadcn::CarouselContent::new(items),
            shadcn::CarouselPrevious::new(),
            shadcn::CarouselNext::new(),
        )
}
// endregion: example
