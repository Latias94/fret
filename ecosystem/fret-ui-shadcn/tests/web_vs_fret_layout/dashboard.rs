use super::*;

#[test]
fn web_vs_fret_layout_dashboard_01_shell_geometry_matches_web() {
    let web = read_web_golden("dashboard-01");
    let theme = web_theme(&web);

    let web_sidebar = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_token(n, "fixed")
            && class_has_token(n, "w-(--sidebar-width)")
            && class_has_token(n, "p-2")
    })
    .expect("web sidebar container");

    let web_header = find_first(&theme.root, &|n| {
        n.tag == "header"
            && class_has_token(n, "h-(--header-height)")
            && class_has_token(n, "border-b")
    })
    .expect("web site header");

    let pad_top = web_header.rect.y;
    let pad_right = theme.viewport.w - (web_header.rect.x + web_header.rect.w);
    let pad_bottom = pad_top;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let sidebar = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_sidebar.rect.w)),
                        height: Length::Px(Px(theme.viewport.h)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        );
        let sidebar = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:dashboard-01:sidebar")),
                ..Default::default()
            },
            move |_cx| vec![sidebar],
        );

        let header = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_header.rect.w)),
                        height: Length::Px(Px(web_header.rect.h)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        );
        let header = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:dashboard-01:header")),
                ..Default::default()
            },
            move |_cx| vec![header],
        );

        let main = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(theme.viewport.w - web_sidebar.rect.w)),
                        height: Length::Px(Px(theme.viewport.h)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                padding: Edges {
                    left: Px(0.0),
                    top: Px(pad_top),
                    right: Px(pad_right),
                    bottom: Px(pad_bottom),
                },
                ..Default::default()
            },
            move |_cx| vec![header],
        );

        vec![cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![sidebar, main],
        )]
    });

    let sidebar = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:dashboard-01:sidebar"),
    )
    .expect("fret dashboard sidebar");
    assert_rect_close_px(
        "dashboard-01 sidebar",
        sidebar.bounds,
        web_sidebar.rect,
        1.0,
    );

    let header = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:dashboard-01:header"),
    )
    .expect("fret dashboard header");
    assert_rect_close_px("dashboard-01 header", header.bounds, web_header.rect, 1.0);
}
