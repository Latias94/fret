use super::*;

fn assert_skeleton_rects_match_web(
    web_name: &str,
    layout: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<AnyElement>,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let mut web_skeletons = find_all(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "bg-accent") && class_has_token(n, "animate-pulse")
    });
    web_skeletons.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .x
                    .partial_cmp(&b.rect.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    assert!(
        !web_skeletons.is_empty(),
        "expected skeleton nodes in {web_name}"
    );

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, layout);

    for (idx, web_node) in web_skeletons.iter().enumerate() {
        let label = format!("Golden:{web_name}:skeleton:{idx}");
        let node = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
            .unwrap_or_else(|| panic!("missing fret skeleton semantics for {label}"));
        assert_rect_close_px(&label, node.bounds, web_node.rect, 1.0);
    }
}

#[test]
fn web_vs_fret_layout_skeleton_demo_rects_match_web() {
    assert_skeleton_rects_match_web("skeleton-demo", |cx| {
        let left = fret_ui_shadcn::Skeleton::new()
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(48.0)))
                    .h_px(MetricRef::Px(Px(48.0))),
            )
            .into_element(cx);
        let left = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:skeleton-demo:skeleton:0")),
                ..Default::default()
            },
            move |_cx| vec![left],
        );

        let line0 = fret_ui_shadcn::Skeleton::new()
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(250.0)))
                    .h_px(MetricRef::Px(Px(16.0))),
            )
            .into_element(cx);
        let line0 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:skeleton-demo:skeleton:1")),
                ..Default::default()
            },
            move |_cx| vec![line0],
        );

        let line1 = fret_ui_shadcn::Skeleton::new()
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(200.0)))
                    .h_px(MetricRef::Px(Px(16.0))),
            )
            .into_element(cx);
        let line1 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:skeleton-demo:skeleton:2")),
                ..Default::default()
            },
            move |_cx| vec![line1],
        );
        let line1 = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(200.0)),
                        height: Length::Px(Px(16.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![line1],
        );

        let col = cx.column(
            ColumnProps {
                layout: LayoutStyle::default(),
                gap: Px(8.0),
                ..Default::default()
            },
            move |_cx| vec![line0, line1],
        );

        vec![cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(16.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| vec![left, col],
        )]
    });
}

#[test]
fn web_vs_fret_layout_skeleton_card_rects_match_web() {
    assert_skeleton_rects_match_web("skeleton-card", |cx| {
        let top = fret_ui_shadcn::Skeleton::new()
            .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(250.0)))
                    .h_px(MetricRef::Px(Px(125.0))),
            )
            .into_element(cx);
        let top = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:skeleton-card:skeleton:0")),
                ..Default::default()
            },
            move |_cx| vec![top],
        );

        let line0 = fret_ui_shadcn::Skeleton::new()
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(250.0)))
                    .h_px(MetricRef::Px(Px(16.0))),
            )
            .into_element(cx);
        let line0 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:skeleton-card:skeleton:1")),
                ..Default::default()
            },
            move |_cx| vec![line0],
        );

        let line1 = fret_ui_shadcn::Skeleton::new()
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(200.0)))
                    .h_px(MetricRef::Px(Px(16.0))),
            )
            .into_element(cx);
        let line1 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:skeleton-card:skeleton:2")),
                ..Default::default()
            },
            move |_cx| vec![line1],
        );
        let line1 = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(200.0)),
                        height: Length::Px(Px(16.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![line1],
        );

        let inner = cx.column(
            ColumnProps {
                layout: LayoutStyle::default(),
                gap: Px(8.0),
                ..Default::default()
            },
            move |_cx| vec![line0, line1],
        );

        vec![cx.column(
            ColumnProps {
                layout: LayoutStyle::default(),
                gap: Px(12.0),
                ..Default::default()
            },
            move |_cx| vec![top, inner],
        )]
    });
}

#[test]
fn web_vs_fret_layout_sonner_demo_button_height_matches_web() {
    let web = read_web_golden("sonner-demo");
    let theme = web_theme(&web);
    let web_button =
        web_find_by_tag_and_text(&theme.root, "button", "Show Toast").expect("web button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        vec![
            fret_ui_shadcn::Button::new("Show Toast")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx),
        ]
    });

    let button = find_semantics(&snap, SemanticsRole::Button, Some("Show Toast"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
        .expect("fret button");

    assert_close_px(
        "sonner-demo button h",
        button.bounds.size.height,
        web_button.rect.h,
        1.0,
    );
}
