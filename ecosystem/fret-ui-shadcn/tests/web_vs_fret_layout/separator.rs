use super::*;

#[test]
fn web_vs_fret_layout_separator_demo_geometry() {
    let web = read_web_golden("separator-demo");
    let theme = web_theme(&web);
    let web_h = find_first(&theme.root, &|n| {
        n.class_name
            .as_deref()
            .is_some_and(|c| c.contains("bg-border shrink-0"))
            && n.attrs
                .get("data-orientation")
                .is_some_and(|o| o == "horizontal")
    })
    .expect("web horizontal separator");
    let web_v = find_first(&theme.root, &|n| {
        n.class_name
            .as_deref()
            .is_some_and(|c| c.contains("bg-border shrink-0"))
            && n.attrs
                .get("data-orientation")
                .is_some_and(|o| o == "vertical")
    })
    .expect("web vertical separator");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, snap, _root) = run_fret_root_with_ui(bounds, |cx| {
        let horizontal = fret_ui_shadcn::Separator::new()
            .orientation(fret_ui_shadcn::SeparatorOrientation::Horizontal)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx);

        let vertical = fret_ui_shadcn::Separator::new()
            .orientation(fret_ui_shadcn::SeparatorOrientation::Vertical)
            .into_element(cx);

        vec![cx.column(
            ColumnProps {
                align: CrossAlign::Start,
                ..Default::default()
            },
            |cx| {
                vec![
                    cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:separator-demo:horizontal")),
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Px(Px(web_h.rect.w)),
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |_cx| vec![horizontal],
                    ),
                    cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:separator-demo:vertical")),
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Auto,
                                    height: Length::Px(Px(web_v.rect.h)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |_cx| vec![vertical],
                    ),
                ]
            },
        )]
    });

    let fret_h = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:separator-demo:horizontal"),
    )
    .expect("fret horizontal separator root");
    let fret_h_child = ui
        .children(fret_h.id)
        .into_iter()
        .next()
        .expect("fret horizontal separator child");
    let fret_h_child_bounds = ui
        .debug_node_bounds(fret_h_child)
        .expect("fret horizontal separator child bounds");
    assert_close_px(
        "separator horizontal inner h",
        fret_h_child_bounds.size.height,
        web_h.rect.h,
        1.0,
    );
    assert_close_px(
        "separator horizontal w",
        fret_h.bounds.size.width,
        web_h.rect.w,
        1.0,
    );
    assert_close_px(
        "separator horizontal h",
        fret_h.bounds.size.height,
        web_h.rect.h,
        1.0,
    );

    let fret_v = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:separator-demo:vertical"),
    )
    .expect("fret vertical separator root");
    let fret_v_child = ui
        .children(fret_v.id)
        .into_iter()
        .next()
        .expect("fret vertical separator child");
    let fret_v_child_bounds = ui
        .debug_node_bounds(fret_v_child)
        .expect("fret vertical separator child bounds");
    assert_close_px(
        "separator vertical inner w",
        fret_v_child_bounds.size.width,
        web_v.rect.w,
        1.0,
    );
    assert_close_px(
        "separator vertical w",
        fret_v.bounds.size.width,
        web_v.rect.w,
        1.0,
    );
    assert_close_px(
        "separator vertical h",
        fret_v.bounds.size.height,
        web_v.rect.h,
        1.0,
    );
}
