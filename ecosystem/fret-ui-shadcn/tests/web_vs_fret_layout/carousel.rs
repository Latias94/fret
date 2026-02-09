use super::*;

#[test]
fn web_vs_fret_layout_carousel_demo_geometry_matches_web() {
    assert_carousel_geometry_matches_web(
        "carousel-demo",
        "max-w-xs",
        &["min-w-0", "shrink-0", "grow-0", "basis-full", "pl-4"],
        |cx| {
            let slides = (1..=5)
                .map(|i| carousel_slide(cx, i, Px(36.0), Px(40.0), true, true))
                .collect::<Vec<_>>();

            fret_ui_shadcn::Carousel::new(slides)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(320.0))))
                .refine_track_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(336.0))))
                .track_start_neg_margin(Space::N4)
                .item_padding_start(Space::N4)
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_layout_carousel_plugin_geometry_matches_web() {
    assert_carousel_geometry_matches_web(
        "carousel-plugin",
        "max-w-xs",
        &["min-w-0", "shrink-0", "grow-0", "basis-full", "pl-4"],
        |cx| {
            let slides = (1..=5)
                .map(|i| carousel_slide(cx, i, Px(36.0), Px(40.0), true, true))
                .collect::<Vec<_>>();

            fret_ui_shadcn::Carousel::new(slides)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(320.0))))
                .refine_track_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(336.0))))
                .track_start_neg_margin(Space::N4)
                .item_padding_start(Space::N4)
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_layout_carousel_api_geometry_matches_web() {
    assert_carousel_geometry_matches_web(
        "carousel-api",
        "max-w-xs",
        &["min-w-0", "shrink-0", "grow-0", "basis-full", "pl-4"],
        |cx| {
            let slides = (1..=5)
                .map(|i| carousel_slide(cx, i, Px(36.0), Px(40.0), true, false))
                .collect::<Vec<_>>();

            let carousel = fret_ui_shadcn::Carousel::new(slides)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(320.0))))
                .refine_track_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(336.0))))
                .track_start_neg_margin(Space::N4)
                .item_padding_start(Space::N4)
                .into_element(cx);

            let caption = ui::text(cx, "Slide 1 of 5")
                .text_size_px(Px(14.0))
                .line_height_px(Px(20.0))
                .text_color(ColorRef::Token {
                    key: "muted-foreground",
                    fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
                })
                .into_element(cx);

            ui::container(cx, move |_cx| vec![carousel, caption])
                .w_full()
                .max_w(MetricRef::Px(Px(320.0)))
                .mx_auto()
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_layout_carousel_size_geometry_matches_web() {
    assert_carousel_geometry_matches_web(
        "carousel-size",
        "max-w-sm",
        &[
            "min-w-0",
            "shrink-0",
            "grow-0",
            "basis-full",
            "pl-4",
            "lg:basis-1/3",
        ],
        |cx| {
            let slides = (1..=5)
                .map(|i| carousel_slide(cx, i, Px(30.0), Px(36.0), true, true))
                .collect::<Vec<_>>();

            fret_ui_shadcn::Carousel::new(slides)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(384.0))))
                .refine_track_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(400.0))))
                .track_start_neg_margin(Space::N4)
                .item_padding_start(Space::N4)
                .item_basis_main_px(Px(133.328))
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_layout_carousel_spacing_geometry_matches_web() {
    assert_carousel_geometry_matches_web(
        "carousel-spacing",
        "max-w-sm",
        &[
            "min-w-0",
            "shrink-0",
            "grow-0",
            "basis-full",
            "pl-1",
            "lg:basis-1/3",
        ],
        |cx| {
            let slides = (1..=5)
                .map(|i| carousel_slide(cx, i, Px(24.0), Px(32.0), true, true))
                .collect::<Vec<_>>();

            fret_ui_shadcn::Carousel::new(slides)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(384.0))))
                .refine_track_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(388.0))))
                .track_start_neg_margin(Space::N1)
                .item_padding_start(Space::N1)
                .item_basis_main_px(Px(129.328))
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_layout_carousel_orientation_geometry_matches_web() {
    assert_carousel_geometry_matches_web(
        "carousel-orientation",
        "max-w-xs",
        &[
            "min-w-0",
            "shrink-0",
            "grow-0",
            "basis-full",
            "pt-1",
            "md:basis-1/2",
        ],
        |cx| {
            let slides = (1..=5)
                .map(|i| carousel_slide(cx, i, Px(30.0), Px(36.0), false, true))
                .collect::<Vec<_>>();

            fret_ui_shadcn::Carousel::new(slides)
                .orientation(fret_ui_shadcn::CarouselOrientation::Vertical)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(320.0))))
                .refine_viewport_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(196.0))))
                .refine_track_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(200.0))))
                .track_start_neg_margin(Space::N1)
                .item_padding_start(Space::N1)
                .into_element(cx)
        },
    );
}
