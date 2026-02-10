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

fn web_find_button_by_sr_text<'a>(root: &'a WebNode, text: &str) -> Option<&'a WebNode> {
    web_find_by_tag_and_text(root, "button", text)
}

fn web_find_carousel_root<'a>(root: &'a WebNode, max_w: &str) -> Option<&'a WebNode> {
    web_find_by_class_tokens(root, &["relative", "w-full", max_w])
}

fn web_find_first_div_by_class_tokens<'a>(
    root: &'a WebNode,
    tokens: &[&str],
) -> Option<&'a WebNode> {
    let mut matches = find_all(root, &|n| n.tag == "div" && class_has_all_tokens(n, tokens));
    matches.sort_by(|a, b| {
        a.rect
            .y
            .total_cmp(&b.rect.y)
            .then_with(|| a.rect.x.total_cmp(&b.rect.x))
    });
    matches.into_iter().next()
}

fn carousel_card_content(
    cx: &mut fret_ui::ElementContext<'_, App>,
    number: u32,
    text_px: Px,
    line_height: Px,
    aspect_square: bool,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let mut layout = LayoutRefinement::default().w_full();
    if aspect_square {
        layout = layout.aspect_ratio(1.0);
    }

    let text = ui::text(cx, format!("{number}"))
        .text_size_px(text_px)
        .line_height_px(line_height)
        .font_semibold()
        .into_element(cx);

    cx.flex(
        FlexProps {
            layout: fret_ui_kit::declarative::style::layout_style(&theme, layout),
            direction: fret_core::Axis::Horizontal,
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            padding: Edges::all(Px(24.0)),
            ..Default::default()
        },
        move |_cx| vec![text],
    )
}

fn carousel_slide(
    cx: &mut fret_ui::ElementContext<'_, App>,
    number: u32,
    text_px: Px,
    line_height: Px,
    aspect_square: bool,
    with_p1_wrapper: bool,
) -> AnyElement {
    let content = carousel_card_content(cx, number, text_px, line_height, aspect_square);
    let card = fret_ui_shadcn::Card::new([content]).into_element(cx);

    if with_p1_wrapper {
        ui::container(cx, move |_cx| vec![card])
            .p_1()
            .into_element(cx)
    } else {
        card
    }
}

fn assert_carousel_geometry_matches_web(
    web_name: &str,
    max_w: &str,
    web_item_tokens: &[&str],
    build: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> AnyElement,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_carousel = web_find_carousel_root(&theme.root, max_w).expect("web carousel root");
    let web_prev =
        web_find_button_by_sr_text(&theme.root, "Previous slide").expect("web prev button");
    let web_next = web_find_button_by_sr_text(&theme.root, "Next slide").expect("web next button");
    let web_item = web_find_first_div_by_class_tokens(&theme.root, web_item_tokens)
        .expect("web carousel item");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| vec![build(cx)]);

    let carousel = find_by_test_id(&snap, "carousel");
    let prev = find_by_test_id(&snap, "carousel-previous");
    let next = find_by_test_id(&snap, "carousel-next");
    let item = find_by_test_id(&snap, "carousel-item-1");

    assert_close_px(
        "carousel width",
        carousel.bounds.size.width,
        web_carousel.rect.w,
        1.0,
    );
    assert_close_px(
        "carousel height",
        carousel.bounds.size.height,
        web_carousel.rect.h,
        1.0,
    );

    assert_close_px("prev width", prev.bounds.size.width, web_prev.rect.w, 1.0);
    assert_close_px("prev height", prev.bounds.size.height, web_prev.rect.h, 1.0);
    assert_close_px("next width", next.bounds.size.width, web_next.rect.w, 1.0);
    assert_close_px("next height", next.bounds.size.height, web_next.rect.h, 1.0);

    assert_close_px(
        "prev dx",
        Px(prev.bounds.origin.x.0 - carousel.bounds.origin.x.0),
        web_prev.rect.x - web_carousel.rect.x,
        1.0,
    );
    assert_close_px(
        "prev dy",
        Px(prev.bounds.origin.y.0 - carousel.bounds.origin.y.0),
        web_prev.rect.y - web_carousel.rect.y,
        1.0,
    );
    assert_close_px(
        "next dx",
        Px(next.bounds.origin.x.0 - carousel.bounds.origin.x.0),
        web_next.rect.x - web_carousel.rect.x,
        1.0,
    );
    assert_close_px(
        "next dy",
        Px(next.bounds.origin.y.0 - carousel.bounds.origin.y.0),
        web_next.rect.y - web_carousel.rect.y,
        1.0,
    );

    assert_close_px(
        "item dx",
        Px(item.bounds.origin.x.0 - carousel.bounds.origin.x.0),
        web_item.rect.x - web_carousel.rect.x,
        1.0,
    );
    assert_close_px(
        "item dy",
        Px(item.bounds.origin.y.0 - carousel.bounds.origin.y.0),
        web_item.rect.y - web_carousel.rect.y,
        1.0,
    );
    assert_close_px("item width", item.bounds.size.width, web_item.rect.w, 1.0);
    assert_close_px("item height", item.bounds.size.height, web_item.rect.h, 1.0);
}
